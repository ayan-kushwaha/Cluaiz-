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
                                        let _ = cluaiz_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster_ctrl);
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
                                let registry = cluaiz_shared::hardware::governor::HardwareGovernor::load_process_registry();
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
                                let _ = cluaiz_shared::hardware::governor::HardwareGovernor::auto_calibrate();
                                let _ = pipe.write_all(b"{\"status\": \"success\", \"message\": \"Hardware recalibrated\"}").await;
                                continue;
                            }
                            "BENCHMARK_RUN" => {
                                engines::telemetry::health_check::CluaizHealthChecker::run_full_benchmark();
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
                                        let model_file = home_dir.join(".cluaiz").join("models").join(format!("{}.gguf", model_id));
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
                                if let Some(payload) = json_cmd.get("payload").and_then(|p| p.get("state")).and_then(|s| s.as_str()) {
                                    if let Ok(mut control) = cluaiz_shared::hardware::governor::HardwareGovernor::load_system_control() {
                                        control.brain.cluaizd_connect_ffi = payload.to_string();
                                        let _ = cluaiz_shared::hardware::system_control::HardwareOrchestrator::persist_sovereign_state(&control);
                                        let _ = pipe.write_all(b"{\"status\": \"success\"}").await;
                                    } else {
                                        let _ = pipe.write_all(b"{\"status\": \"error\"}").await;
                                    }
                                } else {
                                    let _ = pipe.write_all(b"{\"status\": \"error\", \"message\": \"missing state payload\"}").await;
                                }
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
