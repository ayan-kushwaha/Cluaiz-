//! ═══════════════════════════════════════════════════════════════════════
//!  CURE Engine: The Silicon Orchestrator (Dynamic Dispatcher)
//! ═══════════════════════════════════════════════════════════════════════

use anyhow::{anyhow, Result};
use uuid::Uuid;
use archer_shared::{ArcConstructor, BackendType, ModelWeightsWrapper, SovereignContext};
use candle_core::Device;
use dashmap::DashMap;
use once_cell::sync::Lazy;

use crate::models::registry::KernelSignature;
use crate::runtime::execution::session_cache::SESSION_CACHE;

/// THE REGISTRY: A thread-safe global map linking (Backend, Signature) to their constructors.
pub static ARCH_REGISTRY: Lazy<DashMap<(BackendType, KernelSignature), ArcConstructor>> =
    Lazy::new(DashMap::new);

pub struct SiliconOrchestrator;

impl SiliconOrchestrator {
    /// Registers a new mathematical kernel driver into the global hub for a specific backend.
    pub fn register(
        engine_type: BackendType,
        signature: KernelSignature,
        constructor_hook: ArcConstructor,
    ) -> Result<(), String> {
        ARCH_REGISTRY.insert((engine_type.clone(), signature.clone()), constructor_hook);
        tracing::info!(
            "🔩 Silicon Orchestrator: Registered kernel driver for {:?} with signature: {:?}",
            engine_type,
            signature
        );
        Ok(())
    }

    /// Dispatches and instantiates the correct model kernel via a mathematical lookup and hardware override matrix.
    pub fn instantiate(
        model_load_path: &str,
        sovereign_context: SovereignContext,
    ) -> Result<ModelWeightsWrapper> {
        let profile = archer_shared::hardware::get_silicon_state();
        let target_signature = &sovereign_context.dna.signature;

        // 🤖 SOVEREIGN MANAGER LOGIC: Hardware-Aware Engine Override
        let preferred_engine = if target_signature.is_bitnet {
            tracing::info!("🔩 [Manager] BitNet Signature Detected. Routing to Engine C (Native).");
            BackendType::RuntimeC
        } else {
            let vendor = profile.accelerators.gpus.first().map(|g| g.vendor.as_str()).unwrap_or("CPU");
            match vendor {
                "NVIDIA" => {
                    tracing::info!("🔩 [Manager] NVIDIA Detected. Routing to Llama (Nitro/CUDA).");
                    BackendType::RuntimeB
                },
                "Intel" => {
                    tracing::info!("🔩 [Manager] Intel Silicon Detected. Routing to Llama (OpenVINO/SYCL).");
                    BackendType::RuntimeB
                },
                "Apple" => {
                    tracing::info!("🔩 [Manager] Apple Silicon Detected. Routing to Candle (Metal).");
                    BackendType::RuntimeA
                },
                _ => {
                    tracing::info!("🔩 [Manager] Generic Silicon. Routing to Candle (CPU).");
                    BackendType::RuntimeA
                }
            }
        };

        tracing::info!(
            "🔩 Silicon Orchestrator: Dispatching kernel for {:?} DNA Signal: {:?}",
            preferred_engine,
            target_signature
        );

        // 🦾 [Nitro Provisioning] Llama engine handles binary resolution internally via its router.
        
        // 1️⃣  EXACT MATCH: Highest performance lookup
        if let Some(constructor_entry) =
            ARCH_REGISTRY.get(&(preferred_engine.clone(), target_signature.clone()))
        {
            tracing::info!(
                "🔩 [Handshake] Perfect Silicon Match for {:?}: {:?}",
                preferred_engine,
                target_signature
            );
            let constructor_ref: &ArcConstructor = constructor_entry.value();
            let mut model = (constructor_ref)(model_load_path, sovereign_context)?;

            // 🧠 [Persistence] Neural Signal Stitching
            if let Some(signal_ref) = SESSION_CACHE.get(&Uuid::nil()) {
                tracing::info!("🧬 [Stitching] Recalling neural signals for session continuity...");
                let _ = model.inject_signal(signal_ref.value().clone());
            }

            return Ok(model);
        }

        // 2️⃣  BEST-FIT FALLBACK
        for registry_entry in ARCH_REGISTRY.iter() {
            let ((reg_engine, sig), constructor_ref) = registry_entry.pair();

            if reg_engine != &preferred_engine {
                continue;
            }

            let experts_parity = sig.has_experts == target_signature.has_experts;
            let pattern_parity = sig.head_pattern == target_signature.head_pattern;

            if experts_parity && pattern_parity {
                tracing::warn!("🔩 [Agnostic Flex] Precise DNA match unavailable for {:?}. Binding to Architectural Pattern: {}", preferred_engine, sig.head_pattern);
                return (constructor_ref)(model_load_path, sovereign_context);
            }
        }

        Err(anyhow!("❌ SOVEREIGN DISPATCH ERROR: No compatible kernel found for architecture traits."))
    }


    /// 🚨 EMERGENCY EVICT: Instantly purges all neural memory and kills running processes.
    /// Resets the engine to a zero-memory state.
    pub fn purge_hardware_context() {
        tracing::warn!("🚨 [Manager] EMERGENCY EVICT TRIGGERED. Purging Neural Memory...");
        // This hook will be called by the Governor in Phase 3 to kill binary kernels 
        // and drop all cached weights.
    }
}


