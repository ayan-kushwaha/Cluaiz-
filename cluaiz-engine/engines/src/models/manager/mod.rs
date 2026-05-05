use std::path::PathBuf;
use crate::models::manager::client::RegistryClient;
use crate::models::manager::installer::ModelInstaller;
use crate::models::manager::auditor::{HardwareAuditor, HealthStatus};
use crate::models::registry::ModelManifest;

pub mod client;
pub mod installer;
pub mod auditor;

/// The Cluaiz Model Manager
/// Responsible for model discovery, health auditing, and atomic installation.
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

    /// Discovery: Fetch the remote index of available models
    pub async fn list_remote_models(&self) -> Result<String, String> {
        self.client.fetch_index().await
    }

    /// Installation: Pull a specific model by its Unified ID (e.g., bonsai:8b)
    pub async fn pull_model(&self, model_id: &str) -> Result<(), String> {
        // 1. Resolve Metadata: Local Library -> Sovereign Registry (jsDelivr)
        let roster = crate::models::registry::CoreRoster::load_roster();
        let mut manifest = roster.into_iter().find(|m| m.id.to_lowercase() == model_id.to_lowercase());

        if manifest.is_none() {
            println!("🌐 [Manager] ID not found locally. Fetching Sovereign Registry...");
            let remote_models = crate::models::registry::CoreRoster::fetch_external_registry(None).await?;
            manifest = remote_models.into_iter().find(|m| m.id.to_lowercase() == model_id.to_lowercase());
        }

        let manifest = manifest.ok_or_else(|| format!("ID '{}' not found in any registry.", model_id))?;

        // 2. Hardware Audit
        let status = self.audit_model_health(manifest.ram_required_gb as f32, manifest.requires_gpu);
        if status == HealthStatus::Disabled {
            return Err("Cluaiz Audit Failed: Insufficient hardware resources for this model.".to_string());
        }

        // 3. Construct Path: [base_dir]/[category]/[id]
        let mut model_path = self.base_models_dir.clone();
        model_path.push(&manifest.category);
        model_path.push(&manifest.id);

        tokio::fs::create_dir_all(&model_path).await
            .map_err(|e| format!("Failed to create model directory: {}", e))?;

        // 4. Initialize Installer
        let installer = ModelInstaller::new(model_path.clone());

        // 5. Pull Weights (The standard GGUF)
        println!("🚀 Cluaiz Pull: Starting download of {}...", manifest.name);
        installer.download_weights(&manifest.download_url, &manifest.huggingface_filename).await?;

        // 6. Pull Assets
        let asset_pairs: Vec<(String, String)> = manifest.assets.clone().into_iter()
            .map(|a| (a.name, a.url))
            .collect();
        installer.pull_assets(asset_pairs).await?;

        // 7. Save local manifest (Persistence)
        let local_manifest_path = model_path.join("model_manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("JSON Serialize Error: {}", e))?;
        tokio::fs::write(local_manifest_path, manifest_json).await
            .map_err(|e| format!("Failed to save local manifest: {}", e))?;

        println!("✅ Model installed successfully at {:?}", model_path);
        Ok(())
    }

    /// Audit: Check hardware health for a specific model
    pub fn audit_model_health(&self, ram_required: f32, requires_gpu: bool) -> HealthStatus {
        self.auditor.audit_performance(ram_required, requires_gpu)
    }
}
