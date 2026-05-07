use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use reqwest;
use zip::ZipArchive;
use std::fs;
use std::io::Cursor;
use archer_shared::HardwareGovernor;

pub struct DriverProvisioner;

impl DriverProvisioner {
    const MANIFEST_URL: &'static str = "https://github.com/cluaiz/cluaiz/releases/download/drivers-v0.1.0/driver-manifest.json";
// https://cdn.jsdelivr.net/ 
    /// 🛠️ Provision Hardware Driver: Auto-detects, downloads, and extracts missing silicon drivers.
    pub async fn provision_for_hardware(driver_type: &str) -> Result<()> {
        let root = HardwareGovernor::resolve_hub_path();
        let driver_dir = root.join("interface-engines").join("drivers");
        
        if !driver_dir.exists() {
            fs::create_dir_all(&driver_dir)?;
        }

        // 🎯 Step 1: Check if driver already exists
        let marker = driver_dir.join(format!("{}.ready", driver_type));
        if marker.exists() {
            return Ok(());
        }

        println!("🛠️ [PROVISIONER] Missing Hardware Driver detected: {}. Initiating Silicon Handshake...", driver_type);

        // 🎯 Step 2: Fetch Driver Manifest (with User-Agent for GitHub/CDN compliance)
        let client = reqwest::Client::builder()
            .user_agent("Cluaiz-Neural-Engine/0.1.0")
            .build()?;

        let response = client.get(Self::MANIFEST_URL)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to connect to Cloud Foundry: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Cloud Foundry returned error {}: {}", response.status(), Self::MANIFEST_URL));
        }

        let manifest: serde_json::Value = response.json().await?;

        let download_url = manifest["drivers"][driver_type]
            .as_str()
            .ok_or_else(|| anyhow!("Driver '{}' not supported in the current Cluaiz Manifest.", driver_type))?;

        // 🛡️ Placeholder Guard: Don't download if the URL points to a text placeholder
        if download_url.ends_with(".txt") || download_url.ends_with(".md") {
            return Err(anyhow!("Silicon Handshake Aborted: Manifest contains placeholder URLs (.txt/.md) instead of binary kernels for '{}'. Please update your GitHub Release assets.", driver_type));
        }

        println!("📥 [PROVISIONER] Downloading {} driver from Cloud Foundry...", driver_type);

        // 🎯 Step 3: Download Driver Zip
        let response = client.get(download_url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download driver binary: {}", response.status()));
        }
        
        let bytes = response.bytes().await?;

        // 🎯 Step 4: Extract Zip
        println!("📦 [PROVISIONER] Extracting silicon kernels to drivers/ folder...");
        let mut archive = ZipArchive::new(Cursor::new(bytes))?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => driver_dir.join(path),
                None => continue,
            };

            if (*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(&p)?;
                    }
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        // 🎯 Step 5: Silicon Integrity Check (Ensure we actually got binaries, not just placeholders)
        let mut has_binaries = false;
        if let Ok(entries) = fs::read_dir(&driver_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "dll" || ext == "so" || ext == "dylib" || ext == "a" {
                        has_binaries = true;
                        break;
                    }
                } 
            }
        }

        if !has_binaries {
            fs::remove_file(&marker).ok(); // Remove marker so we can retry later
            return Err(anyhow!("Driver provisioning failed: The downloaded package contains no executable binaries (.dll/.so). Please check the GitHub Release assets."));
        }

        // 🎯 Step 6: Mark as Ready
        fs::write(marker, "PROVISIONED")?;
        println!("✅ [PROVISIONER] {} Hardware Driver successfully deployed to Bare-Metal.", driver_type);

        Ok(())
    }

    /// 🔍 Silicon Discovery: Scans the system for pre-installed drivers if local ones are missing.
    pub fn discover_system_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // Local drivers folder
        paths.push(Self::get_driver_path());

        // Windows CUDA Discovery (Dynamic via Environment Variables)
        #[cfg(target_os = "windows")]
        {
            // Standard NVIDIA environment variable
            if let Ok(cuda_path) = std::env::var("CUDA_PATH") {
                let bin_path = PathBuf::from(cuda_path).join("bin");
                if bin_path.exists() {
                    paths.push(bin_path);
                }
            }
        }

        paths
    }

    /// Returns the driver directory for appending to search paths
    pub fn get_driver_path() -> PathBuf {
        HardwareGovernor::resolve_hub_path().join("interface-engines").join("drivers")
    }
}
