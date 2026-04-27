//! ═══════════════════════════════════════════════════════════════════════
//!  CURE Engine: Master Archer Unified Neural Gateway
//! ═══════════════════════════════════════════════════════════════════════

// 🧬 Archer Sovereign Neural Core

pub mod api;
// pub mod kernel; // Removed in V7 DNA Refactor
pub mod runtime;
pub mod models;
pub mod memory;
pub mod hardware;
pub mod telemetry;
pub mod utils;
pub mod cli;
pub mod sync;
#[path = "interface-engines/mod.rs"]
pub mod interface_engines;

// ─── Master Archer Unified Access ────────────────────────────────────

// pub use kernel::core::CureKernel; // Removed in V7 DNA Refactor

// 2. Hardware & Architecture
pub use hardware::{HardwareDetector, SiliconTruth, SiliconTruth as HardwareInfo, InferenceEngine, InferenceEvent};
pub use hardware::system_control_manager::{detect_hardware, has_config, read_config, save_config, update_field};

// 3. Execution & Inference
pub use runtime::execution::runner::{SovereignRunner, SovereignMetrics};
pub use runtime::execution::sampler::NeuralSampler;
pub use runtime::execution::loader::GGUFLoader;

// 4. Intelligence & Registry
pub use models::entities::{ChatMessage, ChatRequest, ChatResponse, ChatSession, MessageRole};
pub use models::registry::{NeuralRoster, ModelManifest, ModelRecommendation, ModelAsset};
pub use models::fetch::{DownloadEvent, ModelDownloader};

// 5. Routing
pub use api::router::NeuralRouter;
pub use archer_shared::BackendType;
