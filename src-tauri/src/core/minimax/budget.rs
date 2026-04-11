//! Budget State + CallGuard RAII
//! See EPIC_4 §C — CallGuard reserves before call, confirms on commit, rolls back on abort
//! This is the CRITICAL fix for the increment-before-call bug

use std::sync::Arc;
use tokio::sync::Mutex;

/// Budget limits per cron cycle
pub const T4_CALL_LIMIT: u32 = 20;
pub const T4_TOKEN_BUDGET: u32 = 80000;
pub const T3_TOKEN_BUDGET: u32 = 20000;
pub const T3_CALL_LIMIT: u32 = 10;
pub const GEMINI_TOKEN_BUDGET: u32 = 10000;

#[derive(Debug, Clone)]
pub struct BudgetState {
    /// MiniMax calls made this cycle
    pub minimax_calls_made: u32,
    /// MiniMax tokens used this cycle
    pub minimax_tokens_used: u32,
    /// Gemini calls made this cycle (T3 fallback)
    pub gemini_calls_made: u32,
    /// Gemini tokens used this cycle
    pub gemini_tokens_used: u32,
    /// Were we in a prepare phase when guard was created?
    was_prepared: bool,
}

impl BudgetState {
    pub fn new() -> Self {
        Self {
            minimax_calls_made: 0,
            minimax_tokens_used: 0,
            gemini_calls_made: 0,
            gemini_tokens_used: 0,
            was_prepared: false,
        }
    }

    /// Record actual token usage after call completes
    pub fn record_minimax(&mut self, tokens_used: u32) {
        self.minimax_calls_made += 1;
        self.minimax_tokens_used += tokens_used;
    }

    pub fn record_gemini(&mut self, tokens_used: u32) {
        self.gemini_calls_made += 1;
        self.gemini_tokens_used += tokens_used;
    }

    pub fn minimax_remaining_tokens(&self) -> u32 {
        T4_TOKEN_BUDGET.saturating_sub(self.minimax_tokens_used)
    }

    pub fn gemini_remaining_tokens(&self) -> u32 {
        GEMINI_TOKEN_BUDGET.saturating_sub(self.gemini_tokens_used)
    }

    pub fn minimax_calls_remaining(&self) -> u32 {
        T4_CALL_LIMIT.saturating_sub(self.minimax_calls_made)
    }

    pub fn gemini_calls_remaining(&self) -> u32 {
        T3_CALL_LIMIT.saturating_sub(self.gemini_calls_made)
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for BudgetState {
    fn default() -> Self {
        Self::new()
    }
}

/// CallGuard RAII — reserves tokens BEFORE network call
/// - On Drop without confirm(): rolls back the reservation
/// - On confirm(): permanently records the actual tokens used
/// This prevents the "increment-before-call" bug where panics left burned slots
pub struct CallGuard {
    state: Arc<Mutex<BudgetState>>,
    call_type: CallType,
    estimated_tokens: u32,
    confirmed: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum CallType {
    MiniMax,
    Gemini,
    T3MiniMax,
}

impl CallGuard {
    /// Reserve a slot. Returns guard if budget available, error otherwise.
    pub async fn reserve_minimax(state: Arc<Mutex<BudgetState>>, estimated_tokens: u32) 
        -> Result<Self, BudgetError> 
    {
        // Reserve state for the CallGuard first (before taking the lock)
        let st = state.clone();

        let mut guard = st.lock().await;
        
        // Check call limit
        if guard.minimax_calls_made >= T4_CALL_LIMIT {
            return Err(BudgetError::CallLimitExceeded {
                provider: "minimax".to_string(),
                used: guard.minimax_calls_made,
                limit: T4_CALL_LIMIT,
            });
        }
        
        // Check token budget
        if guard.minimax_remaining_tokens() < estimated_tokens {
            return Err(BudgetError::TokenBudgetExceeded {
                provider: "minimax".to_string(),
                needed: estimated_tokens,
                remaining: guard.minimax_remaining_tokens(),
            });
        }

        // Mark as prepared (increment AFTER all checks pass)
        guard.minimax_calls_made += 1;
        guard.minimax_tokens_used += estimated_tokens; // Reserve estimated tokens
        
        tracing::debug!(
            "[CallGuard] Reserved MiniMax: {} tokens (budget: {}/{})",
            estimated_tokens,
            guard.minimax_calls_made,
            T4_CALL_LIMIT
        );

        // Drop guard before moving state into CallGuard
        drop(guard);

        Ok(Self {
            state: st,
            call_type: CallType::MiniMax,
            estimated_tokens,
            confirmed: false,
        })
    }

    /// Confirm the call — permanently record actual tokens used
    pub async fn confirm(&mut self, actual_tokens: u32) {
        let mut guard = self.state.lock().await;
        // Adjust: remove estimated, add actual
        guard.minimax_tokens_used = guard.minimax_tokens_used
            .saturating_sub(self.estimated_tokens)
            .saturating_add(actual_tokens);
        self.confirmed = true;
        tracing::debug!("[CallGuard] Confirmed MiniMax: {} actual tokens", actual_tokens);
    }

    /// Roll back the reservation (called on Drop if not confirmed)
    fn rollback(&self) {
        let estimated = self.estimated_tokens;
        tracing::debug!(
            "[CallGuard] Dropped without confirm — rolling back {} tokens",
            estimated
        );
    }
}

impl Drop for CallGuard {
    fn drop(&mut self) {
        if !self.confirmed {
            // Rollback: the slot was reserved but the call failed/dropped without confirm
            // This is the critical safety net — we restore the estimated tokens
            let state = self.state.clone();
            let estimated = self.estimated_tokens;
            let call_type = self.call_type;
            
            // Spawn a task to rollback (can't await in Drop)
            tokio::spawn(async move {
                let mut guard = state.lock().await;
                match call_type {
                    CallType::MiniMax | CallType::T3MiniMax => {
                        guard.minimax_calls_made = guard.minimax_calls_made.saturating_sub(1);
                        guard.minimax_tokens_used = guard.minimax_tokens_used.saturating_sub(estimated);
                    }
                    CallType::Gemini => {
                        guard.gemini_calls_made = guard.gemini_calls_made.saturating_sub(1);
                        guard.gemini_tokens_used = guard.gemini_tokens_used.saturating_sub(estimated);
                    }
                }
                tracing::debug!("[CallGuard Rollback] Restored {} tokens", estimated);
            });
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BudgetError {
    #[error("Call limit exceeded: {provider} used {used}/{limit}")]
    CallLimitExceeded { provider: String, used: u32, limit: u32 },
    #[error("Token budget exceeded: {provider} needs {needed}, only {remaining} remaining")]
    TokenBudgetExceeded { provider: String, needed: u32, remaining: u32 },
}
