//! Tier Router — Escalation Logic
//! See EPIC_4 §R — T1 → T2 → T3 → T4 → JUDGE → PERSISTED
//! T3 retries with Gemini on MiniMax failure

use super::budget::{BudgetState, CallGuard};
use super::client::{MiniMaxClient, Message};
use super::gemini::GeminiClient;
use super::config::{t3_minimax_config, t3_gemini_config};
use crate::core::minimax::budget::{BudgetError, T4_CALL_LIMIT, T4_TOKEN_BUDGET};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::warn;

#[derive(Debug, Clone, PartialEq)]
pub enum TierState {
    Pending,
    T1Done,
    T2Done,
    T3Done,
    T4Done,
    JudgeDone,
    Persisted,
}

impl TierState {
    pub fn next(&self) -> Self {
        match self {
            TierState::Pending => TierState::T1Done,
            TierState::T1Done => TierState::T2Done,
            TierState::T2Done => TierState::T3Done,
            TierState::T3Done => TierState::T4Done,
            TierState::T4Done => TierState::JudgeDone,
            TierState::JudgeDone => TierState::Persisted,
            TierState::Persisted => TierState::Persisted,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TierState::Pending => "PENDING",
            TierState::T1Done => "T1_DONE",
            TierState::T2Done => "T2_DONE",
            TierState::T3Done => "T3_DONE",
            TierState::T4Done => "T4_DONE",
            TierState::JudgeDone => "JUDGE_DONE",
            TierState::Persisted => "PERSISTED",
        }
    }
}

pub struct TierRouter {
    minimax: Option<MiniMaxClient>,
    gemini: Option<GeminiClient>,
    budget: Arc<Mutex<BudgetState>>,
}

impl TierRouter {
    pub fn new() -> Self {
        let minimax = match MiniMaxClient::with_default() {
            Ok(c) => {
                tracing::info!("[Router] MiniMax client initialized.");
                Some(c)
            }
            Err(e) => {
                tracing::warn!("[Router] MiniMax client failed to initialize: {}. API calls will fail.", e);
                None
            }
        };

        let gemini = match GeminiClient::new(t3_gemini_config()) {
            Ok(c) => {
                tracing::info!("[Router] Gemini client initialized.");
                Some(c)
            }
            Err(e) => {
                tracing::warn!("[Router] Gemini client failed to initialize: {}. Gemini fallback will be unavailable.", e);
                None
            }
        };

        Self {
            minimax,
            gemini,
            budget: Arc::new(Mutex::new(BudgetState::new())),
        }
    }

    /// Reset budget at the start of each cron cycle
    pub async fn reset_cycle(&self) {
        self.budget.lock().await.reset();
    }

    /// T1: Quick classification + tagging
    pub async fn run_t1(&self, content: &str) -> Result<String, TierError> {
        let messages = vec![Message {
            role: "system".to_string(),
            content: "Classify this thought. Return JSON: {\"tags\": [...], \"type\": \"fact|belief|question|idea\"}".to_string(),
        }, Message {
            role: "user".to_string(),
            content: content.to_string(),
        }];

        let response = self.call_minimax(messages).await?;
        Ok(response)
    }

    /// T2: Extract claims and relationships
    pub async fn run_t2(&self, content: &str) -> Result<String, TierError> {
        let messages = vec![Message {
            role: "system".to_string(),
            content: "Extract claims and relationships. Return JSON: {\"claims\": [...], \"relationships\": [...]}".to_string(),
        }, Message {
            role: "user".to_string(),
            content: content.to_string(),
        }];

        let response = self.call_minimax(messages).await?;
        Ok(response)
    }

    /// T3: Generate synthesis — MiniMax primary, Gemini retry on failure
    pub async fn run_t3(&self, t1_result: &str, t2_result: &str) -> Result<String, TierError> {
        let messages = vec![Message {
            role: "system".to_string(),
            content: t3_minimax_config().model.clone(), // lightweight prompt
        }, Message {
            role: "user".to_string(),
            content: format!("T1: {}\n\nT2: {}\n\nGenerate a synthesis.", t1_result, t2_result),
        }];

        // Try MiniMax first
        match self.call_minimax_t3(messages.clone()).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                warn!("[T3] MiniMax failed: {}. Trying Gemini fallback.", e);
            }
        }

        // T3 Retry: Gemini
        let prompt = format!("T1: {}\n\nT2: {}\n\nGenerate a synthesis.", t1_result, t2_result);
        self.call_gemini(prompt).await
    }

    /// T4: Deep reasoning — full MiniMax
    pub async fn run_t4(&self, synthesis: &str) -> Result<String, TierError> {
        let messages = vec![Message {
            role: "system".to_string(),
            content: "You are a deep reasoning engine. Expand on this synthesis with critical analysis.".to_string(),
        }, Message {
            role: "user".to_string(),
            content: synthesis.to_string(),
        }];

        self.call_minimax_max(messages).await
    }

    /// Full pipeline: T1 → T2 → T3 → T4 → Judge
    pub async fn run_pipeline(&self, content: &str) -> Result<PipelineResult, TierError> {
        let t1 = self.run_t1(content).await?;
        let t2 = self.run_t2(content).await?;
        let t3 = self.run_t3(&t1, &t2).await?;
        let t4 = self.run_t4(&t3).await?;

        Ok(PipelineResult {
            t1_result: t1,
            t2_result: t2,
            t3_result: t3,
            t4_result: t4,
            tier_used: "t4".to_string(),
            provider_used: "minimax".to_string(),
        })
    }

    async fn call_minimax(&self, messages: Vec<Message>) -> Result<String, TierError> {
        let client = self.minimax.as_ref()
            .ok_or_else(|| TierError::ProviderError("MiniMax client not initialized (reqwest failed to build)".into()))?;
        let estimated = client.estimate_tokens(&messages);
        let mut guard = CallGuard::reserve_minimax(self.budget.clone(), estimated).await
            .map_err(|e| TierError::BudgetError(e))?;

        let response = client
            .chat(messages.clone(), None, None)
            .await
            .map_err(|e| TierError::ProviderError(e.to_string()))?;

        let actual_tokens = response.usage.total_tokens;
        guard.confirm(actual_tokens).await;

        let text = response.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(text)
    }

    async fn call_minimax_t3(&self, messages: Vec<Message>) -> Result<String, TierError> {
        // T3 uses lighter config
        let minimax_t3 = MiniMaxClient::new(t3_minimax_config())
            .map_err(|e| TierError::ProviderError(format!("MiniMax T3 client init failed: {}", e)))?;
        let estimated = minimax_t3.estimate_tokens(&messages);
        
        let mut guard = CallGuard::reserve_minimax(self.budget.clone(), estimated).await
            .map_err(|e| TierError::BudgetError(e))?;

        let response = minimax_t3
            .chat(messages, None, None)
            .await
            .map_err(|e| TierError::ProviderError(e.to_string()))?;

        let actual_tokens = response.usage.total_tokens;
        guard.confirm(actual_tokens).await;

        Ok(response.choices.first().map(|c| c.message.content.clone()).unwrap_or_default())
    }

    async fn call_minimax_max(&self, messages: Vec<Message>) -> Result<String, TierError> {
        let client = self.minimax.as_ref()
            .ok_or_else(|| TierError::ProviderError("MiniMax client not initialized (reqwest failed to build)".into()))?;
        let estimated = client.estimate_tokens(&messages);
        let mut guard = CallGuard::reserve_minimax(self.budget.clone(), estimated).await
            .map_err(|e| TierError::BudgetError(e))?;

        let response = client
            .chat(messages, Some(0.7), Some(8192))
            .await
            .map_err(|e| TierError::ProviderError(e.to_string()))?;

        let actual_tokens = response.usage.total_tokens;
        guard.confirm(actual_tokens).await;

        Ok(response.choices.first().map(|c| c.message.content.clone()).unwrap_or_default())
    }

    async fn call_gemini(&self, prompt: String) -> Result<String, TierError> {
        let client = self.gemini.as_ref()
            .ok_or_else(|| TierError::ProviderError("Gemini client not initialized (reqwest failed to build)".into()))?;
        let (text, tokens) = client
            .generate(prompt, Some(2048), Some(0.5))
            .await
            .map_err(|e| TierError::ProviderError(e.to_string()))?;

        // Record Gemini usage
        {
            let mut guard = self.budget.lock().await;
            guard.record_gemini(tokens);
        }

        Ok(text)
    }

    pub async fn budget_status(&self) -> BudgetStatus {
        let guard = self.budget.lock().await;
        BudgetStatus {
            minimax_calls: guard.minimax_calls_made,
            minimax_calls_limit: T4_CALL_LIMIT,
            minimax_tokens: guard.minimax_tokens_used,
            minimax_tokens_limit: T4_TOKEN_BUDGET,
            gemini_calls: guard.gemini_calls_made,
            gemini_tokens: guard.gemini_tokens_used,
            gemini_tokens_limit: 10000,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PipelineResult {
    pub t1_result: String,
    pub t2_result: String,
    pub t3_result: String,
    pub t4_result: String,
    pub tier_used: String,
    pub provider_used: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BudgetStatus {
    pub minimax_calls: u32,
    pub minimax_calls_limit: u32,
    pub minimax_tokens: u32,
    pub minimax_tokens_limit: u32,
    pub gemini_calls: u32,
    pub gemini_tokens: u32,
    pub gemini_tokens_limit: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum TierError {
    #[error("Budget error: {0}")]
    BudgetError(#[from] BudgetError),
    #[error("Provider error: {0}")]
    ProviderError(String),
}
