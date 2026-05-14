//! 📦 Tier 7: Schema - Booster Control
//! Centralized Tri-State configuration for all system optimizations.

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
#[serde(untagged)]
pub enum SmartState<T> {
    Static(String),
    Custom(T),
}

impl<T> SmartState<T> {
    pub fn is_active(&self) -> bool {
        match self {
            SmartState::Static(s) => s == "On" || s == "Auto",
            SmartState::Custom(_) => true, // Custom config implies active
        }
    }
}

impl<T> Default for SmartState<T> {
    fn default() -> Self {
        SmartState::Static("Auto".into())
    }
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize, PartialEq, Default,
)]
#[archive(check_bytes)]
pub enum FeatureState {
    On,
    Off,
    #[default]
    Auto,
}

impl FeatureState {
    pub fn is_active(&self) -> bool {
        matches!(self, FeatureState::On | FeatureState::Auto)
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Default, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct DFlashConfig {
    pub state: String,       // Always "On" in this variant
    pub budget: u32,         // Safe user-tunable knob
    pub asymmetric_kv: bool, // TQ3_0 for K, F16 for V
    pub draft_model_path: Option<String>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize, PartialEq, Default,
)]
#[archive(check_bytes)]
pub enum BoosterMode {
    #[serde(rename = "edge")]
    Edge,           // 📱 Mobile/NPU/Pi (Extreme pruning)
    #[default]
    #[serde(rename = "multitasking")]
    Multitasking,   // 💻 Standard Laptop (Respects OS/Apps)
    #[serde(rename = "balance")]
    Balance,        // ⚖️ Standard Performance
    #[serde(rename = "max_boost")]
    MaxBoost,       // 🚀 AI Priority (Workstation)
    #[serde(rename = "ultra_max_boost")]
    UltraMaxBoost,  // 🔥 Reclaims everything (Formerly Landlord)
    #[serde(rename = "hyper_cluster")]
    HyperCluster,   // 🌌 Server/H100 Cluster (Zero-margin orchestration)
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct BoosterControl {
    pub mode_run: BoosterMode,
    pub turbo_quant: FeatureState,
    pub flash_attention: FeatureState,
    pub speculative_decoding: FeatureState,
    pub auto_round: FeatureState,
    pub dflash: SmartState<DFlashConfig>,
    pub force_vram_reclaim: FeatureState, // 🏠 'Landlord' Mode Flag
}

impl BoosterControl {
    /// 🧠 The Conflict Resolution Manager
    pub fn resolve_conflicts(
        &mut self,
        silicon: &crate::hardware::schema::profiles::SiliconTruth,
        signature: &crate::backend::signature::KernelSignature,
    ) {
        // 1. BitNet/SSM Constraints
        if signature.is_bitnet || signature.is_ssm {
            // BitNet doesn't need DFlash/Speculative usually
            self.speculative_decoding = FeatureState::Off;
        }

        // 2. VRAM Dependency
        let vram_available = silicon
            .accelerators
            .gpus
            .iter()
            .map(|g| g.vram_available_gb)
            .sum::<f64>();

        if self.speculative_decoding == FeatureState::On {
            if vram_available < 12.0 {
                // Force TurboQuant ON to fit draft model
                self.turbo_quant = FeatureState::On;
                println!("⚖️ [ConflictManager] Low VRAM ({:.1}GB) detected. Forcing TurboQuant = ON for Speculative Decoding.", vram_available);
            }
            // Flash Attention synergy
            if self.flash_attention == FeatureState::Auto {
                self.flash_attention = FeatureState::On;
            }
        }
    }
}

impl Default for BoosterControl {
    fn default() -> Self {
        Self {
            mode_run: BoosterMode::Balance,
            turbo_quant: FeatureState::Auto,
            flash_attention: FeatureState::Auto,
            speculative_decoding: FeatureState::Off,
            auto_round: FeatureState::Auto,
            dflash: SmartState::Static("Auto".into()),
            force_vram_reclaim: FeatureState::Off,
        }
    }
}

/// 🚀 FFI-Compatible C-Struct for injecting Booster configurations into C++ Kernels (llama.cpp)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CluaizBoosterContext {
    pub turbo_quant: bool,
    pub flash_attention: bool,
    pub speculative_decoding: bool,
}

impl From<&BoosterControl> for CluaizBoosterContext {
    fn from(config: &BoosterControl) -> Self {
        Self {
            turbo_quant: config.turbo_quant == FeatureState::On,
            flash_attention: config.flash_attention == FeatureState::On || config.flash_attention == FeatureState::Auto,
            speculative_decoding: config.speculative_decoding == FeatureState::On,
        }
    }
}
