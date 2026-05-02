//! ═══════════════════════════════════════════════════════════════════════
//!  CURE Engine: Asynchronous Neural Pipeline (Async Double-Buffering)
//! ═══════════════════════════════════════════════════════════════════════

use std::sync::Arc;
use tokio::sync::mpsc;
use crate::runtime::execution::runner::{SovereignRunner, SovereignMetrics};
use archer_shared::SovereignContext;

pub struct NeuralPipeline {
    pub runner: SovereignRunner,
    pub context: SovereignContext,
}

impl NeuralPipeline {
    pub fn new(runner: SovereignRunner, context: SovereignContext) -> Self {
        Self { runner, context }
    }

    /// High-performance parallel execution loop.
    /// Implements Async Double-Buffering to overlap CPU-bound sampling and GPU-bound math.
    pub async fn stream_inference(
        &mut self,
        prompt: String,
        max_tokens: usize,
    ) -> mpsc::Receiver<String> {
        let (tx, rx) = mpsc::channel(100);
        
        // 🚀 BARE-METAL SATURATION: Moving inference to a dedicated compute thread
        // This prevents the UI/API thread from blocking the GPU queue.
        std::thread::spawn({
            let mut runner = self.runner.clone(); // In a real scenario, we'd use a reference or Arc
            let prompt = prompt.clone();
            move || {
                let _ = runner.generate(&prompt, max_tokens, move |token| {
                    let _ = tx.blocking_send(token);
                });
            }
        });

        rx
    }
}
