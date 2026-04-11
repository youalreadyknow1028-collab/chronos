//! MiniMax API Client
//! See EPIC_4 §A

use super::config::{MiniMaxConfig, t4_minimax_config};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MiniMaxError {
    #[error("HTTP error: {0}")]
    HttpError(String),
    #[error("API error: {code} — {message}")]
    ApiError { code: u16, message: String },
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Budget exhausted")]
    BudgetExhausted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniMaxRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MiniMaxResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub finish_reason: String,
    pub message: Message,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub struct MiniMaxClient {
    config: MiniMaxConfig,
    http_client: reqwest::Client,
}

impl MiniMaxClient {
    pub fn new(config: MiniMaxConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("reqwest client must build");
        Self { config, http_client }
    }

    pub fn with_default() -> Self {
        Self::new(t4_minimax_config())
    }

    pub async fn chat(
        &self,
        messages: Vec<Message>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<MiniMaxResponse, MiniMaxError> {
        let api_key = std::env::var(&self.config.api_key_env)
            .map_err(|_| MiniMaxError::HttpError("MINIMAX_API_KEY not set".to_string()))?;

        let request = MiniMaxRequest {
            model: self.config.model.clone(),
            messages,
            temperature: temperature.or(Some(self.config.temperature)),
            max_tokens: max_tokens.or(Some(self.config.max_tokens)),
        };

        let resp = self.http_client
            .post(&self.config.base_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| MiniMaxError::HttpError(e.to_string()))?;

        let status = resp.status().as_u16();
        let body = resp.text().await.map_err(|e| MiniMaxError::HttpError(e.to_string()))?;

        if status != 200 {
            return Err(MiniMaxError::ApiError {
                code: status,
                message: body.clone(),
            });
        }

        serde_json::from_str(&body).map_err(|e| MiniMaxError::ParseError(e.to_string()))
    }

    pub fn estimate_tokens(&self, messages: &[Message]) -> u32 {
        // Rough estimation: ~4 chars per token
        messages.iter().map(|m| m.content.len() as u32 / 4).sum()
    }
}
