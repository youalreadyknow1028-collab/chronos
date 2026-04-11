//! MiniMax API Configuration
//! See EPIC_4 §A.4

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MiniMaxConfig {
    pub model: String,
    pub base_url: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_secs: u64,
    pub api_key_env: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeminiConfig {
    pub model: String,
    pub base_url: String,
    pub max_output_tokens: u32,
    pub temperature: f32,
    pub timeout_secs: u64,
    pub api_key_env: String,
}

impl MiniMaxConfig {
    pub fn t4_default() -> Self {
        Self {
            model: "MiniMax-M2.7-highspeed".to_string(),
            base_url: "https://api.minimax.chat/v1/text/chatcompletion_v2".to_string(),
            max_tokens: 8192,
            temperature: 0.7,
            timeout_secs: 120,
            api_key_env: "MINIMAX_API_KEY".to_string(),
        }
    }

    pub fn t3_default() -> Self {
        Self {
            model: "MiniMax-M2.7-highspeed".to_string(),
            base_url: "https://api.minimax.chat/v1/text/chatcompletion_v2".to_string(),
            max_tokens: 2048,
            temperature: 0.5,
            timeout_secs: 60,
            api_key_env: "MINIMAX_API_KEY".to_string(),
        }
    }
}

impl GeminiConfig {
    pub fn t3_fallback() -> Self {
        Self {
            model: "gemini-2.0-flash".to_string(),
            base_url: "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent".to_string(),
            max_output_tokens: 2048,
            temperature: 0.5,
            timeout_secs: 60,
            api_key_env: "GEMINI_API_KEY".to_string(),
        }
    }
}

// Config accessors — use these in the router
pub fn t4_minimax_config() -> MiniMaxConfig { MiniMaxConfig::t4_default() }
pub fn t3_minimax_config() -> MiniMaxConfig { MiniMaxConfig::t3_default() }
pub fn t3_gemini_config() -> GeminiConfig { GeminiConfig::t3_fallback() }
