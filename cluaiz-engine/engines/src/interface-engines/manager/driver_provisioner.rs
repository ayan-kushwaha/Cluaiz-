use std::path::PathBuf;
use anyhow::{Result, anyhow};
use reqwest;
use std::fs;
use cluaiz_shared::HardwareGovernor;
use colored::Colorize;

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

        match driver_type {
            "cuda" => format!("{}-cuda-12", platform),
            "rocm" | "hip" => format!("{}-{}", platform, driver_type),
            "vulkan" => format!("{}-vulkan", platform),
            "openvino" => format!("{}-openvino", platform),
            "cann" => format!("{}-cann", platform),
            "qnn" => format!("{}-qnn", platform),
            "metal" => format!("{}-metal", platform),
            _ => format!("{}-{}", platform, driver_type),
        }
    }

    /// 🛠️ Provision Kernel Binary: Auto-detects and deploys specialized engine kernels (llama-cuda, etc.)
    pub async fn provision_kernel(kernel_type: &str, backend: &str, manifest_url: &str) -> Result<PathBuf> {
        let root = HardwareGovernor::resolve_hub_path();
        let kernel_dir = root.join("interface-engines").join("kernels");
        
        if !kernel_dir.exists() {
            fs::create_dir_all(&kernel_dir)?;
        }

        let registry_key = Self::get_registry_key(backend);
        let binary_id = format!("{}-{}", kernel_type, backend);
        let marker = kernel_dir.join(format!("{}.ready", binary_id));

        let client = reqwest::Client::builder()
            .user_agent("Cluaiz-Neural-Engine/0.1.0")
            .build()?;

        let response = client.get(manifest_url).send().await
            .map_err(|e| anyhow!("Registry Sync Failed: {}", e))?;

        let manifest: serde_json::Value = response.json().await?;
        let manifest_version = manifest["version"].as_str().unwrap_or("unknown");

        if marker.exists() {
            let local_version = fs::read_to_string(&marker).unwrap_or_default();
            if local_version == manifest_version {
                let ext = if cfg!(windows) { "dll" } else if cfg!(target_os = "macos") { "dylib" } else { "so" };
                let p = kernel_dir.join(format!("cluaiz-{}.{}", binary_id, ext));
                if p.exists() { return Ok(p); }
            }
        }

        println!("  {} [PROVISIONER] Missing Neural Kernel '{}'. Provisioning from Registry...", "🧬".cyan(), binary_id);

        let download_url = manifest["kernel"][kernel_type][&registry_key]
            .as_str()
            .ok_or_else(|| anyhow!("Kernel '{}' for platform '{}' not found.", kernel_type, registry_key))?;

        let ext = if cfg!(windows) { "dll" } else if cfg!(target_os = "macos") { "dylib" } else { "so" };
        let dest_filename = format!("cluaiz-{}.{}", binary_id, ext);
        let dest_path = kernel_dir.join(dest_filename);

        let bin_response = client.get(download_url).send().await?;
        let bytes = bin_response.bytes().await?;
        fs::write(&dest_path, bytes)?;

        fs::write(marker, manifest_version)?;
        println!("  {} [PROVISIONER] Kernel '{}' successfully deployed.", "✅".green(), binary_id);

        Ok(dest_path)
    }

    /// 🛠️ Provision Hardware Driver: Auto-detects, downloads, and deploys missing or stale silicon drivers.
    pub async fn provision_for_hardware(driver_type: &str, manifest_url: &str) -> Result<()> {
        let root = HardwareGovernor::resolve_hub_path();
        let driver_dir = root.join("interface-engines").join("drivers");
        
        if !driver_dir.exists() {
            fs::create_dir_all(&driver_dir)?;
        }

        let client = reqwest::Client::builder().user_agent("Cluaiz-Neural-Engine/0.1.0").build()?;
        let response = client.get(manifest_url).send().await?;
        let manifest: serde_json::Value = response.json().await?;
        let manifest_version = manifest["version"].as_str().unwrap_or("unknown");

        let marker = driver_dir.join(format!("{}.ready", driver_type));
        if marker.exists() {
            let local_version = fs::read_to_string(&marker).unwrap_or_default();
            if local_version == manifest_version {
                return Ok(());
            }
        }

        println!("  {} [PROVISIONER] Provisioning Silicon Driver: {}...", "⚙️".yellow(), driver_type);
        let registry_key = Self::get_registry_key(driver_type);
        let download_url = manifest["drivers"][&registry_key].as_str()
            .ok_or_else(|| anyhow!("Driver key '{}' not found.", registry_key))?;

        let dest_filename = download_url.split('/').last().unwrap_or("driver.bin");
        let dest_path = driver_dir.join(dest_filename);

        let bin_response = client.get(download_url).send().await?;
        let bytes = bin_response.bytes().await?;
        fs::write(&dest_path, bytes)?;

        fs::write(marker, manifest_version)?;
        Ok(())
    }

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
