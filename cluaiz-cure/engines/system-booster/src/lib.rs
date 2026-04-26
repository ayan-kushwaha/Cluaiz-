//! ═══════════════════════════════════════════════════════════════════════
//!  CURE External Crate: System Booster (Bare Metal Isolator)
//! ═══════════════════════════════════════════════════════════════════════
//! This crate contains highly experimental, low-level (Assembly/Shell) 
//! hardware optimizations. It is isolated from the main engine so compile 
//! failures on unsupported OS architectures do not break the core system.

pub mod turbo_quant;
pub mod flash_attn;
pub mod speculative;
pub mod auto_round;
pub mod os_tuning;
pub mod telemetry;
pub mod neural_core;
