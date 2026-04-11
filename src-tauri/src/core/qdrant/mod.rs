pub mod client;
pub mod collections;

pub use client::QdrantVectorClient;
pub use collections::{VECTOR_SIZE, SHARD_NUMBER, indexed_payload_fields};
