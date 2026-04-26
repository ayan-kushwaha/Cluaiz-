//! 🏛️ Silicon Kernel: NPU & TPU API Wrapper
//! Agnostic interface for Neural and Tensor processing units.
//! Dispatches to the active Platform Provider via HAL.

use super::super::hal::get_provider;
use super::super::schema::{NPUData, TPUData};

pub struct AcceleratorProbe;

impl AcceleratorProbe {
    pub fn new() -> Self {
        Self
    }

    /// Probes exactly NPU capabilities natively via HAL.
    pub fn probe_npu(&self) -> NPUData {
        get_provider().probe_npu()
    }

    /// Probes for Tensor Processing Units via HAL.
    pub fn probe_tpu(&self) -> TPUData {
        // [Architectural Alignment: TPU detection logic moved to sensors/hal]
        // Currently delegating to provider or returning default until HAL extension is complete.
        TPUData::default()
    }
}
