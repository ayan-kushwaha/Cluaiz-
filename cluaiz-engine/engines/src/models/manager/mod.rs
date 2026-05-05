use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::models::manager::client::RegistryClient;
use crate::models::manager::installer::ModelInstaller;
use crate::models::manager::auditor::{HardwareAuditor, HealthStatus};

pub mod client;
pub mod installer;
pub mod auditor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAsset {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelManifest {
    pub id: String,
    pub name: String,
    pub architecture: String,
    pub parameters: String,
    pub ram_required_gb: f32,
    pub download_size_gb: f32,
    pub download_url: String,
    pub category: String,
    pub requires_gpu: bool,
    pub context_window: String,
    pub assets: Vec<ModelAsset>,
}

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

    /// Installation: Pull a specific model variant from Hugging Face
    /// Syntax: bonsai:v1.1-bonsai-4b-atma-instruct
    pub async fn pull_model(&self, family: &str, version: &str, id: &str) -> Result<(), String> {
        // 1. Fetch Manifest from Registry (GitHub/Cloudflare)
        let manifest_json = self.client.fetch_manifest(family, version, id).await?;
        let manifest: ModelManifest = serde_json::from_str(&manifest_json)
            .map_err(|e| format!("Manifest Parse Error: {}", e))?;

        // 2. Hardware Audit (Hardware Truth)
        let status = self.audit_model_health(manifest.ram_required_gb, manifest.requires_gpu);
        if status == HealthStatus::Disabled {
            return Err("Cluaiz Audit Failed: Insufficient hardware resources for this model.".to_string());
        }

        // 3. Construct Categorized Path (SSD)
        // Pattern: [base_dir]/[category]/[id]
        let mut model_path = self.base_models_dir.clone();
        model_path.push(&manifest.category);
        model_path.push(&manifest.id);

        // 4. Create Directory surgically
        tokio::fs::create_dir_all(&model_path).await
            .map_err(|e| format!("Failed to create model directory: {}", e))?;

        // 5. Initialize Installer for this specific path
        let installer = ModelInstaller::new(model_path.clone());

        // 6. Pull Weights from Hugging Face
        println!("🚀 Cluaiz Pull: Starting download of {} weights...", manifest.name);
        installer.download_weights(&manifest.download_url, &format!("{}.gguf", manifest.id)).await?;

        // 7. Pull Supplemental Assets (tokenizer, config)
        let asset_pairs: Vec<(String, String)> = manifest.assets.into_iter()
            .map(|a| (a.name, a.url))
            .collect();
        installer.pull_assets(asset_pairs).await?;

        // 8. Save local manifest
        let local_manifest_path = model_path.join("model_manifest.json");
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
