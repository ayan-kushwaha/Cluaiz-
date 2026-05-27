use std::path::Path;
use cluaiz_shared::HardwareGovernor;
use color_eyre::{Result, eyre::eyre};
use colored::Colorize;

pub struct Bootstrapper;

impl Bootstrapper {
    const MASTER_REGISTRY_URL: &'static str = "https://raw.githubusercontent.com/cluaiz/cluaiz/main/package.json";

    /// 🚀 Cluaiz BOOTSTRAP: The Sovereign Handshake.
    pub async fn ignite() -> Result<()> {
        #[cfg(debug_assertions)]
        let _ = Self::sync_dev_artifacts();

        #[cfg(windows)]
        let _ = colored::control::set_virtual_terminal(true);

        let bin_dir = HardwareGovernor::resolve_hub_path().join("bin");
        let local_registry_path = bin_dir.join("package.json");
        
        let client = reqwest::Client::builder()
            .user_agent("Cluaiz-Bootstrapper/0.1.0")
            .build()?;

        // 🎯 1. Fetch Master Registry (package.json)
        tracing::info!("🛰️ [Cluaiz] Synchronizing Neural Registry...");
        let master_registry: serde_json::Value = client.get(Self::MASTER_REGISTRY_URL).send().await?.json().await?;
        
        // 🏛️ Seal the Master Registry with Atomic Write Protocol
        cluaiz_shared::RegistryGovernor::seal_registry(master_registry.clone())
            .map_err(|e| eyre!("Binary Truth Seal Error: {}", e))?;
        tracing::debug!("✅ [Registry] Binary Truth sealed and verified.");

        // 🎯 2. CLI Lifecycle Check
        let cli_info = &master_registry["components"]["cli"];
        let latest_cli = cli_info["version"].as_str().unwrap_or("");
        let current_cli = env!("CARGO_PKG_VERSION");
        
        if latest_cli != current_cli && !latest_cli.is_empty() {
            println!("  {} [Cluaiz] Update Available: {} -> {}", "🚀".green(), current_cli, latest_cli);
        }

        // 🎯 3. Engine Sync (Driven by package.json)
        let engine_info = &master_registry["components"]["engine"];
        let engine_dir = HardwareGovernor::resolve_engine_path();
        let ext = if cfg!(windows) { "dll" } else if cfg!(target_os = "macos") { "dylib" } else { "so" };
        let engine_path = engine_dir.join(format!("cluaiz-engine.{}", ext));
        let engine_marker = engine_dir.join("cluaiz-engine.ready");

        let manifest_version = engine_info["version"].as_str().unwrap_or("unknown");
        let local_version = std::fs::read_to_string(&engine_marker).unwrap_or_default();

        if !engine_path.exists() || local_version != manifest_version {
            println!("  {} [Cluaiz] Provisioning Core Engine ({})...", "⚙️".yellow(), manifest_version);
            let manifest_url = engine_info["manifest_url"].as_str().ok_or_else(|| eyre!("Engine Manifest URL missing."))?;
            let engine_manifest: serde_json::Value = client.get(manifest_url).send().await?.json().await?;
            
            Self::download_engine_with_manifest(&engine_path, &engine_manifest).await?;
            std::fs::write(&engine_marker, manifest_version)?;
        }

        // 🎯 4. Kernel & Stack Sync
        Self::sync_neural_stack(&master_registry).await?;

        // Cache the master registry locally for offline reference
        if let Some(parent) = local_registry_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(local_registry_path, serde_json::to_string_pretty(&master_registry)?)?;

        Ok(())
    }

    async fn download_engine_with_manifest(dest: &Path, manifest: &serde_json::Value) -> Result<()> {
        let platform = if cfg!(windows) { "win-x64" } else if cfg!(target_os = "macos") { "mac-arm64" } else { "linux-x64" };
        
        let url_option = manifest["engines"][platform].as_str();
        
        if let Some(url) = url_option {
            Self::download_asset(url, dest).await?;
        } else if dest.exists() {
            tracing::warn!("⚠️ [Bootstrapper] Platform '{}' not found in Engine Registry, but local engine exists. Skipping download.", platform);
        } else {
            return Err(eyre!("Platform '{}' not found in Engine Registry and no local engine found.", platform));
        }
        
        Ok(())
    }

    async fn sync_neural_stack(master_registry: &serde_json::Value) -> Result<()> {
        let control_path = HardwareGovernor::resolve_engine_path().join("system_control.json");
        if !control_path.exists() {
            return Ok(());
        } 

        let control_data: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&control_path)?)?;
        let has_nvidia = control_data["silicon_truth"]["accelerators"]["gpus"]
            .as_array()
            .map(|gpus| gpus.iter().any(|g| g["vendor"].as_str().map(|v| v.to_uppercase()).unwrap_or_default().contains("NVIDIA")))
            .unwrap_or(false);

        let platform = if cfg!(windows) { "win-x64" } else if cfg!(target_os = "macos") { "mac-arm64" } else { "linux-x64" };

        // Kernel Sync (Version-Aware via package.json)
        let kernel_info = &master_registry["components"]["kernel"];
        let kernel_dir = HardwareGovernor::resolve_hub_path().join("interface-engines/kernels");
        let kernel_ext = if cfg!(windows) { "dll" } else if cfg!(target_os = "macos") { "dylib" } else { "so" };
        let kernel_path = kernel_dir.join(format!("cluaiz-llama.{}", kernel_ext));
        let kernel_marker = kernel_dir.join("cluaiz-llama.ready");

        let manifest_version = kernel_info["version"].as_str().unwrap_or("unknown");
        let local_version = std::fs::read_to_string(&kernel_marker).unwrap_or_default();

        if !kernel_path.exists() || local_version != manifest_version {
            println!("  {} [Cluaiz] Synchronizing Neural Kernel ({})...", "📦".magenta(), manifest_version);
            let client = reqwest::Client::builder().user_agent("Cluaiz-Bootstrapper/0.1.0").build()?;
            let manifest_url = kernel_info["manifest_url"].as_str().ok_or_else(|| eyre!("Kernel Manifest URL missing."))?;
            let manifest: serde_json::Value = client.get(manifest_url).send().await?.json().await?;

            let mut spec_key = platform.to_string();
            if platform == "win-x64" || platform == "linux-x64" {
                let isa_features = control_data["silicon_truth"]["cpu"]["isa_features"].as_array();
                let has_avx512 = isa_features.map(|feats| feats.iter().any(|f| f.as_str() == Some("AVX-512"))).unwrap_or(false);
                spec_key = if has_avx512 { format!("{}-avx512", platform) } else { format!("{}-avx2", platform) };
            }

            let url_option = manifest["kernels"][&spec_key].as_str().or_else(|| manifest["kernels"][platform].as_str());
            
            if let Some(url) = url_option {
                Self::download_asset(url, &kernel_path).await?;
            } else if kernel_path.exists() {
                tracing::warn!("⚠️ [Bootstrapper] Kernel key '{}' not found in registry, but local kernel exists. Skipping.", spec_key);
            } else {
                return Err(eyre!("Kernel key '{}' not found and no local kernel found.", spec_key));
            }
            std::fs::write(&kernel_marker, manifest_version)?;
        }

        if has_nvidia {
            let driver_manifest_url = master_registry["components"]["drivers"]["manifest_url"].as_str().unwrap_or_default();
            let _ = engines::interface_engines::manager::driver_provisioner::DriverProvisioner::provision_for_hardware("cuda", driver_manifest_url).await;
        }

        Ok(())
    }

    async fn download_asset(url: &str, dest: &Path) -> Result<()> {
        if let Some(parent) = dest.parent() { std::fs::create_dir_all(parent)?; }
        let client = reqwest::Client::builder().user_agent("Cluaiz-Bootstrapper/0.1.0").danger_accept_invalid_certs(true).build()?;
        let response = client.get(url).send().await.map_err(|e| eyre!("Registry Link Error: {}", e))?;
        if !response.status().is_success() { return Err(eyre!("Registry Error: {} returned {}", url, response.status())); }
        let content = response.bytes().await?;
        std::fs::write(dest, content)?;
        Ok(())
    }

    /// 🛠️ Dev-Sync: Synchronizes local build artifacts to .cluaiz during development.
    /// This ensures cargo run always uses the latest compiled binaries.
    #[cfg(debug_assertions)]
    fn sync_dev_artifacts() -> Result<()> {
        let hub_path = cluaiz_shared::HardwareGovernor::resolve_hub_path();
        
        // Robust workspace root lookup by traversing parent directories
        let mut project_root = None;
        let mut path = std::env::current_dir().unwrap_or_default();
        for _ in 0..5 {
            if path.join("Apps").exists() && path.join("interface-engines").exists() {
                project_root = Some(path.clone());
                break;
            }
            if let Some(parent) = path.parent() {
                path = parent.to_path_buf();
            } else {
                break;
            }
        }
        
        if let Some(root) = project_root {
            let target_dir = root.join("target").join("debug");
            let ext = if cfg!(windows) { "dll" } else if cfg!(target_os = "macos") { "dylib" } else { "so" };

            // 1. Engine Sync (engines.dll -> cluaiz-engine.dll)
            let engine_src = target_dir.join(format!("engines.{}", ext));
            let engine_dest = hub_path.join("engine").join(format!("cluaiz-engine.{}", ext));
            
            if engine_src.exists() {
                let _ = std::fs::create_dir_all(engine_dest.parent().unwrap());
                if let Err(e) = std::fs::copy(&engine_src, &engine_dest) {
                    tracing::warn!("⚠️ [DevSync] Engine Link Failed: {}. (File might be locked by another process)", e);
                } else {
                    let _ = std::fs::write(hub_path.join("engine").join("cluaiz-engine.ready"), "dev-release");
                    tracing::info!("🧬 [DevSync] Engine Linked: {:?}", engine_dest);
                }
            }

            // 2. Kernel Sync (cluaiz_llama.dll -> cluaiz-llama.dll or cluaiz-llama-cuda.dll)
            let kernel_src = target_dir.join(format!("cluaiz_llama.{}", ext));
            let control_path = cluaiz_shared::HardwareGovernor::resolve_engine_path().join("system_control.json");
            let has_nvidia = if control_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&control_path) {
                    if let Ok(control_data) = serde_json::from_str::<serde_json::Value>(&content) {
                        control_data["silicon_truth"]["accelerators"]["gpus"]
                            .as_array()
                            .map(|gpus| gpus.iter().any(|g| g["vendor"].as_str().map(|v| v.to_uppercase()).unwrap_or_default().contains("NVIDIA")))
                            .unwrap_or(false)
                    } else { false }
                } else { false }
            } else { false };

            let kernel_name = if has_nvidia { "cluaiz-llama-cuda" } else { "cluaiz-llama" };
            let kernel_dest = hub_path.join("interface-engines").join(format!("{}.{}", kernel_name, ext));
            let kernel_dest_subdir = hub_path.join("interface-engines").join("kernels").join(format!("{}.{}", kernel_name, ext));
            
            if kernel_src.exists() {
                // Copy to both root and kernels subdirectory to ensure loader finds it
                let _ = std::fs::create_dir_all(kernel_dest.parent().unwrap());
                let _ = std::fs::create_dir_all(kernel_dest_subdir.parent().unwrap());
                
                if let Err(e) = std::fs::copy(&kernel_src, &kernel_dest) {
                    tracing::warn!("⚠️ [DevSync] Kernel Link Failed (root): {}.", e);
                } else {
                    let _ = std::fs::write(hub_path.join("interface-engines").join(format!("{}.ready", kernel_name)), "dev-release");
                    tracing::info!("🧬 [DevSync] Kernel Linked (root): {:?}", kernel_dest);
                }
                
                if let Err(e) = std::fs::copy(&kernel_src, &kernel_dest_subdir) {
                    tracing::warn!("⚠️ [DevSync] Kernel Link Failed (subdir): {}.", e);
                } else {
                    let _ = std::fs::write(hub_path.join("interface-engines").join("kernels").join(format!("{}.ready", kernel_name)), "dev-release");
                    tracing::info!("🧬 [DevSync] Kernel Linked (subdir): {:?}", kernel_dest_subdir);
                }
            } else {
                tracing::warn!("⚠️ [DevSync] Kernel source not found at: {:?}", kernel_src);
            }
        } else {
            tracing::error!("❌ [DevSync] Failed to resolve project root. Binaries will not be synced.");
        }

        Ok(())
    }

    #[cfg(not(debug_assertions))]
    fn sync_dev_artifacts() -> Result<()> {
        Ok(())
    }
}
