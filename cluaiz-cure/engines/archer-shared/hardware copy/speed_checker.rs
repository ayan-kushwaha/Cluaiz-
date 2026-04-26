//! 🏛️ Silicon Kernel: Performance Prediction Engine
//! Calculates exact TPS projections and memory footprint dynamically.

use super::mod_types::SovereignProfile;

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    GodMode,     // 🟣 > 100 t/s
    HyperSpeed,  // 🔵 50-100 t/s
    Instant,     // 🟢 20-50 t/s
    Moderate,    // 🟡 10-20 t/s
    Lagging,     // 🟠 5-10 t/s
    Critical,    // 🔴 < 5 t/s
    Panic,       // ⚫ Out of Memory
}

impl HealthStatus {
    pub fn to_discord_icon(&self) -> &'static str {
        match self {
            Self::GodMode => "🟣",
            Self::HyperSpeed => "🔵",
            Self::Instant => "🟢",
            Self::Moderate => "🟡",
            Self::Lagging => "🟠",
            Self::Critical => "🔴",
            Self::Panic => "⚫",
        }
    }
}

pub struct SpeedReport {
    pub expected_tps: f64,
    pub active_memory_gb: f64,
    pub status: HealthStatus,
    pub can_load: bool,
}

pub fn predict_performance(
    parameters_str: &str,
    bit_depth: f64,
    context_window: &str,
    requires_gpu: bool,
    profile: &SovereignProfile,
) -> SpeedReport {
    // 1. Extract Parameters (Billions) - Robust parser for "Effective 2B", "1.7B" etc.
    let mut params_b = 0.0;
    let p_clean = parameters_str.to_lowercase();
    let num_str: String = p_clean.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    if let Ok(val) = num_str.parse::<f64>() {
        params_b = val;
    }
    
    // Safety clamp (prevent inf T/s)
    if params_b <= 0.0 { params_b = 1.0; }

    // 2. Exact Physical Footprint Math
    let base_model_weight_gb = params_b * (bit_depth / 8.0);
    
    // 3. True KV Cache Context memory Load
    let context_str: String = context_window.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    let context_k = context_str.parse::<f64>().unwrap_or(8.0);
    
    // Context footprint logic: 32k context on an 8B model takes ~1GB at fp16.  
    let kv_cache_gb = (context_k / 32.0) * (params_b / 7.0) * 1.0;
    
    // Some minor overhead for engine operations (~100MB)
    let active_memory_gb = base_model_weight_gb + kv_cache_gb + 0.10;
    
    let total_sys_mem = profile.memory.total_ram_gb;
    
    // Panic check 1 (Memory)
    if active_memory_gb > (total_sys_mem * 0.95) {
         return SpeedReport {
             expected_tps: 0.0,
             active_memory_gb,
             status: HealthStatus::Panic,
             can_load: false,
         };
    }
    
    // Panic check 2 (Hardware Requirement Validation)
    if requires_gpu && profile.compute.vram_gb <= 0.0 {
         return SpeedReport {
             expected_tps: 0.0,
             active_memory_gb,
             status: HealthStatus::Panic,
             can_load: false,
         };
    }
    
    // 4. True Bandwidth Split & Hybrid Offloading Logic
    let mut expected_tps = 0.0;
    
    if profile.compute.vram_gb > 0.0 {
        let vram = profile.compute.vram_gb;
        let gpu_bw = 350.0; // GDDR6+ Simulation
        let cpu_bw = profile.memory.bw_gbps;
        
        if active_memory_gb <= vram {
            // Scenario A: 100% VRAM Fit
            expected_tps = gpu_bw / active_memory_gb;
        } else {
            // Scenario B: Hybrid Offload
            let vram_used = vram;
            let ram_used = active_memory_gb - vram;
            
            // Sequential Math: Weakest Link Pipeline
            let time_gpu = vram_used / gpu_bw;
            let time_cpu = ram_used / cpu_bw;
            let total_time = time_gpu + time_cpu;
            
            if total_time > 0.0 { expected_tps = 1.0 / total_time; }
            
            // STRICT RULE: If Hybrid Offloading crashes speed to < 5.0 TPS, Block it.
            if expected_tps < 5.0 {
                return SpeedReport {
                    expected_tps: 0.0,
                    active_memory_gb,
                    status: HealthStatus::Panic,
                    can_load: false,
                };
            }
        }
    } else {
        // CPU-Only Pipeline
        let cpu_bw = profile.memory.bw_gbps; 
        expected_tps = cpu_bw / active_memory_gb;
    }
    
    // CPU penalty heuristic (if memory is plenty but cores are few)
    if profile.cpu_cores <= 4 && expected_tps > 15.0 {
        expected_tps *= 0.6; 
    }
    
    let status = if expected_tps > 100.0 { HealthStatus::GodMode }
                 else if expected_tps >= 50.0 { HealthStatus::HyperSpeed }
                 else if expected_tps >= 20.0 { HealthStatus::Instant }
                 else if expected_tps >= 10.0 { HealthStatus::Moderate }
                 else if expected_tps >= 3.0 { HealthStatus::Lagging }
                 else { HealthStatus::Critical };
                 
    SpeedReport {
        expected_tps,
        active_memory_gb,
        status,
        can_load: true,
    }
}
