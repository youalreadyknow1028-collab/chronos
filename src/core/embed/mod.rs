//! Embedding Service — Phase B: Candle + HuggingFace Hub
//!
//! Uses candle-transformers' BERT implementation to run all-MiniLM-L6-v2
//! locally on CPU. Model weights and tokenizer are downloaded from HuggingFace
//! Hub on first use (~90MB, cached by hf-hub).
//!
//! Output: 384-dimensional L2-normalized embeddings (cosine similarity ready).

use candle_core::{DType, Device, Result as CR, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{Config as BertConfig, HiddenAct, BertModel};
use hf_hub::{api::sync::Api, Repo, RepoType};
use std::path::PathBuf;
use std::sync::OnceLock;
use tokenizers::Tokenizer;
use tracing::info;

/// Embedding dimension for all-MiniLM-L6-v2
pub const EMBEDDING_DIM: usize = 384;

/// Model identifier on HuggingFace Hub
const MODEL_NAME: &str = "sentence-transformers/all-MiniLM-L6-v2";

/// Cached paths (set after first init)
static MODEL_PATH: OnceLock<PathBuf> = OnceLock::new();
static TOKENIZER_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Represents a generated embedding vector
#[derive(Debug, Clone)]
pub struct Embedding {
    /// 384-dimensional vector
    pub vector: Vec<f32>,
    pub model: String,
}

impl Embedding {
    /// Return a copy of the vector (already L2-normalized)
    pub fn normalized(&self) -> Vec<f32> {
        self.vector.clone()
    }
}

/// Initialize the embedder — downloads model + tokenizer from HuggingFace Hub if not cached.
/// This runs once. Subsequent calls are no-ops.
pub fn init_embedder() -> Result<(), EmbedError> {
    if MODEL_PATH.get().is_some() && TOKENIZER_PATH.get().is_some() {
        return Ok(());
    }

    info!(
        "[Embed] Downloading all-MiniLM-L6-v2 from HuggingFace Hub (~90MB)..."
    );
    info!("[Embed] This runs once — subsequent launches use the local cache.");

    let api = Api::new().map_err(|e| EmbedError::InitFailed(e.to_string()))?;
    let repo = Repo::new(MODEL_NAME.to_string(), RepoType::Model);
    let repo = api.repo(repo);

    let model_file: PathBuf = repo
        .get("model.safetensors")
        .map_err(|e| EmbedError::InitFailed(format!("Failed to download model: {}", e)))?;
    let tokenizer_file: PathBuf = repo
        .get("tokenizer.json")
        .map_err(|e| EmbedError::InitFailed(format!("Failed to download tokenizer: {}", e)))?;

    let _ = MODEL_PATH.set(model_file);
    let _ = TOKENIZER_PATH.set(tokenizer_file);

    info!("[Embed] Model and tokenizer cached successfully.");
    Ok(())
}

/// Generate a 384-dim embedding for the given text using all-MiniLM-L6-v2.
/// Requires init_embedder() to have been called first.
pub fn embed_text(text: &str) -> Result<Embedding, EmbedError> {
    let model_path = MODEL_PATH
        .get()
        .ok_or_else(|| EmbedError::NotInitialized("call init_embedder() first".into()))?;
    let tokenizer_path = TOKENIZER_PATH
        .get()
        .ok_or_else(|| EmbedError::NotInitialized("call init_embedder() first".into()))?;

    let device = Device::Cpu;

    // Tokenize the input text
    let tokenizer = Tokenizer::from_file(tokenizer_path)
        .map_err(|e| EmbedError::EmbeddingFailed(format!("Failed to load tokenizer: {}", e)))?;
    let encoding = tokenizer
        .encode(text, true)
        .map_err(|e| EmbedError::EmbeddingFailed(format!("Tokenization failed: {}", e)))?;

    let token_ids: Vec<u32> = encoding.get_ids().to_vec();
    let token_ids_tensor = Tensor::new(&token_ids[..], &device)?.unsqueeze(0)?;
    let token_type_ids: Vec<u32> = vec![0u32; token_ids.len()];
    let token_type_tensor = Tensor::new(&token_type_ids[..], &device)?.unsqueeze(0)?;

    // Build BERT config for all-MiniLM-L6-v2
    let config = BertConfig {
        vocab_size: 30522,
        hidden_size: 384,
        num_hidden_layers: 6,
        num_attention_heads: 6,
        intermediate_size: 1536,
        hidden_act: HiddenAct::GeluApproximate, // GELU approximation
        hidden_dropout_prob: 0.1,
        max_position_embeddings: 512,
        type_vocab_size: 2,
        initializer_range: 0.02,
        layer_norm_eps: 1e-12,
        pad_token_id: 0,
        position_embedding_type:
            candle_transformers::models::bert::PositionEmbeddingType::Absolute,
        classifier_dropout: None,
        model_type: Some("bert".to_string()),
        use_cache: true,
    };

    // Load BERT model weights from safetensors
    let vb = unsafe {
        VarBuilder::from_mmaped_safetensors(&[model_path.clone()], DType::F32, &device)
            .map_err(|e| EmbedError::EmbeddingFailed(format!("Failed to load safetensors: {}", e)))?
    };

    let model = BertModel::load(vb, &config)
        .map_err(|e| EmbedError::EmbeddingFailed(format!("Failed to build model: {}", e)))?;

    // Forward pass: [1, seq_len, 384]
    let sequence_output = model
        .forward(&token_ids_tensor, &token_type_tensor, None)
        .map_err(|e| EmbedError::EmbeddingFailed(format!("Forward failed: {}", e)))?;

    // Mean pooling over sequence dimension
    let (_batch, seq_len, _hidden) = sequence_output
        .dims3()
        .map_err(|e| EmbedError::EmbeddingFailed(format!("Unexpected output shape: {}", e)))?;

    let pooled: Tensor = sequence_output
        .sum(1)?
        .affine(1.0 / seq_len as f64, 0.0)?;

    // L2 normalize for cosine similarity
    let normalized = normalize_l2(&pooled)?;

    let embedding_vec: Vec<f32> = normalized
        .to_vec1()
        .map_err(|e| EmbedError::EmbeddingFailed(format!("Failed to extract vector: {}", e)))?;

    Ok(Embedding {
        vector: embedding_vec,
        model: MODEL_NAME.to_string(),
    })
}

/// Generate embeddings for a batch of texts
pub fn embed_texts(texts: &[&str]) -> Result<Vec<Embedding>, EmbedError> {
    texts.iter().map(|t| embed_text(t)).collect()
}

/// L2 normalize a tensor
fn normalize_l2(v: &Tensor) -> CR<Tensor> {
    let sq_sum = v.sqr()?;
    let norm = sq_sum.sqrt()?;
    v.broadcast_div(&norm)
}

/// Check if the embedder is ready
pub fn is_ready() -> bool {
    MODEL_PATH.get().is_some() && TOKENIZER_PATH.get().is_some()
}

impl From<candle_core::Error> for EmbedError {
    fn from(e: candle_core::Error) -> Self {
        EmbedError::EmbeddingFailed(e.to_string())
    }
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
            EmbedError::EmbeddingFailed(msg) => write!(f, "Embedding failed: {}", msg),
        }
    }
}

impl std::error::Error for EmbedError {}
