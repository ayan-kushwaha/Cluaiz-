use color_eyre::Result;
use colored::Colorize;
use cluaiz_shared::hardware::governor::HardwareGovernor;
use cluaiz_shared::hardware::schema::booster::{
    BoosterMode, KvCacheQuantization, ContextShiftingMode, FeatureState
};

pub async fn execute(
    kv_quant: Option<String>,
    context_shift: Option<String>,
    mode: Option<String>,
    spec_decode: Option<String>,
) -> Result<()> {
    let mut control = HardwareGovernor::load_booster_settings().unwrap_or_default();
    let mut modified = false;

    // Check if any arguments were provided
    let has_args = kv_quant.is_some() || context_shift.is_some() || mode.is_some() || spec_decode.is_some();

    if has_args {
        if let Some(m) = mode {
            control.mode_run = match m.to_lowercase().as_str() {
                "edge" => BoosterMode::Edge,
                "multitasking" => BoosterMode::Multitasking,
                "balance" => BoosterMode::Balance,
                "max_boost" => BoosterMode::MaxBoost,
                "ultra_max_boost" => BoosterMode::UltraMaxBoost,
                "hyper_cluster" => BoosterMode::HyperCluster,
                _ => {
                    println!("⚠️  Invalid mode '{}'. Keeping current value.", m);
                    control.mode_run
                }
            };
            modified = true;
        }

        if let Some(kv) = kv_quant {
            control.kv_cache_quantization = match kv.to_lowercase().as_str() {
                "auto" => KvCacheQuantization::Auto,
                "kv16" => KvCacheQuantization::Kv16,
                "kv8" => KvCacheQuantization::Kv8,
                "kv4" => KvCacheQuantization::Kv4,
                _ => {
                    println!("⚠️  Invalid KV quantization '{}'. Keeping current value.", kv);
                    control.kv_cache_quantization
                }
            };
            modified = true;
        }

        if let Some(cs) = context_shift {
            control.context_shifting = match cs.to_lowercase().as_str() {
                "auto" => ContextShiftingMode::Auto,
                "off" => ContextShiftingMode::Off,
                "minimal" => ContextShiftingMode::Minimal,
                "standard" => ContextShiftingMode::Standard,
                "aggressive" => ContextShiftingMode::Aggressive,
                "extreme" => ContextShiftingMode::Extreme,
                _ => {
                    println!("⚠️  Invalid context shifting mode '{}'. Keeping current value.", cs);
                    control.context_shifting
                }
            };
            modified = true;
        }

        if let Some(sd) = spec_decode {
            control.speculative_decoding = match sd.to_lowercase().as_str() {
                "auto" => FeatureState::Auto,
                "on" => FeatureState::On,
                "off" => FeatureState::Off,
                _ => {
                    println!("⚠️  Invalid speculative decoding mode '{}'. Keeping current value.", sd);
                    control.speculative_decoding
                }
            };
            modified = true;
        }
    } else {
        // Interactive configuration using inquire
        println!("\n  {} {}", "🚀".cyan(), "Cluaiz Booster - Interactive Performance Setup".bold());
        println!("  Configure core neural performance profiles and hardware budgets.\n");

        // 1. Mode selection
        let modes = vec![
            "balance (Recommended - Standard performance)",
            "multitasking (Laptop mode - respects system apps & RAM)",
            "edge (Mobile/NPU/Pi - extreme hardware constraints)",
            "max_boost (Workstation mode - maximizes GPU utilization)",
            "ultra_max_boost (Aggressively reclaims VRAM for models)",
            "hyper_cluster (Multi-GPU/Server node deployment)",
        ];
        
        let default_mode_idx = match control.mode_run {
            BoosterMode::Balance => 0,
            BoosterMode::Multitasking => 1,
            BoosterMode::Edge => 2,
            BoosterMode::MaxBoost => 3,
            BoosterMode::UltraMaxBoost => 4,
            BoosterMode::HyperCluster => 5,
        };

        let selected_mode_str = inquire::Select::new("Select execution performance mode:", modes)
            .with_starting_cursor(default_mode_idx)
            .prompt()?;

        control.mode_run = match selected_mode_str.split(' ').next().unwrap() {
            "balance" => BoosterMode::Balance,
            "multitasking" => BoosterMode::Multitasking,
            "edge" => BoosterMode::Edge,
            "max_boost" => BoosterMode::MaxBoost,
            "ultra_max_boost" => BoosterMode::UltraMaxBoost,
            "hyper_cluster" => BoosterMode::HyperCluster,
            _ => BoosterMode::Balance,
        };

        // 2. KV Quantization selection
        let kv_options = vec![
            "Auto (Recommended - Let system choose based on VRAM)",
            "Kv16 (High Quality - Raw Float16 precision)",
            "Kv8 (Optimized - 8-bit cache quantization, 50% size)",
            "Kv4 (Extreme - 4-bit cache quantization, 75% size)",
        ];

        let default_kv_idx = match control.kv_cache_quantization {
            KvCacheQuantization::Auto => 0,
            KvCacheQuantization::Kv16 => 1,
            KvCacheQuantization::Kv8 => 2,
            KvCacheQuantization::Kv4 => 3,
        };

        let selected_kv_str = inquire::Select::new("Select KV-Cache Quantization level:", kv_options)
            .with_starting_cursor(default_kv_idx)
            .prompt()?;

        control.kv_cache_quantization = match selected_kv_str.split(' ').next().unwrap() {
            "Auto" => KvCacheQuantization::Auto,
            "Kv16" => KvCacheQuantization::Kv16,
            "Kv8" => KvCacheQuantization::Kv8,
            "Kv4" => KvCacheQuantization::Kv4,
            _ => KvCacheQuantization::Auto,
        };

        // 3. Context Shifting selection
        let cs_options = vec![
            "Auto (Recommended - Standard shift buffer)",
            "Off (Disable sliding window, error on overflow)",
            "Minimal (Prune oldest 5% of tokens when cache is full)",
            "Standard (Prune oldest 10% of tokens when cache is full)",
            "Aggressive (Prune oldest 25% of tokens when cache is full)",
            "Extreme (Prune oldest 50% of tokens when cache is full)",
        ];

        let default_cs_idx = match control.context_shifting {
            ContextShiftingMode::Auto => 0,
            ContextShiftingMode::Off => 1,
            ContextShiftingMode::Minimal => 2,
            ContextShiftingMode::Standard => 3,
            ContextShiftingMode::Aggressive => 4,
            ContextShiftingMode::Extreme => 5,
        };

        let selected_cs_str = inquire::Select::new("Select Context Shifting (Sliding Window):", cs_options)
            .with_starting_cursor(default_cs_idx)
            .prompt()?;

        control.context_shifting = match selected_cs_str.split(' ').next().unwrap() {
            "Auto" => ContextShiftingMode::Auto,
            "Off" => ContextShiftingMode::Off,
            "Minimal" => ContextShiftingMode::Minimal,
            "Standard" => ContextShiftingMode::Standard,
            "Aggressive" => ContextShiftingMode::Aggressive,
            "Extreme" => ContextShiftingMode::Extreme,
            _ => ContextShiftingMode::Auto,
        };

        // 4. Speculative Decoding selection
        let sd_options = vec![
            "Auto (Hardware-aware dynamic routing - Recommended)",
            "On (Force Hybrid Speculative Execution)",
            "Off (Disable completely, pure single-token)",
        ];

        let default_sd_idx = match control.speculative_decoding {
            FeatureState::Auto => 0,
            FeatureState::On => 1,
            FeatureState::Off => 2,
        };

        let selected_sd_str = inquire::Select::new("Select Speculative Decoding (MTP/Eagle/Lookahead):", sd_options)
            .with_starting_cursor(default_sd_idx)
            .prompt()?;

        control.speculative_decoding = match selected_sd_str.split(' ').next().unwrap() {
            "Auto" => FeatureState::Auto,
            "On" => FeatureState::On,
            "Off" => FeatureState::Off,
            _ => FeatureState::Auto,
        };

        modified = true;
    }

    if modified {
        HardwareGovernor::save_booster_settings(&control)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to save booster settings: {}", e))?;
        println!("\n  {} {}", "✅".green(), "Booster Settings Synchronized Successfully!".bold());
    } else {
        println!("\n  {} {}", "📊".cyan(), "Current Booster Settings:".bold());
    }

    // Print a beautiful status summary
    let mode_desc = match control.mode_run {
        BoosterMode::Edge => "Edge (📱 Pi/Mobile optimization)",
        BoosterMode::Multitasking => "Multitasking (💻 Balanced Laptop profile)",
        BoosterMode::Balance => "Balance (⚖️ Default Performance)",
        BoosterMode::MaxBoost => "MaxBoost (🚀 High GPU Priority)",
        BoosterMode::UltraMaxBoost => "UltraMaxBoost (🔥 Maximum VRAM Reclamation)",
        BoosterMode::HyperCluster => "HyperCluster (🌌 Multi-GPU Server Cluster)",
    };

    let kv_desc = match control.kv_cache_quantization {
        KvCacheQuantization::Auto => "Auto (Quantized based on hardware constraints)",
        KvCacheQuantization::Kv16 => "Kv16 (Unquantized - Raw FP16 precision)",
        KvCacheQuantization::Kv8 => "Kv8 (8-bit Quantized - Saves ~50% KV memory)",
        KvCacheQuantization::Kv4 => "Kv4 (4-bit Quantized - Saves ~75% KV memory)",
    };

    let cs_desc = match control.context_shifting {
        ContextShiftingMode::Auto => "Auto (Standard 10% shift buffer)",
        ContextShiftingMode::Off => "Off (No sliding window, stops on overflow)",
        ContextShiftingMode::Minimal => "Minimal (5% shift buffer)",
        ContextShiftingMode::Standard => "Standard (10% shift buffer)",
        ContextShiftingMode::Aggressive => "Aggressive (25% shift buffer)",
        ContextShiftingMode::Extreme => "Extreme (50% shift buffer)",
    };

    let sd_desc = match control.speculative_decoding {
        FeatureState::Auto => "Auto (Native MTP / VRAM-Aware Fallback)",
        FeatureState::On => "On (Forced Speculative Execution)",
        FeatureState::Off => "Off (Single Token Generation)",
    };

    println!("    ├─ Mode:             {}", mode_desc.yellow());
    println!("    ├─ KV Cache:         {}", kv_desc.green());
    println!("    ├─ Spec. Decoding:   {}", sd_desc.magenta());
    println!("    └─ Context Shifting: {}\n", cs_desc.cyan());

    Ok(())
}
