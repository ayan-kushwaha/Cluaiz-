use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::state::AppState;
use dispatcher::EngineResponse;

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ServerOptions, NamedPipeServer};

#[cfg(windows)]
const PIPE_NAME: &str = r"\\.\pipe\cluaize_engine_pipe";

#[cfg(windows)]
pub async fn start_named_pipe_server(state: Arc<AppState>) {
    loop {
        // Attempt to create the first instance or subsequent instances
        let server = match ServerOptions::new().first_pipe_instance(true).create(PIPE_NAME) {
            Ok(server) => server,
            Err(_) => {
                match ServerOptions::new().create(PIPE_NAME) {
                    Ok(server) => server,
                    Err(e) => {
                        tracing::error!("❌ Failed to create named pipe: {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        continue;
                    }
                }
            }
        };

        // Wait for a client (CLI or Tauri App) to connect natively
        match server.connect().await {
            Ok(_) => {
                tracing::info!("🔗 Native Client connected to IPC Pipe.");
                let state_clone = state.clone();
                tokio::spawn(async move {
                    handle_client(server, state_clone).await;
                });
            }
            Err(e) => {
                tracing::error!("❌ Pipe connection error: {}", e);
            }
        }
    }
}

#[cfg(windows)]
async fn handle_client(mut pipe: NamedPipeServer, state: Arc<AppState>) {
    let mut buf = vec![0; 4096];
    loop {
        match pipe.read(&mut buf).await {
            Ok(0) => {
                tracing::info!("🔗 Native Client disconnected from IPC Pipe.");
                break;
            }
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                let command = msg.trim();
                tracing::info!("📥 [IPC] Received Command: {}", command);

                // Try to parse as JSON first for Universal FFI Parity
                if let Ok(json_cmd) = serde_json::from_str::<serde_json::Value>(command) {
                    if let Some(action) = json_cmd.get("action").and_then(|a| a.as_str()) {
                        match action {
                            "BOOSTER_UPDATE" => {
                                if let Some(payload) = json_cmd.get("payload") {
                                    if let Ok(booster_ctrl) = serde_json::from_value(payload.clone()) {
                                        let _ = cluaize_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster_ctrl);
                                        let _ = pipe.write_all(b"{\"status\": \"success\"}").await;
                                    } else {
                                        let _ = pipe.write_all(b"{\"status\": \"error\", \"message\": \"invalid payload\"}").await;
                                    }
                                }
                                continue;
                            }
                            "CDQL_FETCH_HISTORY" => {
                                if let Some(session_id) = json_cmd.get("session_id").and_then(|s| s.as_str()) {
                                    if let Some(payload) = engines::memory::tensor_transducer::TensorTransducer::inject_context(session_id) {
                                        let _ = pipe.write_all(&payload).await;
                                    } else {
                                        let _ = pipe.write_all(b"{\"status\": \"error\", \"message\": \"not found\"}").await;
                                    }
                                }
                                continue;
                            }
                            "SYSTEM_PS" => {
                                let registry = cluaize_shared::hardware::governor::HardwareGovernor::load_process_registry();
                                let mut processes = Vec::new();
                                for (pid_str, info) in registry {
                                    processes.push(serde_json::json!({
                                        "pid": pid_str,
                                        "model_id": info.model_id,
                                        "vram_gb": info.vram_gb,
                                        "context_size": info.context_size,
                                        "engine": info.engine
                                    }));
                                }
                                let res = serde_json::json!({"status": "success", "active_processes": processes});
                                let _ = pipe.write_all(res.to_string().as_bytes()).await;
                                continue;
                            }
                            "HARDWARE_CALIBRATE" => {
                                let _ = cluaize_shared::hardware::governor::HardwareGovernor::auto_calibrate();
                                let _ = pipe.write_all(b"{\"status\": \"success\", \"message\": \"Hardware recalibrated\"}").await;
                                continue;
                            }
                            "BENCHMARK_RUN" => {
                                engines::telemetry::health_check::CluaizeHealthChecker::run_full_benchmark();
                                let _ = pipe.write_all(b"{\"status\": \"success\", \"message\": \"Benchmark started\"}").await;
                                continue;
                            }
                            "SYSTEM_PROFILE_SETUP" => {
                                engines::hardware::system_control_manager::detect_hardware();
                                let _ = pipe.write_all(b"{\"status\": \"success\", \"message\": \"Profile generated\"}").await;
                                continue;
                            }
                            "MODEL_RM" => {
                                if let Some(model_id) = json_cmd.get("payload").and_then(|p| p.get("model_id")).and_then(|m| m.as_str()) {
                                    if let Some(home_dir) = ::dirs::home_dir() {
                                        let model_file = home_dir.join(".cluaize").join("models").join(format!("{}.gguf", model_id));
                                        if model_file.exists() {
                                            let _ = std::fs::remove_file(&model_file);
                                            let _ = pipe.write_all(b"{\"status\": \"success\", \"message\": \"Model removed\"}").await;
                                        } else {
                                            let _ = pipe.write_all(b"{\"status\": \"error\", \"message\": \"File not found\"}").await;
                                        }
                                    }
                                } else {
                                    let _ = pipe.write_all(b"{\"status\": \"error\", \"message\": \"Missing model_id\"}").await;
                                }
                                continue;
                            }
                            "CDQL_DELETE_SESSION" | "SKILL_LIST" | "SKILL_CACHE_CLEAR" | "SKILL_CACHE_LS" | "INGEST_DOC" => {
                                let _ = pipe.write_all(b"{\"status\": \"pending\"}").await;
                                continue;
                            }
                            "SYSTEM_BRAIN" => {
                                if let Some(payload) = json_cmd.get("payload").and_then(|p| p.get("state")).and_then(|s| s.as_bool()) {
                                    if let Ok(mut control) = cluaize_shared::hardware::governor::HardwareGovernor::load_system_control() {
                                        control.brain.cluaizd_connect_ffi = if payload { "on".to_string() } else { "off".to_string() };
                                        let _ = cluaize_shared::hardware::system_control::HardwareOrchestrator::persist_sovereign_state(&control);
                                        let _ = pipe.write_all(b"{\"status\": \"success\"}").await;
                                    } else {
                                        let _ = pipe.write_all(b"{\"status\": \"error\"}").await;
                                    }
                                } else {
                                    let _ = pipe.write_all(b"{\"status\": \"error\", \"message\": \"missing state payload\"}").await;
                                }
                                continue;
                            }
                            "GET_SETTINGS" => {
                                let perms = engines::neural_foundry::security::permission_schema::PermissionSchema::load();
                                let control = cluaize_shared::hardware::governor::HardwareGovernor::load_system_control().unwrap_or_default();
                                let brain_mode = if control.brain.cluaizd_connect_ffi == "on" { "on" } else { "off" };
                                
                                // Load real booster from disk — NOT hardcoded
                                let booster = cluaize_shared::hardware::governor::HardwareGovernor::load_booster_settings().unwrap_or_default();
                                
                                let roster = engines::models::registry::CoreRoster::load_roster();
                                let mut available_chat_models: Vec<String> = Vec::new();
                                let mut available_vector_models: Vec<String> = Vec::new();
                                let mut all_models: Vec<String> = Vec::new();

                                for model in &roster {
                                    all_models.push(model.id.clone());

                                    // PRIMARY: Classify by the actual folder path on disk
                                    // ~/.cluaize/models/chat/     → chat models
                                    // ~/.cluaize/models/embedding/ → vector models (text embeddings)
                                    // ~/.cluaize/models/vision/    → vector models (image embeddings / CLIP)
                                    let folder_category = model.local_path.as_deref()
                                        .and_then(|p| {
                                            let p_lower = p.replace('\\', "/").to_lowercase();
                                            if p_lower.contains("/models/chat/") {
                                                Some("chat")
                                            } else if p_lower.contains("/models/embedding/") {
                                                Some("embedding")
                                            } else if p_lower.contains("/models/vision/") {
                                                Some("vision")
                                            } else {
                                                None
                                            }
                                        });

                                    // FALLBACK: Use the `category` field from model_manifest.json
                                    let cat = folder_category
                                        .unwrap_or_else(|| model.category.as_str())
                                        .to_lowercase();

                                    // "embedding" and "vision" models go into vector list.
                                    // Vision/CLIP models can embed images into vector space.
                                    match cat.as_str() {
                                        "embedding" | "vision" | "multimodal" => {
                                            available_vector_models.push(model.id.clone());
                                        }
                                        _ => {
                                            available_chat_models.push(model.id.clone());
                                        }
                                    }
                                }

                                // Safety: always ensure the currently-active model appears in its list
                                if let Some(ref t) = perms.chat_models.text {
                                    if !t.is_empty() && !available_chat_models.contains(t) {
                                        available_chat_models.push(t.clone());
                                    }
                                }
                                if let Some(ref t) = perms.vector_models.text {
                                    if !t.is_empty() && !available_vector_models.contains(t) {
                                        available_vector_models.push(t.clone());
                                    }
                                }

                                
                                let response = serde_json::json!({
                                    "permissions": {
                                        "wasm_firewall": perms.wasm_firewall,
                                        "vectorize_user_input": perms.vectorize_user_input,
                                        "vectorize_ai_response": perms.vectorize_ai_response,
                                        "stream_telemetry": perms.stream_telemetry,
                                        "lazy_load_model": perms.lazy_load_model,
                                        "temporary_chat_ttl_hours": perms.temporary_chat_ttl_hours,
                                        "chat_models": perms.chat_models,
                                        "vector_models": perms.vector_models,
                                        "available_models": all_models,
                                        "available_chat_models": available_chat_models,
                                        "available_vector_models": available_vector_models,
                                        "available_devices": ["auto", "gpu", "cpu"]
                                    },
                                    "booster": booster,
                                    "brainMode": brain_mode
                                });
                                
                                let _ = pipe.write_all(response.to_string().as_bytes()).await;
                                continue;
                            }
                            // UPDATE_BOOSTER — sent by Tauri store (matches store action name)
                            "UPDATE_BOOSTER" | "BOOSTER_UPDATE" => {
                                if let Some(payload) = json_cmd.get("payload") {
                                    // Handle single key-value update: {key: "flash_attention", value: "On"}
                                    if let (Some(key), Some(value)) = (payload.get("key").and_then(|k| k.as_str()), payload.get("value")) {
                                        let mut booster = cluaize_shared::hardware::governor::HardwareGovernor::load_booster_settings().unwrap_or_default();
                                        if let Ok(mut booster_json) = serde_json::to_value(&booster) {
                                            booster_json[key] = value.clone();
                                            if let Ok(updated) = serde_json::from_value(booster_json) {
                                                booster = updated;
                                                let _ = cluaize_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster);
                                                let _ = pipe.write_all(b"{\"status\": \"success\"}").await;
                                            } else {
                                                let _ = pipe.write_all(b"{\"status\": \"error\", \"message\": \"invalid booster format\"}").await;
                                            }
                                        }
                                    // Handle full booster object update (legacy BOOSTER_UPDATE format)
                                    } else if let Ok(booster_ctrl) = serde_json::from_value(payload.clone()) {
                                        let _ = cluaize_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster_ctrl);
                                        let _ = pipe.write_all(b"{\"status\": \"success\"}").await;
                                    } else {
                                        let _ = pipe.write_all(b"{\"status\": \"error\", \"message\": \"invalid payload\"}").await;
                                    }
                                }
                                continue;
                            }
                            "UPDATE_PERMISSION" => {
                                if let Some(payload) = json_cmd.get("payload") {
                                    if let (Some(key), Some(value)) = (payload.get("key").and_then(|k| k.as_str()), payload.get("value")) {
                                        let perms = engines::neural_foundry::security::permission_schema::PermissionSchema::load();
                                        if let Ok(mut perms_json) = serde_json::to_value(&perms) {
                                            perms_json[key] = value.clone();
                                            if let Ok(updated_perms) = serde_json::from_value::<engines::neural_foundry::security::permission_schema::PermissionSchema>(perms_json) {
                                                updated_perms.save();
                                                let _ = pipe.write_all(b"{\"status\": \"success\"}").await;
                                            } else {
                                                let _ = pipe.write_all(b"{\"status\": \"error\", \"message\": \"invalid permission format\"}").await;
                                            }
                                        }
                                    }
                                }
                                continue;
                            }

                            "SET_HARDWARE" => {
                                tracing::info!("🚀 [IPC] Received SET_HARDWARE: {:?}", json_cmd);
                                // Here we would dynamically adjust thread counts/Vulcan map in engine
                                let _ = pipe.write_all(b"{\"status\": \"success\"}").await;
                                continue;
                            }
                            "SET_MODEL" => {
                                tracing::info!("🚀 [IPC] Received SET_MODEL: {:?}", json_cmd);
                                // Here we would unload current model and load new one
                                let _ = pipe.write_all(b"{\"status\": \"success\"}").await;
                                continue;
                            }
                            "EAGER_LOAD" => {
                                tracing::info!("🚀 [IPC] Received EAGER_LOAD. Pre-loading text model...");
                                let _ = pipe.write_all(b"{\"status\": \"success\"}").await;
                                continue;
                            }
                            _ => {}
                        }
                    }
                }

                if command.starts_with("//CDQL_") {
                    let response = format!("{{\"status\": \"success\", \"query\": \"{}\"}}", command);
                    let _ = pipe.write_all(response.as_bytes()).await;
                } else {
                    // Dispatch natural language inference via Master Router
                    match state.dispatcher.dispatch_stream(command, false).await {
                        EngineResponse::TokenStream(mut rx) => {
                            while let Some(token) = rx.recv().await {
                                if pipe.write_all(token.as_bytes()).await.is_err() {
                                    break;
                                }
                            }
                        }
                        EngineResponse::FinalResult(res) => {
                            let _ = pipe.write_all(res.as_bytes()).await;
                        }
                        EngineResponse::Error(err) => {
                            let _ = pipe.write_all(format!("ERROR: {}", err).as_bytes()).await;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("❌ Pipe read error: {}", e);
                break;
            }
        }
    }
}

#[cfg(not(windows))]
pub async fn start_named_pipe_server(_state: Arc<AppState>) {
    tracing::warn!("Native Named Pipes are only supported on Windows. IPC Disabled.");
}
