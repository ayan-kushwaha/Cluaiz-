use anyhow::Result;
use neural_core::interfaces::router_contract::{EmbeddingDriver, EngineError, Modality};

// Core Engine state
pub mod engine;
pub use engine::OnnxEngine;

// Modality Processors
pub mod text;
pub mod audio;
pub mod vision;

impl EmbeddingDriver for OnnxEngine {
    fn gen_embedding(&self, text: &str) -> Result<Vec<f32>, EngineError> {
        // Dispatch to the Text Sub-Engine
        self.execute_text_embedding(text)
    }

    fn gen_multimodal_embedding(&self, payload: &[u8], modality: Modality) -> Result<Vec<f32>, EngineError> {
        match modality {
            Modality::Text => {
                let text = std::str::from_utf8(payload).unwrap_or("");
                self.execute_text_embedding(text)
            },
            Modality::Audio => self.execute_audio_embedding(payload),
            Modality::Image => self.execute_vision_embedding(payload),
        }
    }
}
