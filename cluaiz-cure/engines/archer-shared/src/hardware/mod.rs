//! 🏛️ Sovereign Silicon Kernel: The Architectural Heart
//! Enforces a strict 7-tier domain-driven model for hardware Agnosticism.
//! 
//! Structure:
//! 1. sensors        - OS Ground Truth logic (Direct OS Probing)
//! 2. hal            - The Bridge (Trait contracts & OS Routing)
//! 3. accelerators   - Hardware Units (CPU/GPU/NPU API wrappers)
//! 4. memory         - Resource Allocation (Paged Allocator/Monitor)
//! 5. bare_metal     - Deep Probes (Inline Assembly ISA checks)
//! 6. intelligence   - The Brain (Math, Goals, Telemetry, Scheduling)
//! 7. schema         - The Data (Pure Structs/Profiles)

pub mod schema;
pub mod sensors;
pub mod hal;
pub mod accelerators;
pub mod memory;
pub mod bare_metal;
pub mod intelligence;

// ── Re-exports for Zero-Latency Orchestration ──
pub use hal::{get_provider, SiliconProvider};
pub use schema::{SovereignProfile, SiliconMetrics, MemorySnapshot};
pub use intelligence::{GhostObserver, HardwareGovernor, GrandOrchestrator};
pub use intelligence::governor;
pub use intelligence::telemetry;
pub use intelligence::scheduler;
pub use intelligence::speed_checker;

/// 🏛️ The One-Call System State Facade
pub fn get_silicon_state() -> SovereignProfile {
    hal::detect_silicon()
}

pub fn get_live_metrics() -> SiliconMetrics {
    hal::capture_metrics()
}
