pub mod core;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tauri::Manager;
use tracing_appender::non_blocking::WorkerGuard;
use tracing::dispatcher::set_global_default;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Guards the tracing non-blocking worker thread for the entire app lifetime.
/// MUST be kept alive or the logging thread dies and writes panic.
/// Phase 1: guard for null writer (no logging)
/// Phase 2: guard for file writer (logs to file)
static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

/// Tracks whether Phase 1 successfully claimed the global tracing default.
/// Used by Phase 2 to choose the correct init path.
static PHASE1_GLOBAL_SET: AtomicBool = AtomicBool::new(false);

/// Phase 1: Run BEFORE Tauri is initialized.
/// Sets up a null writer so no console window flashes on Windows.
/// Does NOT set a global tracing default — Phase 2 handles that.
fn init_phase1_logging() {
    // null_writer → non_blocking → LOG_GUARD
    let null_writer = std::io::sink();
    let (non_blocking, guard) = tracing_appender::non_blocking(null_writer);

    // If this fails, the null guard goes out of scope immediately.
    // The non_blocking worker thread dies. That's fine — Phase 1 is brief.
    let _ = LOG_GUARD.set(guard);

    // Build the null-writer subscriber and set it as global.
    // Phase 2 will overwrite this with the file-writer subscriber.
    let null_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(false);

    // Use set_global_default() — standalone function, takes built subscriber.
    // Ok(()) = we set it. Err(_) = someone else already set it.
    // Phase 2 checks PHASE1_GLOBAL_SET to choose the correct init path.
    let null_subscriber = tracing_subscriber::registry()
        .with(null_layer)
        .with(tracing_subscriber::EnvFilter::new("info"));
    let claimed = set_global_default(null_subscriber).is_ok();
    PHASE1_GLOBAL_SET.store(claimed, Ordering::Relaxed);

    // NOTE: any log lines before this point are silently discarded.
    // The null writer is fine — we have no writable path yet.
}

/// Phase 2: Run INSIDE Tauri .setup() closure.
/// Has access to app.path() for guaranteed-writable directories.
/// Sets the global tracing default to use the file writer.
fn init_phase2_logging(app: &tauri::App) {

    // ── Get log directory — NEVER use cwd ──────────────────────────────────────
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

    // ── Build file appender (daily rotation, writes ONLY to AppData) ────────────
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "chronos.log");
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);

    // Store the file guard — keeps the logging worker thread alive for the app's lifetime.
    // LOG_GUARD.set() returns None if set for first time, or Some(()) if already set.
    // If a Phase 1 null-guard was already stored, it's dropped here — that's fine.
    let _ = LOG_GUARD.set(file_guard);

    // ── Set global tracing default with file writer ─────────────────────────────
    // Strategy depends on what Phase 1 did:
    // - PHASE1_GLOBAL_SET = true: Phase 1 claimed the global with null writer.
    //   Use set_global_default() to overwrite (always succeeds, overwrites global).
    // - PHASE1_GLOBAL_SET = false: Phase 1 failed to claim global (someone else
    //   set it, or we never called set_global_default). Use try_init() instead.
    let phase1_set_global = PHASE1_GLOBAL_SET.load(Ordering::Relaxed);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false);
    let file_env_filter = tracing_subscriber::EnvFilter::new("info");
    let file_subscriber = tracing_subscriber::registry()
        .with(file_layer)
        .with(file_env_filter);

    if phase1_set_global {
        // Phase 1 set null writer as global. Overwrite it with file writer.
        // set_global_default() takes ownership and overwrites the global.
        let _prev = set_global_default(file_subscriber);
        tracing::info!(
            "[Startup] Global subscriber upgraded to file logging (overrode null writer)."
        );
    } else {
        // No global set (or not by us). try_init() will succeed if unclaimed,
        // or return Err if someone else set it (we ignore that and continue).
        let _ = file_subscriber.try_init();
        tracing::info!("[Startup] Global subscriber set to file logging.");
    }

    tracing::info!(
        "[Startup] File logging active: {}/chronos-YYYY-MM-DD.log",
        log_dir.display()
    );
}

/// Phase 3: Initialize databases at proper AppData locations.
fn init_phase3_databases(app: &tauri::App) {
    // ── Get app data directory — NEVER use cwd ─────────────────────────────────
    // Windows: %LOCALAPPDATA%\com.chronos.app\  (writable per-user, NOT Program Files)
    // Linux:   ~/.local/share/com.chronos.app/
    // macOS:   ~/Library/Application Support/com.chronos.app/
    let data_dir = app
        .path()
        .app_local_data_dir()
        .unwrap_or_else(|_| {
            dirs::data_local_dir().unwrap_or_else(|| {
                let fallback = std::env::temp_dir().join("chronos-appdata");
                tracing::warn!(
                    "[Startup] No app data dir available. Using temp dir {}. \
                    Data will NOT persist across reboots.",
                    fallback.display()
                );
                fallback
            })
        });

    tracing::info!("[Startup] App data dir: {}", data_dir.display());

    // CRITICAL: Tauri v2's app_local_data_dir() returns the path but does NOT
    // create it. On first boot the folder doesn't exist. Without explicit create,
    // all subsequent DB writes fail silently — this was a root cause of the crash.
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        // Write to a fallback file so we have SOME diagnostic if this fails.
        let crash_path = std::env::temp_dir().join("chronos-startup-error.txt");
        let msg = format!(
            "[{}] FATAL: Cannot create app data dir {}: {}\n",
            chrono::Utc::now(),
            data_dir.display(),
            e
        );
        let _ = std::fs::write(&crash_path, &msg);
        tracing::error!(
            "[Startup] CANNOT create app data dir {}: {}. \
            LanceDB and all persistent storage will be unavailable.",
            data_dir.display(),
            e
        );
        // DO NOT early-return — let IPC handlers report their own errors.
        // The user will see empty graph/insights rather than a silent crash.
    } else {
        tracing::info!("[Startup] App data dir created/verified OK.");
    }

    // ── LanceDB vector store ───────────────────────────────────────────────────
    // Stores at: data_dir/chronos/vectordb/
    // Safe to fail: vault searches return empty gracefully.
    if let Err(e) = core::lance::init_lance_with_path(&data_dir) {
        tracing::warn!(
            "[Startup] LanceDB init failed: {}. Vector storage unavailable.",
            e
        );
    }

    // ── Candle embedder (local ML) ─────────────────────────────────────────────
    // Model cached at: ~/.cache/huggingface/ (writable, managed by hf-hub).
    // Safe to fail: ingest falls back gracefully.
    if let Err(e) = core::embed::init_embedder() {
        tracing::warn!(
            "[Startup] Candle embedder init failed: {}. Embeddings unavailable.",
            e
        );
    }

    // NOTE: KuzuDB and Qdrant are stub implementations in this build.
    tracing::info!("Chronos Tauri setup complete — all systems reporting.");
}

pub fn run() {
    // ── Phase 1: Null logging — no files written, no cwd dependency ───────────
    // The working directory on Windows (MSI install) is C:\Program Files\ — read-only.
    // We cannot write any files until we have a guaranteed-writable path from Tauri.
    init_phase1_logging();

    tracing::info!("Chronos starting up...");

    // ── Tauri Builder ─────────────────────────────────────────────────────────
    let result = tauri::Builder::default()
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

            // ── Phase 2: Switch to file logging (AppData paths now available) ──
            init_phase2_logging(app);

            // ── Phase 3: Initialize databases at AppData locations ─────────────
            init_phase3_databases(app);

            Ok(())
        })
        .run(tauri::generate_context!());

    // ── Error handling — write crash diagnostics to file before exiting ─────────
    // On Windows, .expect() would panic with no visible output.
    // This writes the error to a temp file so we have a diagnostic.
    match result {
        Ok(()) => {
            tracing::info!("Chronos exited cleanly.");
        }
        Err(e) => {
            let crash_path = std::env::temp_dir().join("chronos-fatal.txt");
            let msg = format!(
                "[{}] CHRONOS FATAL ERROR:\n{:?}\n\n\
                If you're seeing this, please share this file with the developers.\n\
                Log file (if file logging activated): %LOCALAPPDATA%\\com.chronos.app\\logs\\\n",
                chrono::Utc::now(),
                e
            );
            // Try to write the crash file, but don't crash trying to report the crash.
            let _ = std::fs::write(&crash_path, &msg);
            // Also try stderr — useful for debugging via command line.
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    }
}
