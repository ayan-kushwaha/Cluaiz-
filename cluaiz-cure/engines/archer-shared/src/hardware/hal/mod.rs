pub mod provider;
pub mod platform_identity;
pub mod factory;

pub use provider::SiliconProvider;
pub use factory::get_provider;

/// 🏛️ Simplified System Facade
pub fn detect_silicon() -> super::schema::SovereignProfile {
    get_provider().detect_specs()
}

pub fn capture_metrics() -> super::schema::SiliconMetrics {
    get_provider().capture_metrics()
}
