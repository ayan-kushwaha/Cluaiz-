use serde::{Deserialize, Serialize};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use crate::backend::context::SovereignContext;
use crate::backend::traits::ModelWeightsWrapper;

/// ArcConstructor: The factory closure signature for instantiating model architectures.

/// Removed candle-core dependencies to allow for truly agnostic engine backends.
pub type ArcConstructor = std::sync::Arc<dyn Fn(
    &str,                // load_path
    SovereignContext,     // system context
) -> anyhow::Result<ModelWeightsWrapper> + Send + Sync>;


// ─── Kernel Signature (The Mathematical Identity) ──────────────────────────
#[derive(Debug, Clone, PartialEq, Eq, Default, std::hash::Hash, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct KernelSignature {
    pub has_experts: bool,
    pub is_asymmetric: bool,
    pub is_multimodal: bool,
    pub is_heterogeneous: bool,
    pub is_bitnet: bool,
    pub is_ssm: bool, // Mamba/Linear Recurrence
    pub head_pattern: String, 
    pub activation: String,
}

// ─── Core Backend Enums ────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
#[serde(rename_all = "camelCase")]
pub enum BackendType {

    #[serde(alias = "candlenative")]
    RuntimeA,
    #[serde(alias = "llamacppffi")]
    RuntimeB,
    #[serde(alias = "bitnetnative")]
    RuntimeC,
    #[serde(alias = "tritonkernel")]
    RuntimeD,
    #[serde(alias = "moerouter")]
    SwitchNode,
}

// ─── Global Registration System ──────────────────────────────────────────
pub struct GlobalFeatureRegistry;

impl GlobalFeatureRegistry {
    /// Dispatches a model loader based on the model signature matching
    pub fn select_runtime(signature: &KernelSignature) -> BackendType {
        // 🚀 Sovereign Dispatch Logic
        // BitNet models are now supported natively via RuntimeC (Engine C)
        if signature.is_bitnet {
            tracing::info!("🔩 [Signature] 1-bit architecture detected. Routing to Runtime C (Native Bit-Depth).");
            BackendType::RuntimeC
        } else if signature.is_heterogeneous {
            BackendType::RuntimeA 
        } else if signature.is_asymmetric {
             BackendType::RuntimeB 
        } else {
             BackendType::RuntimeA
        }
    }
}
