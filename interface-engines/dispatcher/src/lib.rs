use anyhow::{Result, anyhow};
use cluaiz_shared::backend::signature::{KernelSignature, GlobalFeatureRegistry, BackendType};
use system_booster::BoosterControl;

use tokio::sync::mpsc;

pub enum EngineResponse {
    TokenStream(mpsc::Receiver<String>),
    FinalResult(String),
    Error(String),
}

/// 🚦 NeuralDispatcher (The Master Router)
/// The core router that owns hardware logic and dispatches prompts across Native IPC and HTTP.
pub struct NeuralDispatcher {
    pub booster_state: BoosterControl,
    pub current_signature: KernelSignature,
    // Future additions: TensorTransducer, NeuralFoundry instances
}

impl NeuralDispatcher {
    pub fn new(booster_state: BoosterControl, signature: KernelSignature) -> Self {
        Self {
            booster_state,
            current_signature: signature,
        }
    }

    /// Primary entry point for real-time token streaming.
    /// Used by both the FFI Named Pipes (Native Desktop) and HTTP SSE (External).
    pub async fn dispatch_stream(&self, prompt: &str, skip_brain: bool) -> EngineResponse {
        // 🚀 Real-time Silicon Probe
        let hardware = cluaiz_shared::hardware::HardwareOrchestrator::probe().silicon_truth;
        let backend = GlobalFeatureRegistry::select_runtime(&self.current_signature, &hardware);
        
        tracing::info!("🚦 [Master Router] Routing prompt to backend: {:?}", backend);

        let (tx, rx) = mpsc::channel::<String>(100);
        let prompt_clone = prompt.to_string();

        match backend {
            BackendType::RuntimeB | BackendType::RuntimeC | BackendType::RuntimeA => {
                tokio::spawn(async move {
                    // Mocking actual engine stream output for architectural wiring
                    let words = prompt_clone.split_whitespace();
                    for word in words {
                        if tx.send(format!("{} ", word)).await.is_err() {
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                    }
                    let _ = tx.send("\n[DONE]\n".to_string()).await;
                });
                EngineResponse::TokenStream(rx)
            }
            _ => {
                EngineResponse::Error(format!("Unsupported backend architecture: {:?}", backend))
            }
        }
    }

    /// Legacy blocking call, to be deprecated once all clients shift to `dispatch_stream`.
    pub async fn dispatch_prompt(&self, prompt: &str) -> Result<String> {
        let mut stream = match self.dispatch_stream(prompt, false).await {
            EngineResponse::TokenStream(rx) => rx,
            EngineResponse::Error(e) => return Err(anyhow::anyhow!(e)),
            EngineResponse::FinalResult(r) => return Ok(r),
        };
        
        let mut final_text = String::new();
        while let Some(token) = stream.recv().await {
            if token.trim() == "[DONE]" { break; }
            final_text.push_str(&token);
        }
        Ok(final_text)
    }
}

/// 🚥 EmbeddingDispatcher
/// Routes embedding requests to ONNX (default) or Llama depending on the configuration.
pub struct EmbeddingDispatcher {
    pub onnx_engine: cluaiz_onnx::OnnxEngine,
    // Future: pub llama_engine: Option<llama::LlamaEngine>,
}

impl EmbeddingDispatcher {
    pub fn new() -> Result<Self> {
        let onnx_engine = cluaiz_onnx::OnnxEngine::new()?;
        Ok(Self { onnx_engine })
    }

    /// Primary entry point for vector generation.
    pub fn dispatch_embedding(&self, text: &str) -> Result<Vec<f32>> {
        use neural_core::interfaces::router_contract::EmbeddingDriver;
        tracing::info!("🚥 [Dispatcher] Routing embedding request to ONNX...");
        self.onnx_engine.gen_embedding(text).map_err(|e| anyhow::anyhow!("Embedding Error: {}", e))
    }

    pub fn dispatch_multimodal(&self, bytes: &[u8], modality: neural_core::interfaces::router_contract::Modality) -> Result<Vec<f32>> {
        use neural_core::interfaces::router_contract::EmbeddingDriver;
        tracing::info!("🚥 [Dispatcher] Routing multimodal request to ONNX...");
        self.onnx_engine.gen_multimodal_embedding(bytes, modality).map_err(|e| anyhow::anyhow!("Multimodal Error: {}", e))
    }
}
