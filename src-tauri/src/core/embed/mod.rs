//! Embedding Service — Phase A STUB
//! 
//! PHASE A STATUS: fastembed BLOCKED — corenn-kernels crate requires nightly Rust.
//! Pipeline generates 384-dim placeholder vectors (XXHash64-based) for structural testing.
//! Phase B will implement actual embeddings via candle + local ONNX model or equivalent.
//!
//! See: https://github.com/wilsonzlin/corenn — the corenn project requires #![feature(f16)]
//! which is nightly-only. Affects: fastembed (ort → ort-sys → corenn-kernels) AND hnswlib-rs.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tracing::warn;

/// Embedding dimension for all-MiniLM-L6-v2 (target model)
pub const EMBEDDING_DIM: usize = 384;

/// Represents a generated embedding vector
#[derive(Debug, Clone)]
pub struct Embedding {
    pub vector: Vec<f32>,
    pub model: String,
}

impl Embedding {
    /// Return a normalized copy of the vector (for cosine similarity)
    pub fn normalized(&self) -> Vec<f32> {
        let norm = self.vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm == 0.0 {
            return self.vector.clone();
        }
        self.vector.iter().map(|v| v / norm).collect()
    }
}

/// Initialize the embedder (stub — no-op in Phase A)
pub fn init_embedder() -> Result<(), EmbedError> {
    warn!("[Embed] Phase A: fastembed blocked (corenn-kernels/nightly). Using stub vectors.");
    Ok(())
}

/// Generate a stub 384-dim embedding using XXHash64 determinism.
///
/// NOT semantic similarity — this is for structural pipeline testing only.
/// Phase B replaces this with actual fastembed (all-MiniLM-L6-v2) embeddings.
pub fn embed_text(text: &str) -> Result<Embedding, EmbedError> {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let seed = hasher.finish();
    
    // Deterministic "pseudo-embedding" from hash seed — 384-dim
    // Spread the hash seed across all dimensions using a simple PRNG (xorshift)
    let mut rng_state = seed;
    let mut vector = Vec::with_capacity(EMBEDDING_DIM);
    for _ in 0..EMBEDDING_DIM {
        // xorshift64 — fast, deterministic, no external dep
        rng_state ^= rng_state << 13;
        rng_state ^= rng_state >> 7;
        rng_state ^= rng_state << 17;
        let val = (rng_state as f32) / (u32::MAX as f32) * 2.0 - 1.0; // [-1, 1]
        vector.push(val);
    }

    // Normalize so this behaves like a real embedding for cosine sim
    let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut vector {
            *v /= norm;
        }
    }

    Ok(Embedding {
        vector,
        model: "STUB-xorshift64".to_string(),
    })
}

/// Generate embeddings for a batch of texts
pub fn embed_texts(texts: &[&str]) -> Result<Vec<Embedding>, EmbedError> {
    texts.iter().map(|t| embed_text(t)).collect()
}

/// Check if the embedder is ready
pub fn is_ready() -> bool {
    true // Stub always ready
}

#[derive(Debug, Clone)]
pub enum EmbedError {
    InitFailed(String),
    NotInitialized(String),
    EmbeddingFailed(String),
}

impl std::fmt::Display for EmbedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbedError::InitFailed(msg) => write!(f, "Embedder init failed: {}", msg),
            EmbedError::NotInitialized(msg) => write!(f, "Embedder not initialized: {}", msg),
            EmbedError::EmbeddingFailed(msg) => write!(f, "Embedding generation failed: {}", msg),
        }
    }
}

impl std::error::Error for EmbedError {}
