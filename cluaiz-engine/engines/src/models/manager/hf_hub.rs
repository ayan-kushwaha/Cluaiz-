use reqwest::Client;
use serde::Deserialize;
use tracing::info;
use crate::models::registry::ModelManifest;

#[derive(Debug, Deserialize)]
struct HfTreeItem {
    path: String,
    size: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct HfVariant {
    pub filename: String,
    pub size_gb: f64,
}

pub struct HuggingFaceHub;

impl HuggingFaceHub {
    /// List all GGUF files in a repository
    pub async fn list_gguf_variants(repo_id: &str) -> Result<Vec<HfVariant>, String> {
        let client = Client::new();
        let url = format!("https://huggingface.co/api/models/{}/tree/main", repo_id);
        
        let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            return Err(format!("Failed to fetch repository '{}'. Does it exist?", repo_id));
        }

        let items: Vec<HfTreeItem> = response.json().await.map_err(|e| e.to_string())?;
        
        let mut variants = Vec::new();
        for item in items {
            if item.path.ends_with(".gguf") {
                let size_gb = item.size.unwrap_or(0) as f64 / (1024.0 * 1024.0 * 1024.0);
                variants.push(HfVariant {
                    filename: item.path,
                    size_gb,
                });
            }
        }

        if variants.is_empty() {
            return Err(format!("No .gguf files found in repository '{}'. Only GGUF format is currently supported.", repo_id));
        }

        Ok(variants)
    }

    pub async fn build_manifest(repo_id: &str, filename: &str, download_size_gb: f64) -> Result<ModelManifest, String> {
        let url = format!("https://huggingface.co/{}/resolve/main/{}", repo_id, filename);
        
        // Base Engine + Weights overhead (~0.5 GB). KV Cache will dynamically add more.
        let ram_required_gb = download_size_gb + 0.5;

        // Auto-format ID to Sovereign Library standard: family:size:gguf:quantization
        let name = filename.to_lowercase().replace(".gguf", "");
        let fallback = name.clone();
        
        let mut family_parts = Vec::new();
        let mut size = "unknown".to_string();
        let mut quant = "unknown".to_string();
        
        let parts: Vec<&str> = name.split('-').collect();
        let mut found_size = false;

        for part in parts {
            if (part.starts_with('q') && part.chars().nth(1).map_or(false, |c| c.is_digit(10))) 
               || part == "f16" || part == "f32" || part == "int4" || part == "int8" {
                quant = part.to_string();
                continue;
            }

            if part.ends_with('b') && (part.chars().next().unwrap_or('a').is_digit(10) || part.contains('x')) {
                size = part.to_string();
                found_size = true;
                continue;
            }

            if part == "instruct" || part == "chat" {
                family_parts.push(part);
                continue;
            }

            if !found_size && part != "gguf" && part != "unsloth" {
                family_parts.push(part);
            }
        }

        let family = if family_parts.is_empty() { fallback } else { family_parts.join("_") };
        let sovereign_id = format!("{}:{}:gguf:{}", family, size, quant);

        Ok(ModelManifest {
            id: sovereign_id.clone(),
            name: sovereign_id.clone(),
            architecture: "Unknown (GGUF)".to_string(),
            parameters: "Unknown".to_string(),
            training_tokens: "Unknown".to_string(),
            bit_depth: 4.0, // Default assumption
            ram_required_gb,
            download_size_gb,
            huggingface_repo: repo_id.to_string(),
            huggingface_filename: filename.to_string(),
            download_url: url,
            description: "Custom HuggingFace Model".to_string(),
            is_cloud_api: false,
            requires_gpu: false,
            is_free_tier: true,
            input_modality: "Text".to_string(),
            context_window: "8k".to_string(), // Default assumption, will be updated by prober
            family: "HuggingFace".to_string(),
            category: "chat".to_string(),
            assets: vec![],
            local_path: None,
            dna_path: None,
            has_vision: false,
            has_audio: false,
            expert_count: None,
            experts_per_token: None,
        })
    }

    /// Fetches the first 8MB of a remote GGUF file to probe its metadata
    pub async fn fetch_partial_gguf_metadata(url: &str) -> Result<(std::collections::HashMap<String, String>, std::collections::HashMap<String, Vec<usize>>, usize), String> {
        use reqwest::header::RANGE;
        use std::io::Write;

        // Custom client that DOES NOT follow redirects automatically
        let custom_policy = reqwest::redirect::Policy::none();
        let client = Client::builder().redirect(custom_policy).build().map_err(|e| e.to_string())?;
        
        let mut target_url = url.to_string();
        
        // Manual redirect following (up to 5 times) to preserve headers
        for _ in 0..5 {
            let res = client.get(&target_url).header(RANGE, "bytes=0-8388607").send().await.map_err(|e| e.to_string())?;
            if res.status().is_redirection() {
                if let Some(loc) = res.headers().get(reqwest::header::LOCATION) {
                    target_url = loc.to_str().unwrap_or(&target_url).to_string();
                    continue;
                }
            }
            
            if !res.status().is_success() && res.status() != reqwest::StatusCode::PARTIAL_CONTENT {
                return Err(format!("Failed to fetch GGUF header chunks. HTTP {}", res.status()));
            }

            let bytes = res.bytes().await.map_err(|e| e.to_string())?;
            
            let temp_dir = std::env::temp_dir();
            let temp_file_path = temp_dir.join(format!("cluaiz_probe_{}.gguf", std::process::id()));
            
            let mut file = std::fs::File::create(&temp_file_path).map_err(|e| e.to_string())?;
            file.write_all(&bytes).map_err(|e| e.to_string())?;
            
            let result = cluaiz_shared::utils::gguf_prober::GGUFProber::probe(&temp_file_path);
            let _ = std::fs::remove_file(&temp_file_path);
            
            return result.map_err(|e| e.to_string());
        }
        
        Err("Too many redirects while trying to fetch metadata".to_string())
    }
}
