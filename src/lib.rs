pub mod core;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

pub fn run() {
    let log_dir = std::env::current_dir().unwrap_or_default().join("logs");
    std::fs::create_dir_all(&log_dir).ok();
    let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "chronos.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

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
            core::ipc::health::health_check,
            core::ipc::vault::vault_ingest_file,
            core::ipc::vault::vault_search_notes,
            core::ipc::vault::vault_delete_note,
            core::ipc::graph::graph_get_nodes,
            core::ipc::graph::graph_get_timeline,
            core::ipc::insights::insights_get_pending,
            core::ipc::pipeline::pipeline_ingest,
            core::ipc::pipeline::pipeline_diary_write,
            core::ipc::pipeline::pipeline_budget_status,
            core::ipc::pipeline::pipeline_status,
            core::ipc::pipeline::pipeline_trigger_cron,
        ])
        .setup(|_app| {
            tracing::info!("Chronos Tauri app setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
