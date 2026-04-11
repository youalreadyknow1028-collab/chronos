pub mod ipc;
pub mod kuzu;
pub mod qdrant;
pub mod minimax;
pub mod pipeline;
pub mod encryption;
pub mod embed;   // Phase B: candle + hf-hub embedder (all-MiniLM-L6-v2, 384-dim)
pub mod lance;   // Phase B: LanceDB vector store (384-dim)
pub mod layout;
pub mod scheduler;
pub mod system;
pub mod sync;
