use std::path::{Path, PathBuf};
use std::process::Command;
use archer_shared::HardwareGovernor;
use anyhow::{Result, anyhow};
use colored::Colorize;

pub struct Bootstrapper;

impl Bootstrapper {
    /// 🚀 SOVEREIGN BOOTSTRAP: Ensures the Neural Engine is present and initialized.
    pub async fn ignite() -> Result<()> {
        let control = HardwareGovernor::load_system_control()?;
        let root_path = PathBuf::from(&control.context.cluaiz_root);
        let engine_dir = root_path.join("engine");
        
        let engine_name = if cfg!(windows) { "cluaiz-engine.exe" } else { "cluaiz-engine" };
        let engine_path = engine_dir.join(engine_name);

        if !engine_path.exists() {
            println!("  {} [Sovereign] Neural Engine missing in cluaiz_root. Initiating retrieval...", "📡".blue());
            Self::download_engine(&engine_path).await?;
            Self::trigger_setup(&engine_path)?;
        } else {
            // Verify if system_control.bin exists, if not, trigger setup anyway
            let bin_truth = HardwareGovernor::resolve_base_path().join("interface-engines").join("system_control.bin");
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
            .map_err(|e| anyhow!("Failed to connect to Sovereign Registry: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Registry Error: Server returned status {}", response.status()));
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
            .map_err(|e| anyhow!("Failed to execute engine setup: {}", e))?;

        if !status.success() {
            return Err(anyhow!("Engine setup failed with status: {}", status));
        }

        Ok(())
    }

    fn resolve_engine_url() -> Result<String> {
        let version = "v0.1.0";
        let base = format!("https://github.com/cluaiz/cluaiz/releases/download/{}", version);

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return Ok(format!("{}/cluaiz-engine-win-x64.exe", base));

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return Ok(format!("{}/cluaiz-engine-linux-x64", base));

        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        return Ok(format!("{}/cluaiz-engine-linux-arm64", base));

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Ok(format!("{}/cluaiz-engine-mac-arm64", base));

        Err(anyhow!("Sovereign Registry: Platform not supported in this build."))
    }
}
