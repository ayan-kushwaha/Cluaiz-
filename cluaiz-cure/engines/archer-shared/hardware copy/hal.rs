//! 🏛️ Silicon Kernel: Hardware Abstraction Layer (HAL)
//! Professional entry point for the Silicon Kernel. 
//! 100% DRY Compliance: Dispatches to the Sovereign Platform Provider.

use super::{get_provider, mod_types::SovereignProfile};

pub fn detect_silicon() -> SovereignProfile {
    get_provider().detect_specs()
}

pub fn capture_metrics() -> super::mod_types::SiliconMetrics {
    get_provider().capture_metrics()
}
