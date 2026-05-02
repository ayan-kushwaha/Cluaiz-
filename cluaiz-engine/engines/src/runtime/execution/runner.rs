//! ═══════════════════════════════════════════════════════════════════════
//!  CURE Engine: Universal Runner (Sovereign)
//! ═══════════════════════════════════════════════════════════════════════

use anyhow::Result;
use candle_core::Device;
use crate::runtime::execution::sampler::NeuralSampler;
use archer_shared::ModelWeightsWrapper;
use tokenizers::Tokenizer;


#[derive(Debug, Clone)]
pub struct SovereignMetrics {
    pub ttft_ms: f64,
    pub tps: f64,
    pub total_tokens: usize,
    pub total_time_ms: f64,
}

pub struct SovereignRunner {
    pub model: ModelWeightsWrapper,
    pub tokenizer: Tokenizer,
    pub sampler: NeuralSampler,
    pub bos_token_id: Option<u32>,
    pub device: Device,
}

impl SovereignRunner {
    pub fn new(model: ModelWeightsWrapper, tokenizer: Tokenizer, sampler: NeuralSampler, bos_token_id: Option<u32>, device: Device) -> Self {
        Self { model, tokenizer, sampler, bos_token_id, device }
    }

    /// 🔗 Instant Recall: Injects neural sovereign signals before generation.
    pub fn inject_neural_signals(&mut self, signals: Vec<archer_shared::hardware::memory::kv_cache::stitching::SovereignSignal>) -> Result<()> {
        self.model.inject_signals(signals)
    }

    pub fn generate(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        mut callback: impl FnMut(String) + Send + 'static,
    ) -> Result<SovereignMetrics> {
        // 🛰️ SOVEREIGN BOOSTER SYNC: Load truth from Governor before generation
        let booster = archer_shared::hardware::governor::HardwareGovernor::load_booster_settings().unwrap_or_default();
        self.model.apply_booster(&booster)?;
        
        // 🌊 Liquid Mode Linkage
        if booster.turbo_quant == archer_shared::hardware::schema::booster::FeatureState::On {
            self.model.set_liquid_mode(true)?;
        }

        let start_time = std::time::Instant::now();
        
        let token_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count_clone = std::sync::Arc::clone(&token_count);

        // 🧪 ARCHER V5.3 CONVERGENCE: Delegating generation to the kernel
        self.model.generate_stream(
            prompt,
            max_tokens,
            &self.tokenizer,
            Box::new(move |t| {
                count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                callback(t);
            }),
        )?;

        let total_duration = start_time.elapsed();
        let total_seconds = total_duration.as_secs_f64();
        let actual_tokens = token_count.load(std::sync::atomic::Ordering::SeqCst);

        // Safe division logic to avoid infinite TPS anomalies
        let tps = if total_seconds > 0.0 && actual_tokens > 0 {
            actual_tokens as f64 / total_seconds
        } else {
            0.0
        };

        Ok(SovereignMetrics {
            ttft_ms: 0.0, // Model TTFT placeholder for now
            tps,
            total_tokens: actual_tokens,
            total_time_ms: total_duration.as_secs_f64() * 1000.0,
        })
    }
}
