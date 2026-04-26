//! archer-shared: Common traits and types for the CURE Engine ecosystem.

pub mod hardware;
pub mod metadata;
pub mod prompting;
pub mod backend;
pub mod neural_core;
pub mod orchestrator;

// ── Business Logic (Unified from shared) ──
pub mod profile;
pub mod auth;
pub mod onboarding;
pub mod Chat;

pub use hardware::{governor::*, telemetry::*};
pub use metadata::dna::*;
pub use prompting::templater::*;
pub use backend::{context::*, traits::*, signature::*};
pub use neural_core::NeuralResult;
pub use orchestrator::*;
