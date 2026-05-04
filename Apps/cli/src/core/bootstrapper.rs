use std::path::Path;
use std::process::Command;
use archer_shared::HardwareGovernor;
use color_eyre::{Result, eyre::eyre};
use colored::Colorize;

pub struct Bootstrapper;

impl Bootstrapper {
    /// 🚀 SOVEREIGN BOOTSTRAP: Ensures the Neural Engine is present and initialized.
    pub async fn ignite() -> Result<()> {
        #[cfg(windows)]
        let _ = colored::control::set_virtual_terminal(true);

        let engine_dir = HardwareGovernor::resolve_engine_path();
        
        let ext = if cfg!(windows) { "dll" } else if cfg!(target_os = "macos") { "dylib" } else { "so" };
        let engine_name = format!("cluaiz-engine.{}", ext);
        let engine_path = engine_dir.join(engine_name);

        if !engine_path.exists() {
            println!("  {} [Sovereign] Neural Engine missing in Hub. Initiating retrieval...", "📡".blue());
            Self::download_engine(&engine_path).await?;
            // Setup logic for DLLs will be handled via libloading in the next phase
            // Self::trigger_setup(&engine_path)?; 
        } else {
            // Verify if system_control.bin exists in Hub, if not, trigger setup anyway
            let bin_truth = HardwareGovernor::resolve_interface_path().join("system_control.bin");
            if !bin_truth.exists() {
                println!("  {} [Sovereign] System Truth missing. Re-calibrating Silicon...", "🛠️".yellow());
                Self::trigger_setup(&engine_path)?;
            }
        }

        Ok(())
    }

    async fn download_engine(dest: &Path) -> Result<()> {
        let url = Self::resolve_engine_url()?;
        
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        println!("  {} [Sovereign] Downloading Engine from: {}", "📥".cyan(), url);

        let response = reqwest::get(url).await
            .map_err(|e| eyre!("Failed to connect to Sovereign Registry: {}", e))?;

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

        println!("  {} [Sovereign] Engine binary mounted and sealed.", "✅".green());
        Ok(())
    }

    fn trigger_setup(engine_path: &Path) -> Result<()> {
        println!("  {} [Sovereign] Igniting Hardware Calibration...", "🔥".red());
        
        let status = Command::new(engine_path)
            .arg("--setup")
            .status()
            .map_err(|e| eyre!("Failed to execute engine setup: {}", e))?;

        if !status.success() {
            return Err(eyre!("Engine setup failed with status: {}", status));
        }

        Ok(())
    }

    fn resolve_engine_url() -> Result<String> {
        let tag = "engine-v0.1.0";
        let base = format!("https://github.com/cluaiz/cluaiz/releases/download/{}", tag);

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return Ok(format!("{}/cluaiz-engine-dev-win-x64.dll", base));

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return Ok(format!("{}/cluaiz-engine-dev-linux-x64.so", base));

        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        return Ok(format!("{}/cluaiz-engine-dev-linux-arm64.so", base));

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Ok(format!("{}/cluaiz-engine-dev-mac-arm64.dylib", base));

        Err(eyre!("Sovereign Registry: Platform not supported in this build."))
    }
}
