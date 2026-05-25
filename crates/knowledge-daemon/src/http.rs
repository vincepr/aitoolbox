use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use camino::Utf8PathBuf;
use http::StatusCode;
use knowledge_core::notes::NoteStore;
use knowledge_core::schema::verify_schema;
use knowledge_core::store::KnowledgeStore;
use rusqlite::Connection;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

/// Shared daemon state injected via Axum's typed state layer.
#[derive(Debug, Clone)]
pub struct AppState {
    pub db_path: Utf8PathBuf,
    pub notes_root: Utf8PathBuf,
}

/// JSON response for health checks.
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

/// JSON response mirroring knowledge-cli get semantics.
#[derive(Debug, Clone, Serialize)]
pub struct EntityResponse {
    pub found: bool,
    pub canonical_name: String,
    pub summary: String,
}

/// Builds the daemon router with required middleware and typed state.
pub fn router(state: AppState) -> Router {
    let shared = Arc::new(state);

    Router::new()
        .route("/health", get(health))
        .route("/entity/:name", get(get_entity))
        .with_state(shared)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(5)))
}

async fn health() -> (StatusCode, Json<HealthResponse>) {
    (StatusCode::OK, Json(HealthResponse { status: "ok" }))
}

async fn get_entity(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> (StatusCode, Json<EntityResponse>) {
    match query_entity(state.as_ref(), &name) {
        Ok(Some(answer)) => (
            StatusCode::OK,
            Json(EntityResponse {
                found: true,
                canonical_name: answer.canonical_name,
                summary: answer.summary,
            }),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(EntityResponse {
                found: false,
                canonical_name: String::new(),
                summary: String::new(),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(EntityResponse {
                found: false,
                canonical_name: String::new(),
                summary: String::new(),
            }),
        ),
    }
}

fn query_entity(state: &AppState, name: &str) -> anyhow::Result<Option<knowledge_core::store::QueryAnswer>> {
    let conn = Connection::open(state.db_path.as_std_path())?;
    verify_schema(&conn)?;
    let notes = NoteStore::new(state.notes_root.clone());
    let store = KnowledgeStore::new(&conn);
    let answer = store.query_exact(name, &notes)?;
    Ok(answer)
}
