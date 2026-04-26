use super::provider::SiliconProvider;
use super::platform_identity::{detect, Environment};

/// Global Factory: Returns the optimal SiliconProvider for the current environment.
/// This is the ONLY place where platform-specific sensor modules are loaded.
pub fn get_provider() -> Box<dyn SiliconProvider> {
    let identity = detect();
    
    match identity.env {
        #[cfg(target_os = "windows")]
        Environment::Windows => Box::new(crate::hardware::sensors::windows_sensor::WindowsSensor::new()),
        
        #[cfg(target_os = "linux")]
        Environment::Linux | Environment::EdgePi | Environment::EdgeJetson | Environment::CloudGCP => {
            Box::new(crate::hardware::sensors::linux_sensor::LinuxSensor::new())
        },
        
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        Environment::MacOS | Environment::IOS => Box::new(crate::hardware::sensors::darwin_sensor::DarwinSensor::new()),
        
        // Android routing
        #[cfg(target_os = "android")]
        Environment::Android => Box::new(crate::hardware::sensors::android_sensor::AndroidSensor::new()),

        _ => panic!("Unsupported Sovereign Environment: {:?}", identity.env), // Safe panic at boot-time only, prevents runtime explosion
    }
}
