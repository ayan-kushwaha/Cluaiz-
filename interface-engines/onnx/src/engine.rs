use anyhow::Result;
use ort::session::Session;
use tokenizers::Tokenizer;
use std::sync::Arc;

/// The Sovereign Gatekeeper: ONNX Multimodal Router
pub struct OnnxEngine {
    // Real Production Engine State
    pub(crate) session: Option<Arc<std::sync::Mutex<Session>>>,
    pub(crate) tokenizer: Option<Arc<Tokenizer>>,
}

impl OnnxEngine {
    pub fn new() -> Result<Self> {
        // Initialize ONNX Runtime environment implicitly.
        ort::init()
            .with_name("cluaiz_onnx_env")
            .commit();

        tracing::info!("🧿 [ONNX] Runtime initialized. Ready to load models via API.");

        Ok(Self {
            session: None,
            tokenizer: None,
        })
    }

    /// Dynamically load a model from disk into the ONNX Runtime (e.g. bge-m3-quantized.onnx)
    pub fn load_text_model(&mut self, model_path: &str, tokenizer_path: &str) -> Result<()> {
        tracing::info!("📦 [ONNX] Loading model from: {}", model_path);
        
        // Build the session for CPU execution
        let session = Session::builder()?
            .with_intra_threads(4).map_err(|e| anyhow::anyhow!("Threads error: {:?}", e))?
            .commit_from_file(model_path)
            .map_err(|e| anyhow::anyhow!("ORT Session failed: {}", e))?;

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Tokenizer failed: {}", e))?;

        self.session = Some(Arc::new(std::sync::Mutex::new(session)));
        self.tokenizer = Some(Arc::new(tokenizer));
        
        Ok(())
    }
}
