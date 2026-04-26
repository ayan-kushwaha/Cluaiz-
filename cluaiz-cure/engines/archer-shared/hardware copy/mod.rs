// 🏛️ Sovereign Platform Kernel: Architectural Backbone
pub mod platform;
pub mod provider;
pub mod mod_types; // Shared data structures
pub use mod_types::{SovereignProfile, SiliconMetrics};
pub mod governor;
pub mod benchmark;
pub mod speed_checker;

// Platform-Specific Modules (Conditional Compilation)
#[cfg(target_os = "windows")]
pub mod windows_sensor;

#[cfg(target_os = "linux")]
pub mod linux_sensor;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub mod darwin_sensor;

// High-Level Agnostic Wrappers
pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod telemetry;
pub mod hal;
pub mod isa_probe;
pub mod scheduler;

use self::provider::SiliconProvider;
use self::platform::Environment;

/// Global Factory: Returns the optimal SiliconProvider for the current environment.
/// This is the ONLY place where platform-specific detection occurs (Single Source of Truth).
pub fn get_provider() -> Box<dyn SiliconProvider> {
    let identity = platform::detect();
    
    match identity.env {
        #[cfg(target_os = "windows")]
        Environment::Windows => Box::new(windows_sensor::WindowsSensor::new()),
        
        #[cfg(target_os = "linux")]
        Environment::Linux | Environment::EdgePi | Environment::EdgeJetson | Environment::CloudGCP => {
            Box::new(linux_sensor::LinuxSensor::new())
        },
        
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        Environment::MacOS | Environment::IOS => Box::new(darwin_sensor::DarwinSensor::new()),
        
        _ => panic!("Unsupported Sovereign Environment: {:?}", identity.env),
    }
}
