use axum::{
    extract::{Path, State},
    response::Json,
};
use kernel::{ChatRequest, ChatResponse, ChatSession};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::state::AppState;

// ─── POST /chat — The Main Event ─────────────────────────────────────
pub async fn chat(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Json<ChatResponse> {
    let response = state.kernel.process_chat(&request);
    Json(response)
}

// ─── GET /history/:session_id ────────────────────────────────────────
pub async fn get_history(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Json<Value> {
    match state.kernel.get_history(&session_id) {
        Some(session) => Json(json!({
            "success": true,
            "session": session
        })),
        None => Json(json!({
            "success": false,
            "error": format!("Session '{}' not found", session_id)
        })),
    }
}

// ─── GET /sessions ───────────────────────────────────────────────────
pub async fn get_sessions(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions: Vec<ChatSession> = state.kernel.get_all_sessions();
    Json(json!({
        "success": true,
        "count": sessions.len(),
        "sessions": sessions
    }))
}
