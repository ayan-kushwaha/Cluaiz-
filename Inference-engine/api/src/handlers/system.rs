use axum::{
    extract::State,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::state::AppState;

// ─── Root ────────────────────────────────────────────────────────────
pub async fn root() -> Json<Value> {
    Json(json!({
        "engine": "CURE — Cluaiz Universal Runtime Engine",
        "version": env!("CARGO_PKG_VERSION"),
        "gateway": "http://localhost:8000",
        "endpoints": {
            "GET  /":                 "This welcome message",
            "GET  /health":           "Engine health check",
            "GET  /info":             "System information & pillars",
            "POST /chat":             "Send message → get AI response",
            "GET  /history/:session": "Chat history for a session",
            "GET  /sessions":         "List all chat sessions",
            "GET  /status/sidecars":  "Database sidecar status",
            "GET  /hardware":         "Detect system RAM/CPU to suggest models",
            "POST /models/download":  "Download .gguf from Hugging Face",
            "POST /models/load":      "Load a downloaded .gguf file",
            "POST /engine/skip_think":"Skip thinking during generation"
        },
        "philosophy": "Nothing Need. Just CURE."
    }))
}

// ─── Health Check ────────────────────────────────────────────────────
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "alive",
        "engine": "CURE — Cluaiz Universal Runtime Engine",
        "version": env!("CARGO_PKG_VERSION"),
        "message": "🚀 CURE is alive! All systems operational."
    }))
}

// ─── System Info ─────────────────────────────────────────────────────
pub async fn system_info() -> Json<Value> {
    Json(json!({
        "engine": "CURE",
        "full_name": "Cluaiz Universal Runtime Engine",
        "version": env!("CARGO_PKG_VERSION"),
        "pillars": {
            "api": "Gateway — HTTP server on port 8000 (this!)",
            "kernel": "Brain — Decision-making & orchestration",
            "storage": "Sidecars — 5 Official DB engines (Mongo, Neo4j, ClickHouse, Qdrant, MinIO)",
            "engines": "Muscles — C++ model inference via llama.cpp FFI"
        },
        "philosophy": "Nothing Need. Just CURE.",
        "banned": ["Python", "Docker", "npm", "pip"]
    }))
}

// ─── Skip Thinking ───────────────────────────────────────────────────
pub async fn skip_think() -> Json<Value> {
    cluaiz_shared::GLOBAL_SKIP_THINKING_SIGNAL.store(true, std::sync::atomic::Ordering::SeqCst);
    Json(json!({
        "status": "success",
        "message": "Brain skip signal injected. Neural graph will pivot."
    }))
}

// ─── GET /v1/system/control ───────────────────────────────────────────
pub async fn get_system_control(State(_state): State<Arc<AppState>>) -> Json<Value> {
    use cluaiz_shared::hardware::governor::HardwareGovernor;
    if let Ok(control) = HardwareGovernor::load_system_control() {
        Json(json!({
            "status": "success",
            "control": control
        }))
    } else {
        Json(json!({
            "status": "error",
            "message": "Failed to load system control config"
        }))
    }
}

// ─── POST /v1/system/brain ────────────────────────────────────────────
#[derive(serde::Deserialize)]
pub struct BrainControlPayload {
    pub state: String,
}

pub async fn toggle_brain(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<BrainControlPayload>,
) -> Json<Value> {
    use cluaiz_shared::hardware::governor::HardwareGovernor;
    use cluaiz_shared::hardware::system_control::HardwareOrchestrator;
    
    if let Ok(mut control) = HardwareGovernor::load_system_control() {
        control.brain.cluaizd_connect_ffi = payload.state.clone();
        if let Err(e) = HardwareOrchestrator::persist_sovereign_state(&control) {
            return Json(json!({
                "status": "error",
                "message": format!("Failed to save system control: {}", e)
            }));
        } else {
            return Json(json!({
                "status": "success",
                "message": format!("Cluaizd FFI Connection toggled to: {}", payload.state)
            }));
        }
    }
    
    Json(json!({
        "status": "error",
        "message": "Failed to load system control config"
    }))
}
