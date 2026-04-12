pub mod core;

use dirs;
use std::sync::OnceLock;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

pub fn run() {
    // v0.2.1 hotfix: Use OS AppData directory instead of current working directory.
    // This prevents permission crashes on Windows when installed in Program Files.
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("chronos")
        .join("logs");
    std::fs::create_dir_all(&log_dir).ok();
    let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "chronos.log");
    let (_non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    // Leak the guard so logging works for the entire lifetime of the app.
    let _ = LOG_GUARD.set(guard);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .with(tracing_subscriber::EnvFilter::new("info"))
        .init();

    tracing::info!("Chronos starting up...");

    // Phase B: Initialize LanceDB vector store
    if let Err(e) = core::lance::init_lance() {
        tracing::warn!("[Startup] LanceDB init failed: {}. Vector storage unavailable.", e);
    }

    // Phase B: Initialize candle embedder (downloads model from HuggingFace Hub on first run)
    if let Err(e) = core::embed::init_embedder() {
        tracing::warn!("[Startup] Candle embedder init failed: {}. Embeddings unavailable.", e);
    }

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
        .setup(|_app| {
            tracing::info!("Chronos Tauri app setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
