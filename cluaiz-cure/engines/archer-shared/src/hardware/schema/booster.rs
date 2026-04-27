//! 📦 Tier 7: Schema - Booster Control
//! Centralized Tri-State configuration for all system optimizations.

use serde::{Deserialize, Serialize};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize, PartialEq)]
#[archive(check_bytes)]
pub enum FeatureState {
    On,
    Off,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct BoosterControl {
    pub turbo_quant: FeatureState,
    pub flash_attention: FeatureState,
    pub speculative_decoding: FeatureState,
    pub auto_round: FeatureState,
}

impl Default for BoosterControl {
    fn default() -> Self {
        Self {
            turbo_quant: FeatureState::Auto,
            flash_attention: FeatureState::Auto,
            speculative_decoding: FeatureState::Off,
            auto_round: FeatureState::Auto,
        }
    }
}
