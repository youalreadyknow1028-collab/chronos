//! Settings IPC — API key storage and vault sync via Tauri commands.

use crate::core::minimax::TierRouter;
use crate::core::pipeline::IngestRequest;
use serde::Serialize;
use std::path::Path;
use tracing::info;

/// Save an API key to ~/.config/chronos/.env
#[tauri::command]
pub fn save_api_key(key_name: String, key_value: String) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("Cannot find config directory")?
        .join("chronos");
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    let env_file = config_dir.join(".env");

    let content = if env_file.exists() {
        std::fs::read_to_string(&env_file).unwrap_or_default()
    } else {
        String::new()
    };

    let new_line = format!("{}={}", key_name, key_value);
    let lines: Vec<String> = content
        .lines()
        .filter(|l| !l.starts_with(&format!("{}=", key_name)))
        .map(|s| s.to_string())
        .collect();
    let mut lines = lines;
    lines.push(new_line);

    std::fs::write(&env_file, lines.join("\n")).map_err(|e| e.to_string())?;
    info!("[Settings] Saved key: {}", key_name);
    Ok(())
}

/// Load an API key value from ~/.config/chronos/.env
#[tauri::command]
pub fn load_api_key(key_name: String) -> Result<String, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Cannot find config directory")?
        .join("chronos");
    let env_file = config_dir.join(".env");

    if !env_file.exists() {
        return Err("No settings found".to_string());
    }

    let content = std::fs::read_to_string(&env_file).map_err(|e| e.to_string())?;
    for line in content.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if k == key_name {
                return Ok(v.to_string());
            }
        }
    }
    Err("Key not found".to_string())
}

/// List all settings (keys with masked values)
#[tauri::command]
pub fn list_settings() -> Result<Vec<(String, String)>, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Cannot find config directory")?
        .join("chronos");
    let env_file = config_dir.join(".env");

    if !env_file.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&env_file).map_err(|e| e.to_string())?;
    let mut result = vec![];
    for line in content.lines() {
        if let Some((k, v)) = line.split_once('=') {
            let masked = if v.len() > 4 {
                format!("{}****{}", &v[..2], &v[v.len() - 2..])
            } else {
                "****".to_string()
            };
            result.push((k.to_string(), masked));
        }
    }
    Ok(result)
}

/// Trigger a vault sync — walks directory and ingests all markdown/text files
#[tauri::command]
pub async fn trigger_vault_sync(directory_path: String) -> Result<SyncResult, String> {
    info!("[VaultSync] Starting sync of: {}", directory_path);
    let mut processed = 0u32;
    let mut errors = vec![];

    fn is_text_file(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| {
                matches!(
                    e.to_lowercase().as_str(),
                    "md" | "txt" | "org" | "note" | "markdown"
                )
            })
            .unwrap_or(false)
    }

    fn walk_and_ingest(dir: &Path, processed: &mut u32, errors: &mut Vec<String>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                errors.push(format!("Cannot read dir {}: {}", dir.display(), e));
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_and_ingest(&path, processed, errors);
            } else if is_text_file(&path) {
                let content = match std::fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(e) => {
                        errors.push(format!("{}: {}", path.display(), e));
                        continue;
                    }
                };

                let request = IngestRequest {
                    content,
                    source: format!("vault:{}", path.display()),
                    tags: vec!["vault".to_string(), "sync".to_string()],
                };

                let rt = tokio::runtime::Handle::current();
                let result =
                    rt.block_on(crate::core::pipeline::pipeline_ingest(request, &TierRouter::new()));
                match result {
                    Ok(_) => {
                        *processed += 1;
                        info!("[VaultSync] Ingested: {}", path.display());
                    }
                    Err(e) => {
                        errors.push(format!("{}: [{}] {}", path.display(), e.code, e.message));
                    }
                }
            }
        }
    }

    let path = Path::new(&directory_path);
    walk_and_ingest(path, &mut processed, &mut errors);

    info!(
        "[VaultSync] Complete: {} processed, {} errors",
        processed,
        errors.len()
    );
    Ok(SyncResult {
        processed,
        errors: errors.into_iter().take(20).collect(),
    })
}

#[derive(Serialize)]
pub struct SyncResult {
    pub processed: u32,
    pub errors: Vec<String>,
}
