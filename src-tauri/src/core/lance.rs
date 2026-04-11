//! LanceDB Vector Store — Phase B Wiring
//!
//! Stores 384-dim embeddings + metadata in a local LanceDB directory.
//! Schema: id, content, source, tags(JSON), vector(384-dim), created_at.

use arrow_array::{
    types::Float32Type,
    FixedSizeListArray, Int64Array, RecordBatch, StringArray,
};
use arrow_schema::DataType;
use futures::TryStreamExt;
use lancedb::{
    connection::Connection,
    connect,
    query::{ExecutableQuery, QueryBase},
};
use std::sync::OnceLock;
use tracing::info;

const LANCE_DB_DIR: &str = ".chronos/vectordb";
const TABLE_NAME: &str = "embeddings";

static LANCE_CONN: OnceLock<Connection> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct EmbedRecord {
    pub id: String,
    pub content: String,
    pub source: String,
    pub tags: Vec<String>,
    pub vector: Vec<f32>,
    pub created_at: i64,
}

pub fn init_lance() -> Result<(), LanceError> {
    if LANCE_CONN.get().is_some() {
        return Ok(());
    }

    std::fs::create_dir_all(LANCE_DB_DIR)
        .map_err(|e| LanceError::InitFailed(format!("create dir failed: {}", e)))?;

    let rt = tokio::runtime::Handle::current();

    // Connect: connect() → ConnectBuilder → .execute() → Connection
    let conn: Connection = rt
        .block_on(connect(LANCE_DB_DIR).execute())
        .map_err(|e| LanceError::InitFailed(format!("connect() failed: {}", e)))?;

    let _ = LANCE_CONN.set(conn);

    // Create table if not exists
    create_table_if_not_exists()?;

    info!("[LanceDB] Initialized at {}", LANCE_DB_DIR);
    Ok(())
}

fn create_table_if_not_exists() -> Result<(), LanceError> {
    let conn = LANCE_CONN
        .get()
        .ok_or_else(|| LanceError::NotInitialized("not initialized".into()))?;

    let rt = tokio::runtime::Handle::current();

    // Check existing tables
    let table_names = rt
        .block_on(conn.table_names().execute())
        .map_err(|e| LanceError::InitFailed(format!("table_names failed: {}", e)))?;
    let existing: Vec<String> = table_names.into_iter().collect();

    if existing.iter().any(|n| n == TABLE_NAME) {
        info!("[LanceDB] Table '{}' already exists", TABLE_NAME);
        return Ok(());
    }

    info!("[LanceDB] Creating table '{}'", TABLE_NAME);

    // Build Arrow schema
    let schema = std::sync::Arc::new(arrow_schema::Schema::new(vec![
        arrow_schema::Field::new("id", DataType::Utf8, false),
        arrow_schema::Field::new("content", DataType::Utf8, false),
        arrow_schema::Field::new("source", DataType::Utf8, false),
        arrow_schema::Field::new("tags", DataType::Utf8, false),
        arrow_schema::Field::new(
            "vector",
            DataType::FixedSizeList(
                std::sync::Arc::new(arrow_schema::Field::new("item", DataType::Float32, true)),
                384,
            ),
            false,
        ),
        arrow_schema::Field::new("created_at", DataType::Int64, false),
    ]));

    // Create empty table
    let _table = rt
        .block_on(conn.create_empty_table(TABLE_NAME, schema).execute())
        .map_err(|e| LanceError::InitFailed(format!("create_empty_table failed: {}", e)))?;

    // Add ANN index on vector column
    use lancedb::index::Index;
    rt.block_on(_table.create_index(&["vector"], Index::Auto).execute())
        .map_err(|e| LanceError::InitFailed(format!("create_index failed: {}", e)))?;

    info!("[LanceDB] Table '{}' created with ANN index on 'vector'", TABLE_NAME);
    Ok(())
}

pub fn insert_record(record: EmbedRecord) -> Result<(), LanceError> {
    let conn = LANCE_CONN
        .get()
        .ok_or_else(|| LanceError::NotInitialized("call init_lance() first".into()))?;

    let rt = tokio::runtime::Handle::current();

    // Open table: OpenTableBuilder → .execute() → Table
    let table = rt
        .block_on(conn.open_table(TABLE_NAME).execute())
        .map_err(|e| LanceError::InsertFailed(format!("open_table failed: {}", e)))?;

    let tags_json = serde_json::to_string(&record.tags).unwrap_or_else(|_| "[]".into());

    let batch = build_batch(
        vec![record.id],
        vec![record.content],
        vec![record.source],
        vec![tags_json],
        vec![record.vector],
        vec![record.created_at],
    )?;

    rt.block_on(table.add(batch).execute())
        .map_err(|e| LanceError::InsertFailed(format!("add failed: {}", e)))?;

    Ok(())
}

fn build_batch(
    ids: Vec<String>,
    contents: Vec<String>,
    sources: Vec<String>,
    tags: Vec<String>,
    vectors: Vec<Vec<f32>>,
    created_ats: Vec<i64>,
) -> Result<RecordBatch, LanceError> {
    let schema = arrow_schema::Schema::new(vec![
        arrow_schema::Field::new("id", DataType::Utf8, false),
        arrow_schema::Field::new("content", DataType::Utf8, false),
        arrow_schema::Field::new("source", DataType::Utf8, false),
        arrow_schema::Field::new("tags", DataType::Utf8, false),
        arrow_schema::Field::new(
            "vector",
            DataType::FixedSizeList(
                std::sync::Arc::new(arrow_schema::Field::new("item", DataType::Float32, true)),
                384,
            ),
            false,
        ),
        arrow_schema::Field::new("created_at", DataType::Int64, false),
    ]);

    // FixedSizeListArray: inner type must be Option<f32>
    // Each item: Option<Vec<Option<f32>>> — outer=is_row_null, inner=per-element nullability
    let vector_array = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
        vectors.into_iter().map(|vec| {
            Some(vec.into_iter().map(Some).collect::<Vec<_>>())
        }),
        384,
    );

    RecordBatch::try_new(
        std::sync::Arc::new(schema),
        vec![
            std::sync::Arc::new(StringArray::from(ids)),
            std::sync::Arc::new(StringArray::from(contents)),
            std::sync::Arc::new(StringArray::from(sources)),
            std::sync::Arc::new(StringArray::from(tags)),
            std::sync::Arc::new(vector_array),
            std::sync::Arc::new(Int64Array::from(created_ats)),
        ],
    )
    .map_err(|e| LanceError::InsertFailed(format!("RecordBatch failed: {}", e)))
}

pub fn search_similar(vector: &[f32], limit: usize) -> Result<Vec<SearchHit>, LanceError> {
    let conn = LANCE_CONN
        .get()
        .ok_or_else(|| LanceError::NotInitialized("call init_lance() first".into()))?;

    let rt = tokio::runtime::Handle::current();

    rt.block_on(async {
        let table = conn.open_table(TABLE_NAME).execute().await
            .map_err(|e| LanceError::QueryFailed(format!("open_table failed: {}", e)))?;

        let query_result = table.query()
            .nearest_to(vector)
            .map_err(|e| LanceError::QueryFailed(format!("nearest_to failed: {}", e)))?
            .limit(limit)
            .execute().await
            .map_err(|e| LanceError::QueryFailed(format!("execute failed: {}", e)))?;

        let batches: Vec<RecordBatch> = query_result
            .try_collect().await
            .map_err(|e| LanceError::QueryFailed(format!("collect failed: {}", e)))?;

        let mut hits = Vec::new();
        for batch in &batches {
            let arr_id = batch.column(0).as_any().downcast_ref::<StringArray>();
            let arr_content = batch.column(1).as_any().downcast_ref::<StringArray>();
            let arr_source = batch.column(2).as_any().downcast_ref::<StringArray>();
            let n_rows = batch.num_rows();
            for i in 0..n_rows {
                let id = arr_id.map(|arr| arr.value(i).to_string()).unwrap_or_default();
                let content = arr_content.map(|arr| arr.value(i).to_string()).unwrap_or_default();
                let source = arr_source.map(|arr| arr.value(i).to_string()).unwrap_or_default();
                let score = if batch.num_columns() > 3 {
                    batch.column(batch.num_columns() - 1)
                        .as_any()
                        .downcast_ref::<arrow_array::Float32Array>()
                        .map(|arr| arr.value(i))
                        .unwrap_or(0.0_f32)
                } else {
                    0.0_f32
                };
                hits.push(SearchHit { id, content, source, score });
            }
        }

        Ok::<Vec<SearchHit>, LanceError>(hits)
    })
}

pub fn is_ready() -> bool {
    LANCE_CONN.get().is_some()
}

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub id: String,
    pub content: String,
    pub source: String,
    pub score: f32,
}

#[derive(Debug, Clone)]
pub enum LanceError {
    InitFailed(String),
    NotInitialized(String),
    InsertFailed(String),
    QueryFailed(String),
}

impl std::fmt::Display for LanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LanceError::InitFailed(msg) => write!(f, "LanceDB init failed: {}", msg),
            LanceError::NotInitialized(msg) => write!(f, "LanceDB not initialized: {}", msg),
            LanceError::InsertFailed(msg) => write!(f, "LanceDB insert failed: {}", msg),
            LanceError::QueryFailed(msg) => write!(f, "LanceDB query failed: {}", msg),
        }
    }
}

impl std::error::Error for LanceError {}
