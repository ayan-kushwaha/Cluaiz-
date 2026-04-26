//! 🏛️ Silicon Kernel: Platform Intelligence
//! Single Source of Truth for Environment, OS, and Architecture detection.
//! Satisfies the DRY (Don't Repeat Yourself) protocol V7.4.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Windows,
    Linux,
    MacOS,
    Android,
    IOS,
    CloudGCP,
    CloudAWS,
    EdgePi,
    EdgeJetson,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuFeatures {
    pub avx2: bool,
    pub avx512: bool,
    pub amx: bool, // Intel Advanced Matrix Extensions
    pub neon: bool, // ARM NEON
    pub sve: bool,  // ARM Scalable Vector Extension
    pub dotprod: bool, // ARM Dot Product
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformIdentity {
    pub env: Environment,
    pub arch: String,
    pub features: CpuFeatures,
    pub is_headless: bool,
    pub is_unified_memory: bool,
}

/// Detects the current operating environment once at boot.
pub fn detect() -> PlatformIdentity {
    let mut env = Environment::Generic;
    let mut is_unified_memory = false;

    // 1. OS & Device Specific Detection
    if cfg!(target_os = "windows") {
        env = Environment::Windows;
    } else if cfg!(target_os = "macos") {
        env = Environment::MacOS;
        is_unified_memory = true; // Apple Silicon/Intel Mac Unified assumption
    } else if cfg!(target_os = "ios") {
        env = Environment::IOS;
        is_unified_memory = true;
    } else if cfg!(target_os = "android") {
        env = Environment::Android;
        is_unified_memory = true;
    } else if cfg!(target_os = "linux") {
        // Linux Sub-detection (Pi, Jetson, Cloud)
        if std::path::Path::new("/proc/device-tree/model").exists() {
            env = Environment::EdgePi;
            is_unified_memory = true;
        } else if std::path::Path::new("/etc/nv_tegra_release").exists() {
            env = Environment::EdgeJetson;
            is_unified_memory = true;
        } else if std::env::var("TPU_NAME").is_ok() {
            env = Environment::CloudGCP;
        } else {
            env = Environment::Linux;
        }
    }

    // 2. ISA / SIMD Runtime Probing (Bare-Metal Mastery)
    let mut features = CpuFeatures::default();

    #[cfg(target_arch = "x86_64")]
    {
        features.avx2 = std::is_x86_feature_detected!("avx2");
        features.avx512 = std::is_x86_feature_detected!("avx512f");
        // AMX tile is unstable on standard rustc, skipping for now:
        features.amx = false;
    }

    #[cfg(target_arch = "aarch64")]
    {
        features.neon = true; // Always true on Aarch64
        // Logic for SVE/DotProd usually via /proc/cpuinfo or auxv on Linux
        features.dotprod = std::path::Path::new("/proc/cpuinfo")
            .map(|_| true) // Placeholder for real regex-based flag check
            .unwrap_or(false);
    }

    PlatformIdentity {
        env,
        arch: std::env::consts::ARCH.to_string(),
        features,
        is_headless: std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err(),
        is_unified_memory,
    }
}
