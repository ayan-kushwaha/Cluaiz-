use std::path::PathBuf;
use tracing::{info, warn};
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::models::registry::ModelManifest;

#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Progress(f32, u64, u64, f64, u64),
    Complete(String),
    Error(String, String),
    PurgeComplete(String),
    PurgeError(String, String),
}

pub struct ModelDownloader;

impl ModelDownloader {
    fn get_models_dir() -> PathBuf {
        let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        // Step up to find workspace root if running from a sub-crate
        for _ in 0..3 {
            if path.join("models").is_dir() {
                return path.join("models");
            }
            if let Some(parent) = path.parent() {
                path = parent.to_path_buf();
            } else { break; }
        }
        PathBuf::from("models") // Default fallback
    }

    pub fn is_model_cached(category: &str, repo_id: &str, filename: &str) -> bool {
        Self::get_cached_path(category, repo_id, filename).is_some()
    }

    pub fn get_cached_path(category: &str, repo_id: &str, filename: &str) -> Option<PathBuf> {
        let model_name = repo_id.split('/').last().unwrap_or(repo_id);
        let repo_path = Self::get_models_dir().join(category).join(model_name);
        
        // 1. Check for main weight file
        let weight_path = repo_path.join(filename);
        if weight_path.exists() { return Some(weight_path); }
        
        // 2. Fallback: Search for any GGUF in the directory
        if let Ok(entries) = std::fs::read_dir(&repo_path) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("gguf") {
                    return Some(entry.path());
                }
            }
        }
        None
    }

    /// 🌐 NATIVE DOWNLOAD: Includes 'abort' signal, multi-asset support, and manifest generation.
    pub async fn download_gguf_async(
        category: &str,
        repo_id: &str,
        download_url: &str,
        filename: &str,
        assets: Vec<crate::models::registry::ModelAsset>,
        manifest: Option<ModelManifest>,
        tx: mpsc::Sender<DownloadEvent>,
        abort: Arc<AtomicBool>
    ) -> Result<PathBuf, String> {
        let model_name = repo_id.split('/').last().unwrap_or(repo_id);
        let dest_dir = Self::get_models_dir().join(category).join(model_name);
        std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3600))
            .user_agent("Cluaiz-Neural-OS/1.0 (Production Registry)")
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(reqwest::header::REFERER, "https://huggingface.co/".parse().unwrap_or(reqwest::header::HeaderValue::from_static("https://huggingface.co/")));
                headers.insert(reqwest::header::ACCEPT, "*/*".parse().unwrap_or(reqwest::header::HeaderValue::from_static("*/*")));
                headers
            })
            .build()
            .map_err(|e| e.to_string())?;

        // 1. Download the main weights
        Self::download_single_file(&client, download_url, &dest_dir.join(filename), tx.clone(), abort.clone()).await?;

        // 2. Download all specified assets (tokenizer, config, etc.)
        for asset in assets {
            if abort.load(Ordering::SeqCst) { break; }
            let asset_path = dest_dir.join(&asset.name);
            if !asset_path.exists() {
                // Try mirror first, if it fails, trigger Auto-Heal from alternative sources
                if let Err(_) = Self::download_single_file(&client, &asset.url, &asset_path, tx.clone(), abort.clone()).await {
                    let _ = Self::fetch_asset_auto_heal(repo_id, &dest_dir, &asset.name).await;
                }
            }
        }

        // 3. Final safety check for critical assets (tokenizer.json)
        let _ = Self::fetch_asset_auto_heal(repo_id, &dest_dir, "tokenizer.json").await;

        // 4. ✅ Save model_manifest.json — makes the folder fully self-contained & portable
        let weight_path = dest_dir.join(filename);
        if let Some(m) = manifest {
            let manifest_path = dest_dir.join("model_manifest.json");
            if let Ok(json) = serde_json::to_string_pretty(&m) {
                let _ = std::fs::write(&manifest_path, json);
            }
            
            // 🧬 DNA HANDSHAKE: Generate structural_dna.json with Binary Trace
            let _ = Self::generate_sovereign_dna(&m, &dest_dir, &weight_path);
        }

        Ok(weight_path)
    }

    /// 🧬 DNA GENERATOR: Creates the structural backbone for the engine's loader by probing the binary.
    fn generate_sovereign_dna(manifest: &ModelManifest, dest_dir: &std::path::Path, weight_path: &std::path::Path) -> Result<(), String> {
        info!("🧬 [DNA] Generating sovereign architectural backbone for '{}'", manifest.id);
        
        let mut signature = archer_shared::KernelSignature::default();
        signature.is_multimodal = manifest.has_vision;
        if manifest.expert_count.is_some() {
            signature.has_experts = true;
        }

        let bit_val = manifest.bit_depth;
        let mut preferred_runtime = Some(archer_shared::backend::signature::BackendType::RuntimeA); // Default: Candle

        if bit_val < 2.0 {
            signature.is_bitnet = true;
            // 🧠 ARCHER STEERING: BitNet models perform best on Llama.cpp (RuntimeB)
            preferred_runtime = Some(archer_shared::backend::signature::BackendType::RuntimeB);
        }

        let mut dna = crate::models::registry::StructuralDNA {
            model_identity: manifest.id.clone(),
            layer_count: None,
            attention_head_count: None,
            attention_head_count_kv: None,
            attention_head_dim: None,
            hidden_size: None,
            intermediate_size: None,
            attention_dimensionality_truth: None,
            signature: signature.clone(),
            preferred_runtime,
            heterogeneous_map: None,
            dynamic_attributes: {
                let mut map = std::collections::HashMap::new();
                map.insert("bit_depth".to_string(), bit_val.to_string());
                map.insert("parameters".to_string(), manifest.parameters.clone());
                map.insert("training_tokens".to_string(), manifest.training_tokens.clone());
                map.insert("context_window".to_string(), manifest.context_window.clone());
                map.insert("category".to_string(), manifest.category.clone());
                map
            },

        };

        // 🔍 BINARY PROBE: Extracting truth directly from GGUF Silicon
        if weight_path.exists() {
            info!("🧬 [DNA] Probing weight binary: {:?}", weight_path);
            if let Ok(mut file) = std::fs::File::open(weight_path) {
                if let Ok(content) = candle_core::quantized::gguf_file::Content::read(&mut file) {
                    dna.sync_with_gguf_metadata(&content.metadata, &content.tensor_infos);
                    info!("🧬 [DNA] Truth-Grounding complete. Signal verified.");
                } else {
                    warn!("🧬 [DNA] Warning: GGUF content probe failed. DNA remains in skeleton state.");
                }
            }
        }

        let dna_path = dest_dir.join("structural_dna.json");
        if let Ok(json) = serde_json::to_string_pretty(&dna) {
            std::fs::write(&dna_path, json).map_err(|e| e.to_string())?;
        }
        
        Ok(())
    }

    async fn download_single_file(
        client: &reqwest::Client,
        url: &str,
        dest_path: &std::path::Path,
        tx: mpsc::Sender<DownloadEvent>,
        abort: Arc<AtomicBool>
    ) -> Result<(), String> {
        let response = client.get(url).send().await.map_err(|e| {
            if e.is_status() && e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                format!("ERROR: 404 Not Found at {}", url)
            } else if e.is_status() && e.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                "ERROR: 401 Unauthorized (Access Token Required for Gated Repo)".to_string()
            } else { format!("Connection Error: {}", e) }
        })?;

        if !response.status().is_success() {
            return Err(format!("Download failed for {}: HTTP {}", url, response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;
        let mut file = tokio::fs::File::create(dest_path).await.map_err(|e| e.to_string())?;

        let start_time = Instant::now();
        let mut last_update = Instant::now();
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            if abort.load(Ordering::SeqCst) {
                drop(file);
                let _ = std::fs::remove_file(dest_path);
                return Err("ABORTED".to_string());
            }

            let chunk = item.map_err(|e| e.to_string())?;
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
            downloaded += chunk.len() as u64;

            if last_update.elapsed().as_millis() > 100 {
                let progress = if total_size > 0 { downloaded as f32 / total_size as f32 } else { 0.0 };
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 { downloaded as f64 / elapsed } else { 0.0 };
                let eta = if speed > 0.0 { (total_size.saturating_sub(downloaded) as f64 / speed) as u64 } else { 0 };
                let _ = tx.send(DownloadEvent::Progress(progress, downloaded, total_size, speed, eta)).await;
                last_update = Instant::now();
            }
        }
        Ok(())
    }

    /// 🪄 AUTO-HEAL: Recursively hunts for any missing asset (config, tokenizer, etc.) on Hugging Face.
    pub async fn fetch_asset_auto_heal(repo_id: &str, dest_dir: &std::path::Path, asset_name: &str) -> Result<(), String> {
        let asset_path = dest_dir.join(asset_name);
        if asset_path.exists() { return Ok(()); }

        let client = reqwest::Client::new();
        let model_name = repo_id.split('/').last().unwrap_or(repo_id);
        
        // 🚀 SMART FALLBACK LIST: Try various repository formats to bypass gating/missing files
        let repo_ids_to_try = vec![
            repo_id.to_string(),                                      // 1. Original (e.g., google/gemma-3-12b-it)
            format!("unsloth/{}", model_name),                        // 2. Unsloth Mirror (Highest probability for assets)
            format!("bartowski/{}", model_name),                       // 3. Bartowski GGUF (Community standard)
            format!("lmstudio-community/{}", model_name),              // 4. LM Studio Mirror
            repo_id.replace("-GGUF", ""),                             // 5. Stripped GGUF
            repo_id.replace("-GGUF", "-it"),                          // 6. IT variant
        ];

        for id in repo_ids_to_try {
            let url = format!("https://huggingface.co/{}/resolve/main/{}", id, asset_name);
            let response = client.get(&url).send().await.map_err(|e: reqwest::Error| e.to_string())?;

            // ✅ If we get success, we recover. 
            // ❌ If we get 401/403 (Gated) or 404, we continue to the next mirror.
            if response.status().is_success() {
                let mut file = tokio::fs::File::create(&asset_path).await.map_err(|e: std::io::Error| e.to_string())?;
                let mut stream = response.bytes_stream();
                while let Some(item) = stream.next().await {
                    let chunk = item.map_err(|e: reqwest::Error| e.to_string())?;
                    file.write_all(&chunk).await.map_err(|e: std::io::Error| e.to_string())?;
                }
                println!("🪄 [AUTO-HEAL] Recovered '{}' from public mirror: {}", asset_name, id);
                return Ok(());
            } else if response.status() == reqwest::StatusCode::UNAUTHORIZED || response.status() == reqwest::StatusCode::FORBIDDEN {
                println!("🛡️ [AUTO-HEAL] Gated repo detected ({}). Switching to next public mirror...", id);
                continue;
            }
        }
        
        Err(format!("Asset '{}' not found in any repository.", asset_name))
    }

    pub fn download_gguf(
        category: &str,
        repo_id: &str,
        download_url: &str,
        filename: &str,
        assets: Vec<crate::models::registry::ModelAsset>,
        manifest: Option<ModelManifest>,
        tx: mpsc::Sender<DownloadEvent>,
        abort: Arc<AtomicBool>
    ) -> Result<PathBuf, String> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { Self::download_gguf_async(category, repo_id, download_url, filename, assets, manifest, tx, abort).await })
    }

    pub fn purge_model(category: &str, repo_id: &str) -> Result<(), String> {
        let model_name = repo_id.split('/').last().unwrap_or(repo_id);
        let path = Self::get_models_dir().join(category).join(model_name);
        if !path.exists() { return Err("Model directory not found".to_string()); }
        for attempt in 1..=3 {
            match std::fs::remove_dir_all(&path) {
                Ok(_) => return Ok(()),
                Err(_e) => { std::thread::sleep(std::time::Duration::from_millis(500 * attempt as u64)); }
            }
        }
        Err("Purge failed after 3 attempts.".to_string())
    }

    pub fn cleanup_partial_download(category: &str, repo_id: &str) -> Result<(), String> {
        let model_name = repo_id.split('/').last().unwrap_or(repo_id);
        let blobs_path = Self::get_models_dir().join(category).join(model_name).join("blobs");
        if let Ok(entries) = std::fs::read_dir(&blobs_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let ext = path.extension().and_then(|s| s.to_str());
                if ext == Some("part") || ext == Some("lock") { let _ = std::fs::remove_file(path); }
            }
        }
        Ok(())
    }
}
