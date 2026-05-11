use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use reqwest;
use std::fs;
use cluaiz_shared::HardwareGovernor;

pub struct DriverProvisioner;

impl DriverProvisioner {
    /// 🛠️ Construct Registry Key: Dynamically maps local hardware details to the registry's flat keys.
    fn get_registry_key(driver_type: &str) -> String {
        let platform = if cfg!(windows) { 
            "win-x64" 
        } else if cfg!(target_os = "macos") { 
            "mac-arm64" 
        } else if cfg!(target_os = "android") {
            "android-arm64"
        } else { 
            "linux-x64" 
        };

        // Handle specialized naming for CUDA versions and vendors
        match driver_type {
            "cuda" => {
                // Default to v12 for now, can be expanded to detect installed toolkit
                format!("{}-cuda-12", platform)
            },
            "rocm" | "hip" => format!("{}-{}", platform, driver_type),
            "vulkan" => format!("{}-vulkan", platform),
            "openvino" => format!("{}-openvino", platform),
            "cann" => format!("{}-cann", platform),
            "qnn" => format!("{}-qnn", platform),
            "metal" => format!("{}-metal", platform),
            _ => format!("{}-{}", platform, driver_type),
        }
    }

    /// 🛠️ Provision Hardware Driver: Auto-detects, downloads, and deploys missing or stale silicon drivers.
    pub async fn provision_for_hardware(driver_type: &str, manifest_url: &str) -> Result<()> {
        let root = HardwareGovernor::resolve_hub_path();
        let driver_dir = root.join("interface-engines").join("drivers");
        
        if !driver_dir.exists() {
            fs::create_dir_all(&driver_dir)?;
        }

        // 🎯 Step 1: Fetch Dynamic Driver Registry FIRST to check for updates
        let client = reqwest::Client::builder()
            .user_agent("Cluaiz-Neural-Engine/0.1.0")
            .build()?;

        let response = client.get(manifest_url).send().await
            .map_err(|e| anyhow!("Failed to connect to Cluaiz Registry: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Registry Error: Server returned status {}", response.status()));
        }

        let manifest: serde_json::Value = response.json().await
            .map_err(|e| anyhow!("Failed to parse driver manifest: {}", e))?;

        let manifest_version = manifest["version"].as_str().unwrap_or("unknown");

        // 🎯 Step 2: Check if driver already exists AND matches the latest version
        let marker = driver_dir.join(format!("{}.ready", driver_type));
        if marker.exists() {
            let local_version = fs::read_to_string(&marker).unwrap_or_default();
            if local_version == manifest_version {
                return Ok(()); // 100% Sync, skip download
            }
            println!("  {} [PROVISIONER] New Silicon Update detected: {} -> {}. Deploying...", "🚀".green(), local_version, manifest_version);
        } else {
            println!("  {} [PROVISIONER] Missing {} Silicon Driver detected. Initiating Sovereign Handshake...", "⚙️".yellow(), driver_type);
        }

        let registry_key = Self::get_registry_key(driver_type);

        // 🎯 Step 3: Resolve Download URL from Registry Key
        let download_url = manifest["drivers"][&registry_key]
            .as_str()
            .ok_or_else(|| anyhow!("Driver key '{}' not found in registry. OS/Hardware combination may be unsupported.", registry_key))?;

        // Extract filename from URL
        let dest_filename = download_url.split('/').last().unwrap_or("driver.bin");
        let dest_path = driver_dir.join(dest_filename);

        println!("  {} [PROVISIONER] Downloading silicon payload from Registry...", "📥".cyan());

        // 🎯 Step 4: Download Driver Binary
        let bin_response = client.get(download_url).send().await?;
        if !bin_response.status().is_success() {
            return Err(anyhow!("Failed to download driver binary: {} from {}", bin_response.status(), download_url));
        }
        
        let bytes = bin_response.bytes().await?;
        fs::write(&dest_path, bytes)?;

        // 🎯 Step 5: Silicon Integrity Check
        if !dest_path.exists() || fs::metadata(&dest_path)?.len() == 0 {
            return Err(anyhow!("Driver provisioning failed: Loaded file is empty or missing."));
        }

        // 🎯 Step 6: Mark as Ready with the manifest version
        fs::write(marker, manifest_version)?;
        println!("  {} [PROVISIONER] {} Hardware Driver successfully deployed to Bare-Metal.", "✅".green(), driver_type);

        Ok(())
    }

    /// 🔍 Silicon Discovery: Scans the system for pre-installed drivers if local ones are missing.
    pub fn discover_system_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        paths.push(Self::get_driver_path());

        #[cfg(target_os = "windows")]
        {
            if let Ok(cuda_path) = std::env::var("CUDA_PATH") {
                let bin_path = PathBuf::from(cuda_path).join("bin");
                if bin_path.exists() {
                    paths.push(bin_path);
                }
            }
        }
        paths
    }

    pub fn get_driver_path() -> PathBuf {
        HardwareGovernor::resolve_hub_path().join("interface-engines").join("drivers")
    }
}
