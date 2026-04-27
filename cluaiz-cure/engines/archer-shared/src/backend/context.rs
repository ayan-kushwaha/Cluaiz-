use crate::metadata::dna::StructuralDNA;

use crate::hardware::HardwareGovernor;
use crate::prompting::templater::TemplateManager;


/// The core intelligence payload bridging system state and the kernel wrappers.
#[derive(Clone)]
pub struct SovereignContext {
    pub dna: StructuralDNA,
    pub governor: HardwareGovernor,
    pub templater: TemplateManager,
}

impl SovereignContext {
    /// Initialize a high-performance sovereign context
    pub fn boot(dna: StructuralDNA, templater: TemplateManager) -> Self {
        let governor = HardwareGovernor::start();
        
        Self {
            dna,
            governor,
            templater,
        }
    }
}
