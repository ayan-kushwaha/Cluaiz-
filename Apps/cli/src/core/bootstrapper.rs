use std::path::Path;
use std::process::Command;
use cluaiz_shared::HardwareGovernor;
use color_eyre::{Result, eyre::eyre};
use colored::Colorize;

pub struct Bootstrapper;

impl Bootstrapper {
    /// 🚀 Cluaiz BOOTSTRAP: Ensures the Core Engine is present and initialized.
    pub async fn ignite() -> Result<()> {
        #[cfg(windows)]
        let _ = colored::control::set_virtual_terminal(true);

        let engine_dir = HardwareGovernor::resolve_engine_path();
        
        let ext = if cfg!(windows) { "dll" } else if cfg!(target_os = "macos") { "dylib" } else { "so" };
        let engine_name = format!("cluaiz-engine.{}", ext);
        let engine_path = engine_dir.join(engine_name);

        if !engine_path.exists() {
            println!("  {} [Cluaiz] Core Engine missing. Please run the Installer.", "🛠️".red());
            return Err(eyre!("Core Engine not found."));
        }

        // --- FULL STACK SYNC: Kernels & Drivers ---
        Self::sync_neural_stack().await?;

        // Verify if system_control.bin exists in Hub
        let bin_truth = HardwareGovernor::resolve_interface_path().join("system_control.bin");
        if !bin_truth.exists() {
            // Background calibration happens here
        }

        Ok(())
    }

    async fn download_engine(dest: &Path) -> Result<()> {
        let url = Self::resolve_engine_url()?;
        
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        println!("  {} [Cluaiz] Downloading Engine from: {}", "📥".cyan(), url);

        let response = reqwest::get(url).await
            .map_err(|e| eyre!("Failed to connect to Cluaiz Registry: {}", e))?;

        if !response.status().is_success() {
            return Err(eyre!("Registry Error: Server returned status {}", response.status()));
        }

        let content = response.bytes().await?;
        std::fs::write(dest, content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(dest)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(dest, perms)?;
        }

        println!("  {} [Cluaiz] Engine binary mounted and sealed.", "✅".green());
        Ok(())
    }

    fn trigger_setup(engine_path: &Path) -> Result<()> {
        println!("  {} [Cluaiz] Initializing hardware...", "⚙️".yellow());
        
        let status = Command::new(engine_path)
            .arg("--setup")
            .status()
            .map_err(|e| eyre!("Failed to execute engine setup: {}", e))?;

        if !status.success() {
            return Err(eyre!("Engine setup failed with status: {}", status));
        }

        Ok(())
    }

    async fn sync_neural_stack() -> Result<()> {
        let control_path = HardwareGovernor::resolve_engine_path().join("system_control.json");
        
        if !control_path.exists() {
            return Ok(()); // Wait for first calibration
        }

        let control_data: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&control_path)?)?;
        let has_nvidia = control_data["Hardware_truth"]["accelerators"]["gpus"]
            .as_array()
            .map(|gpus| gpus.iter().any(|g| g["vendor"].as_str() == Some("NVIDIA_CORP")))
            .unwrap_or(false);

        let backend = if has_nvidia { "cuda" } else { "cpu" };
        let platform = if cfg!(windows) { "win-x64" } else if cfg!(target_os = "macos") { "mac-arm64" } else { "linux-x64" };

        // 1. Kernel Sync
        let kernel_dir = HardwareGovernor::resolve_hub_path().join("interface-engines/kernels");
        let kernel_ext = if cfg!(windows) { "dll" } else if cfg!(target_os = "macos") { "dylib" } else { "so" };
        let kernel_name = format!("cluaiz-llama.{}", kernel_ext);
        let kernel_path = kernel_dir.join(&kernel_name);

        if !kernel_path.exists() {
            println!("  {} [Cluaiz] Downloading kernel ({})...", "📦".magenta(), backend);
            // 🛡️ Cluaiz DNA Sync: Match Workflow Naming DNA
            let version = cluaiz_shared::CluaizDNA::KERNEL; 
            let url = format!("https://github.com/cluaiz/cluaiz/releases/download/{}/cluaiz-llama-{}-{}-{}.{}", version, version, platform, backend, kernel_ext);
            Self::download_asset(&url, &kernel_path).await?;
        }

        // 2. Driver Sync (If needed)
        if has_nvidia {
            let driver_dir = HardwareGovernor::resolve_hub_path().join("interface-engines/drivers");
            let driver_tag = driver_dir.join("cuda.tag");
            if !driver_tag.exists() {
                println!("  {} [Cluaiz] Deploying Hardware Driver (CUDA)...", "🏎️".yellow());
                let version = cluaiz_shared::CluaizDNA::DRIVER;
                let url = format!("https://github.com/cluaiz/cluaiz/releases/download/{}/cluaiz-driver-cuda.zip", version);
                let zip_path = driver_dir.join("driver.zip");
                Self::download_asset(&url, &zip_path).await?;
                // TODO: Extraction logic
                std::fs::write(driver_tag, "ready")?;
            }
        }

        Ok(())
    }

    async fn download_asset(url: &str, dest: &Path) -> Result<()> {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let response = reqwest::get(url).await
            .map_err(|e| eyre!("Failed to connect to Cluaiz Registry: {}", e))?;

        if !response.status().is_success() {
            return Err(eyre!("Registry Error: {} returned {}", url, response.status()));
        }

        let content = response.bytes().await?;
        std::fs::write(dest, content)?;
        Ok(())
    }

    fn resolve_engine_url() -> Result<String> {
        let version = cluaiz_shared::CluaizDNA::ENGINE;
        let base = format!("https://github.com/cluaiz/cluaiz/releases/download/{}", version);

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return Ok(format!("{}/cluaiz-engine-{}-win-x64.dll", base, version));

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return Ok(format!("{}/cluaiz-engine-{}-linux-x64.so", base, version));

        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        return Ok(format!("{}/cluaiz-engine-{}-linux-arm64.so", base, version));

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Ok(format!("{}/cluaiz-engine-{}-mac-arm64.dylib", base, version));

        Err(eyre!("Cluaiz Registry: Platform not supported in this build."))
    }
}
