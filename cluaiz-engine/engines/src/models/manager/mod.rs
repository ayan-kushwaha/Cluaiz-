use std::path::PathBuf;
use colored::Colorize;
use crate::models::manager::client::RegistryClient;
use crate::models::manager::installer::ModelInstaller;
use crate::models::manager::auditor::{HardwareAuditor, HealthStatus};

pub mod client;
pub mod installer;
pub mod auditor;

/// The Cluaiz Model Manager
/// Responsible for model discovery, health auditing, and atomic installation/repair.
pub struct ModelManager {
    client: RegistryClient,
    installer: ModelInstaller,
    auditor: HardwareAuditor,
    base_models_dir: PathBuf,
}

impl ModelManager {
    pub fn new(registry_url: String, base_models_dir: PathBuf) -> Self {
        Self {
            client: RegistryClient::new(registry_url),
            installer: ModelInstaller::new(base_models_dir.clone()),
            auditor: HardwareAuditor,
            base_models_dir,
        }
    }

    /// Installation & Repair: Pull a specific model by its Unified ID (e.g., bonsai:8b)
    pub async fn pull_model(&self, model_id: &str) -> Result<(), String> {
        // 1. Resolve Metadata
        let roster = crate::models::registry::CoreRoster::load_roster();
        let mut manifest = roster.into_iter().find(|m| m.id.to_lowercase() == model_id.to_lowercase());

        if manifest.is_none() {
            let remote_models = crate::models::registry::CoreRoster::fetch_external_registry(None).await?;
            manifest = remote_models.into_iter().find(|m| m.id.to_lowercase() == model_id.to_lowercase());
        }

        let manifest = manifest.ok_or_else(|| format!("ID '{}' not found in any registry.", model_id))?;

        // 2. Hardware Audit
        let status = self.audit_model_health(manifest.ram_required_gb as f32, manifest.requires_gpu);
        if status == HealthStatus::Disabled {
            return Err("Cluaiz Audit Failed: Insufficient hardware resources for this model.".to_string());
        }

        // 3. Construct Path
        let safe_id = manifest.id.replace(':', "-");
        let mut model_path = self.base_models_dir.clone();
        model_path.push(&manifest.category);
        model_path.push(&safe_id);

        tokio::fs::create_dir_all(&model_path).await
            .map_err(|e| format!("Failed to create model directory: {}", e))?;

        // 4. Initialize Installer
        let installer = ModelInstaller::new(model_path.clone());

        // 5. Check Weights & Assets (The Surgical Repair)
        let weight_file = model_path.join(&manifest.huggingface_filename);
        let dna_file = model_path.join("structural_dna.json");

        let mut needs_repair = !weight_file.exists() || !dna_file.exists();
        
        // Check if any asset is missing
        for asset in &manifest.assets {
            if !model_path.join(&asset.name).exists() {
                needs_repair = true;
                break;
            }
        }

        if !needs_repair {
            println!("  {} Model '{}' is healthy and ready.", "✅".green(), manifest.id);
            return Ok(());
        }

        // 6. Pull Missing Weights
        if !weight_file.exists() {
            installer.download_weights(&manifest.download_url, &manifest.huggingface_filename).await?;
        } else {
            println!("  {} Weights verified.", "✅".green());
        }

        // 7. Pull Missing Assets
        let asset_pairs: Vec<(String, String)> = manifest.assets.clone().into_iter()
            .map(|a| (a.name, a.url))
            .collect();
        installer.pull_assets(asset_pairs).await?;

        // 8. Save/Refresh local manifest
        let local_manifest_path = model_path.join("model_manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("JSON Serialize Error: {}", e))?;
        tokio::fs::write(local_manifest_path, manifest_json).await
            .map_err(|e| format!("Failed to save local manifest: {}", e))?;

        // 9. 🧬 Neural DNA Handshake (Always ensure DNA is fresh)
        let _ = crate::models::fetch::ModelDownloader::generate_Cluaiz_dna(&manifest, &model_path, &weight_file);

        println!("  {} Model '{}' synchronized and ready.\n", "✅".green(), manifest.id);
        Ok(())
    }

    pub fn audit_model_health(&self, ram_required: f32, requires_gpu: bool) -> HealthStatus {
        self.auditor.audit_performance(ram_required, requires_gpu)
    }
}
