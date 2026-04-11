//! Gemini API Client — T3 retry fallback
//! See EPIC_4 §A.4

use super::config::GeminiConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeminiError {
    #[error("HTTP error: {0}")]
    HttpError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub max_output_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeminiResponse {
    #[serde(rename = "candidates")]
    pub candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Candidate {
    pub content: Content,
    #[serde(rename = "finishReason")]
    pub finish_reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_tokens: u32,
    #[serde(rename = "candidatesTokenCount")]
    pub completion_tokens: u32,
    #[serde(rename = "totalTokenCount")]
    pub total_tokens: u32,
}

pub struct GeminiClient {
    config: GeminiConfig,
    http_client: reqwest::Client,
}

impl GeminiClient {
    pub fn new(config: GeminiConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("reqwest client must build");
        Self { config, http_client }
    }

    pub async fn generate(
        &self,
        prompt: String,
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<(String, u32), GeminiError> {
        let api_key = std::env::var(&self.config.api_key_env)
            .map_err(|_| GeminiError::HttpError("GEMINI_API_KEY not set".to_string()))?;

        let request = GeminiRequest {
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part { text: prompt }],
            }],
            generation_config: Some(GenerationConfig {
                max_output_tokens: max_tokens.unwrap_or(self.config.max_output_tokens),
                temperature: temperature.unwrap_or(self.config.temperature),
            }),
        };

        let url = format!("{}/?key={}", self.config.base_url, api_key);
        let resp = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| GeminiError::HttpError(e.to_string()))?;

        let status = resp.status().as_u16();
        let body = resp.text().await.map_err(|e| GeminiError::HttpError(e.to_string()))?;

        if status != 200 {
            return Err(GeminiError::ApiError(format!("status={}: {}", status, body)));
        }

        let response: GeminiResponse = serde_json::from_str(&body)
            .map_err(|e| GeminiError::ParseError(e.to_string()))?;

        let text = response.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_default();

        let total_tokens = response.usage_metadata
            .map(|u| u.total_tokens)
            .unwrap_or(0);

        Ok((text, total_tokens))
    }
}
