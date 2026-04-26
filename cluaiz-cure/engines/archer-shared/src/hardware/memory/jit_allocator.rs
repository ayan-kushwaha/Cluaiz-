use crate::hardware::SovereignProfile;

#[derive(Debug, PartialEq)]
pub enum ExecutionTier {
    Tier1Parallel,     // High-resource: parallel model loading + inference (Zero Latency)
    Tier2Sequential,   // Mid-resource: Drop -> Load -> Run pipeline to prevent OOM
    Tier3EdgeFallback, // Edge devices: extreme memory conservation mode
}

pub struct JitAllocator;

impl JitAllocator {
    /// Scale-agnostic execution tier selection.
    /// Uses percentage-based thresholds instead of fixed GB values
    /// so the logic works from 2GB Raspberry Pi to 128GB workstation.
    pub fn determine_execution_graph(profile: &SovereignProfile) -> ExecutionTier {
        let total_memory = profile.compute.vram_gb + profile.memory.total_ram_gb;
        let available_memory = profile.memory.free_ram_gb + profile.compute.vram_gb;
        
        // Percentage of total resources that are available
        let available_pct = if total_memory > 0.0 {
            (available_memory / total_memory) * 100.0
        } else {
            0.0
        };

        let tier = if available_pct > 60.0 && profile.storage.is_nvme {
            // >60% resources free + fast storage = safe for parallel loading
            ExecutionTier::Tier1Parallel
        } else if available_pct > 25.0 {
            // >25% resources free = sequential is safe
            ExecutionTier::Tier2Sequential
        } else {
            // <25% resources free = edge/conservation mode
            ExecutionTier::Tier3EdgeFallback
        };

        tracing::info!("🧠 [JIT] Execution Tier: {:?} (Available: {:.0}% of {:.1}GB)", tier, available_pct, total_memory);
        
        tier
    }
}
