//! Sovereign Implementation B: Core Loader.

use archer_shared::backend::signature::{BackendType, KernelSignature};
use archer_shared::backend::context::CluaizContext;
use archer_shared::backend::traits::ModelWeightsWrapper;
use std::sync::Arc;
use crate::config::RuntimeBConfig;

pub struct RuntimeBLoader;

impl RuntimeBLoader {
    pub fn register_drivers(mut register_fn: impl FnMut(BackendType, KernelSignature, archer_shared::ArcConstructor)) -> Result<(), String> {
        let patterns = vec!["uniform", "asymmetric"];

        for pattern in patterns {
            let signature = KernelSignature {
                has_experts: false,
                is_asymmetric: pattern == "asymmetric",
                is_multimodal: true,
                is_heterogeneous: true,
                is_bitnet: false,
                is_ssm: false,
                head_pattern: pattern.into(),
                activation: "silu".into(),
            };

            register_fn(
                BackendType::RuntimeB,
                signature,
                Arc::new(
                    |model_load_path: &str,
                     sovereign_context: CluaizContext| {
                        // Dynamic param resolution
                        let _params = RuntimeBConfig::resolve(&sovereign_context.dna);
                        
                        let engine = crate::RuntimeB::new(model_load_path, sovereign_context);
                        Ok(Box::new(engine) as ModelWeightsWrapper)
                    },
                ) as archer_shared::ArcConstructor,
            );

        }
        Ok(())
    }
}
