//! Sovereign Implementation B: Core Loader.

use cluaize_shared::backend::signature::{BackendType, KernelSignature};
use cluaize_shared::backend::context::CluaizeContext;
use cluaize_shared::backend::traits::ModelWeightsWrapper;
use std::sync::Arc;
use crate::config::BoosterConfig;

pub struct RuntimeBLoader;

impl RuntimeBLoader {
    pub fn register_drivers(mut register_fn: impl FnMut(BackendType, KernelSignature, cluaize_shared::ArcConstructor)) -> Result<(), String> {
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
                     sovereign_context: CluaizeContext| {
                        // Dynamic param resolution (Handled autonomously by BoosterConfig)
                        
                        let engine = crate::RuntimeB::new(model_load_path, sovereign_context);
                        Ok(Box::new(engine) as ModelWeightsWrapper)
                    },
                ) as cluaize_shared::ArcConstructor,
            );

        }
        Ok(())
    }
}
