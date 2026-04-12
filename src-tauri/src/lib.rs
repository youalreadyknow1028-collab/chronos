pub mod core;

use dirs;
use std::sync::OnceLock;
use tauri::Manager;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn run() {
    // v0.2.2 fix: Use dirs::data_local_dir() for logs at startup.
    // After .setup() runs, we switch to the proper app data dir for LanceDB.
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::temp_dir())
        .join("chronos")
        .join("logs");
    std::fs::create_dir_all(&log_dir).ok();
    let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "chronos.log");
    let (non_blocking_writer, guard) = tracing_appender::non_blocking(file_appender);
    let _ = LOG_GUARD.set(guard);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking_writer))
        .with(tracing_subscriber::EnvFilter::new("info"))
        .init();

    tracing::info!("Chronos starting up...");

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

            // === Phase B: Initialize LanceDB vector store ===
            // Use Tauri's app.path().app_local_data_dir() which gives:
            //   Windows: %LOCALAPPDATA%\com.chronos.app\  (writable, NOT Program Files)
            //   Linux:   ~/.local/share/com.chronos.app/
            //   macOS:   ~/Library/Application Support/com.chronos.app/
            //
            // This fixes the silent startup crash on Windows where the app was
            // trying to write to the protected C:\Program Files directory.
            let data_dir = app
                .path()
                .app_local_data_dir()
                .unwrap_or_else(|_| {
                    dirs::data_local_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                });

            tracing::info!("[Startup] App data dir: {}", data_dir.display());

            // CRITICAL: Create the app data directory before any database init.
            // Tauri v2's app_local_data_dir() returns the path but does NOT create it.
            // On first boot the folder doesn't exist — without this, DB init silently fails.
            if let Err(e) = std::fs::create_dir_all(&data_dir) {
                tracing::error!(
                    "[Startup] FAILED to create app data dir {}: {}. App may not function.",
                    data_dir.display(),
                    e
                );
                // Continue anyway — let the DB init report its own error
            } else {
                tracing::info!("[Startup] App data dir created/verified.");
            }

            // Initialize LanceDB at the proper AppData location
            if let Err(e) = core::lance::init_lance_with_path(&data_dir) {
                tracing::warn!(
                    "[Startup] LanceDB init failed: {}. Vector storage unavailable.",
                    e
                );
            }

            // === Phase B: Initialize candle embedder ===
            // Downloads model from HuggingFace Hub (~90MB, cached at ~/.cache/huggingface/).
            // hf_hub uses HF_HOME env var or defaults to ~/.cache/huggingface/ — user-writable.
            if let Err(e) = core::embed::init_embedder() {
                tracing::warn!(
                    "[Startup] Embedder init failed: {}. Embeddings unavailable.",
                    e
                );
            }

            tracing::info!("Chronos Tauri app setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
