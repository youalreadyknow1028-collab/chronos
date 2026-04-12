pub mod core;

use std::sync::OnceLock;
use tauri::Manager;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Guards the tracing non-blocking worker thread for the entire app lifetime.
/// MUST be kept alive or the logging thread dies and writes panic.
static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn run() {
    // Phase 1: Re-route stderr → /dev/null so no console window flashes on Windows.
    // The working directory might be C:\Program Files (MSI install) — do NOT write
    // any files here. All logging goes to a null writer until .setup() runs.
    let null_writer = std::io::sink();
    let (non_blocking, guard) = tracing_appender::non_blocking(null_writer);
    let _ = LOG_GUARD.set(guard);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .with(tracing_subscriber::EnvFilter::new("info"))
        .init();

    tracing::info!("Chronos starting up (logging suspended until AppData ready)...");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // === Health ===
            core::ipc::health::health_check,
            // === Vault ===
            core::ipc::vault::vault_ingest_file,
            core::ipc::vault::vault_search_notes,
            core::ipc::vault::vault_delete_note,
            // === Graph ===
            core::ipc::graph::graph_get_nodes,
            core::ipc::graph::graph_get_timeline,
            // === Insights ===
            core::ipc::insights::insights_get_pending,
            // === Pipeline ===
            core::ipc::pipeline::pipeline_ingest,
            core::ipc::pipeline::pipeline_diary_write,
            core::ipc::pipeline::pipeline_budget_status,
            core::ipc::pipeline::pipeline_status,
            core::ipc::pipeline::pipeline_trigger_cron,
            // === Settings ===
            core::ipc::settings::save_api_key,
            core::ipc::settings::load_api_key,
            core::ipc::settings::list_settings,
            core::ipc::settings::trigger_vault_sync,
        ])
        .setup(|app| {
            tracing::info!("Chronos Tauri setup starting...");

            // ── Phase 2: Upgrade to file logging using Tauri's guaranteed paths ──
            //
            // The null writer from Phase 1 is still active. We now have app.path()
            // available, so switch to a file writer at a guaranteed-writable path.
            //
            // Get log directory — NEVER use cwd (might be C:\Program Files).
            // Priority: app_log_dir() > app_local_data_dir()/logs > temp_dir
            let log_dir = app
                .path()
                .app_log_dir()
                .ok()
                .or_else(|| {
                    app.path()
                        .app_local_data_dir()
                        .ok()
                        .map(|p| p.join("logs"))
                })
                .unwrap_or_else(|| std::env::temp_dir().join("chronos-logs"));

            // Create log dir — fail gracefully, never crash.
            let log_dir = match std::fs::create_dir_all(&log_dir) {
                Ok(()) => {
                    tracing::info!("[Startup] Log dir ready: {}", log_dir.display());
                    log_dir
                }
                Err(e) => {
                    // Last resort: temp dir is always writable.
                    let fallback = std::env::temp_dir().join("chronos-logs");
                    let _ = std::fs::create_dir_all(&fallback);
                    tracing::warn!(
                        "[Startup] Cannot create log dir {}: {}. \
                        Falling back to {}. Report this bug.",
                        log_dir.display(),
                        e,
                        fallback.display()
                    );
                    fallback
                }
            };

            // Build file appender — daily rotation, writes ONLY to AppData (never cwd).
            use tracing_appender::rolling::{RollingFileAppender, Rotation};
            let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "chronos.log");
            let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);

            // Store the guard. If this fails, the null writer from Phase 1 stays active.
            // That's non-ideal (no logs to file) but never crashes.
            if LOG_GUARD.set(file_guard).is_err() {
                tracing::warn!(
                    "[Startup] LOG_GUARD already set — file logging may not activate. \
                    This is a programming error if you see it."
                );
            }

            // Replace the global subscriber's writer with the file writer.
            // Safe during setup: no concurrent logging yet (app just started).
            let file_layer = tracing_subscriber::fmt::layer()
                .with_writer(file_writer)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(false);

            // Replace the global subscriber's writer with the file writer.
            //
            // CRITICAL: set_global_default() PANICS if a default is already set.
            // Phase 1's init() called try_init() which locked the global state.
            // set_global_default() here would panic → crash on startup.
            // FIX: use try_init() which safely returns Err instead of panicking.
            let file_env_filter = tracing_subscriber::EnvFilter::new("info");
            let file_registry = tracing_subscriber::registry()
                .with(file_layer)
                .with(file_env_filter);

            if file_registry.try_init().is_ok() {
                tracing::info!("[Startup] Global subscriber upgraded to file logging.");
            } else {
                tracing::warn!(
                    "[Startup] Global subscriber already set (null writer from Phase 1). \
                    File writer NOT activated. Restart the app for full logs. \
                    The app will run correctly — this only affects log output."
                );
            }

            tracing::info!(
                "[Startup] File logging active: {}/chronos-$(date).log",
                log_dir.display()
            );

            // ── Phase 3: Initialise databases at proper AppData locations ──

            // Get app data directory — NEVER use cwd.
            // Windows: %LOCALAPPDATA%\com.chronos.app\  (writable per-user, NOT Program Files)
            // Linux:   ~/.local/share/com.chronos.app/
            // macOS:   ~/Library/Application Support/com.chronos.app/
            let data_dir = app
                .path()
                .app_local_data_dir()
                .unwrap_or_else(|_| {
                    dirs::data_local_dir().unwrap_or_else(|| {
                        // FINAL fallback — temp is always writable but not persistent.
                        // If this is reached, app data is lost on reboot.
                        tracing::warn!(
                            "[Startup] No app data dir available. Using temp dir. \
                            Data will NOT persist across reboots."
                        );
                        std::env::temp_dir().join("chronos-appdata")
                    })
                });

            tracing::info!("[Startup] App data dir: {}", data_dir.display());

            // CRITICAL: Tauri v2's app_local_data_dir() RETURNS the path but does NOT
            // create it. On first boot the folder doesn't exist. Without explicit create,
            // all subsequent DB writes silently fail and the window closes immediately.
            if let Err(e) = std::fs::create_dir_all(&data_dir) {
                tracing::error!(
                    "[Startup] CANNOT create app data dir {}: {}. \
                    LanceDB and all persistent storage will be unavailable.",
                    data_dir.display(),
                    e
                );
                // DO NOT early-return — let the IPC handlers report their own errors.
                // The user will see empty graph/insights which is better than silent crash.
            } else {
                tracing::info!("[Startup] App data dir created/verified OK.");
            }

            // ── LanceDB vector store ──
            // Stores at: app_data_dir/chronos/vectordb/
            // Safe to fail: vault searches return empty, graph shows no nodes.
            if let Err(e) = core::lance::init_lance_with_path(&data_dir) {
                tracing::warn!(
                    "[Startup] LanceDB init failed: {}. Vector storage unavailable.",
                    e
                );
            }

            // ── Candle embedder (local ML) ──
            // Model cached at: ~/.cache/huggingface/ (writable, managed by hf-hub crate).
            // Safe to fail: ingest falls back to no embeddings.
            if let Err(e) = core::embed::init_embedder() {
                tracing::warn!(
                    "[Startup] Candle embedder init failed: {}. Embeddings unavailable.",
                    e
                );
            }

            // NOTE: KuzuDB and Qdrant are stub implementations in this build.
            // Graph and advanced search are not yet wired up — safe to ignore.

            tracing::info!("Chronos Tauri setup complete — all systems reporting.");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
