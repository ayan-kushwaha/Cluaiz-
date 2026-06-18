use crate::engine::OnnxEngine;
use neural_core::interfaces::router_contract::EngineError;

impl OnnxEngine {
    pub fn execute_audio_embedding(&self, _bytes: &[u8]) -> Result<Vec<f32>, EngineError> {
        // Whisper Audio Tensor Extraction Logic
        Err(EngineError::UnsupportedModality("Audio ONNX graph not loaded yet".into()))
    }
}
 
