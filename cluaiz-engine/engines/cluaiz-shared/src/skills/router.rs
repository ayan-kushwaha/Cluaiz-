use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::sync::{LazyLock, RwLock};

pub static GLOBAL_SKILL_ROUTER: LazyLock<RwLock<SkillRouter>> = LazyLock::new(|| {
    let mut router = SkillRouter::new();
    let _ = router.boot_index(); // Ignore failure on boot
    RwLock::new(router)
});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillManifest {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub triggers: SkillTriggers,
    #[serde(default)]
    pub soul_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillTriggers {
    pub semantic: Vec<String>,
    pub entropy_threshold: Option<f32>,
}

/// The Tier 1: Skill Router
/// Lazy loads manifest metadata in O(1) without touching WASM or KV-cache.
pub struct SkillRouter {
    /// Maps a keyword trigger directly to the skill's absolute path
    pub keyword_index: HashMap<String, PathBuf>,
    pub loaded_manifests: HashMap<String, SkillManifest>,
    /// Maps a skill's absolute path to its semantic vector
    pub skill_vectors: HashMap<PathBuf, Vec<f32>>,
}

#[derive(serde::Deserialize)]
struct MinimalModelSelection {
    text: Option<String>,
}

#[derive(serde::Deserialize)]
struct MinimalPermissionSchema {
    vector_models: Option<MinimalModelSelection>,
}

fn get_active_embedding_model() -> Option<String> {
    if let Some(home_dir) = dirs::home_dir() {
        let permission_path = home_dir.join(".cluaiz").join("engine").join("Permission.json");
        if permission_path.exists() {
            if let Ok(content) = fs::read_to_string(permission_path) {
                if let Ok(schema) = serde_json::from_str::<MinimalPermissionSchema>(&content) {
                    if let Some(vector_models) = schema.vector_models {
                        return vector_models.text;
                    }
                }
            }
        }
    }
    None
}

impl SkillRouter {
    pub fn new() -> Self {
        Self {
            keyword_index: HashMap::new(),
            loaded_manifests: HashMap::new(),
            skill_vectors: HashMap::new(),
        }
    }

    /// Scans the ~/.cluaiz/skills/ directory and builds the FST/Trie index
    pub fn boot_index(&mut self) -> Result<()> {
        let home_dir = dirs::home_dir().ok_or_else(|| color_eyre::eyre::eyre!("No home dir"))?;
        let skills_dir = home_dir.join(".cluaiz").join("skills");
        
        if !skills_dir.exists() {
            return Ok(());
        }

        let active_model = get_active_embedding_model();
        let target_filename = active_model.map(|m| format!("{}.emb.bin", m.replace(":", "-")));

        for entry in fs::read_dir(skills_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let manifest_path = path.join("manifest.json");
                let skill_md_path = path.join("SKILL.md");
                
                let parsed_manifest = if manifest_path.exists() {
                    if let Ok(content) = fs::read_to_string(&manifest_path) {
                        serde_json::from_str::<SkillManifest>(&content).ok()
                    } else { None }
                } else if skill_md_path.exists() {
                    if let Ok(content) = fs::read_to_string(&skill_md_path) {
                        if let Some(start) = content.find("---\n") {
                            if let Some(end) = content[start + 4..].find("\n---") {
                                let yaml_content = &content[start + 4..start + 4 + end];
                                serde_yaml::from_str::<SkillManifest>(yaml_content).ok()
                            } else { None }
                        } else { None }
                    } else { None }
                } else { None };

                if let Some(mut manifest) = parsed_manifest {
                    if manifest.id.is_empty() {
                        manifest.id = manifest.name.clone();
                    }
                    // Register into memory index
                    for keyword in &manifest.triggers.semantic {
                        self.keyword_index.insert(keyword.to_lowercase(), path.clone());
                    }
                    self.loaded_manifests.insert(manifest.id.clone(), manifest);
                    
                    // Load Semantic Vector
                    let cache_dir = path.join(".cache");
                    if cache_dir.exists() {
                        if let Some(ref filename) = target_filename {
                            let emb_path = cache_dir.join(filename);
                            if emb_path.exists() {
                                if let Ok(bytes) = fs::read(&emb_path) {
                                    let floats: Vec<f32> = bytes
                                        .chunks_exact(4)
                                        .map(|b| f32::from_ne_bytes(b.try_into().unwrap()))
                                        .collect();
                                    self.skill_vectors.insert(path.clone(), floats);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// O(1) Check if a generated token triggers any skill
    pub fn check_trigger(&self, token_text: &str) -> Option<PathBuf> {
        let normalized = token_text.trim().to_lowercase();
        self.keyword_index.get(&normalized).cloned()
    }

    /// O(N) Check if a prompt vector triggers any skill via Cosine Similarity
    pub fn check_semantic_trigger(&self, prompt_vector: &[f32], default_threshold: f32) -> Option<PathBuf> {
        let mut best_match = None;
        let mut highest_score = default_threshold;

        for (path, skill_vec) in &self.skill_vectors {
            if skill_vec.len() == prompt_vector.len() && !skill_vec.is_empty() {
                let mut dot = 0.0;
                let mut mag_a = 0.0;
                let mut mag_b = 0.0;
                
                for (a, b) in prompt_vector.iter().zip(skill_vec.iter()) {
                    dot += a * b;
                    mag_a += a * a;
                    mag_b += b * b;
                }
                
                if mag_a > 0.0 && mag_b > 0.0 {
                    let score = dot / (mag_a.sqrt() * mag_b.sqrt());
                    if score > highest_score {
                        highest_score = score;
                        best_match = Some(path.clone());
                    }
                }
            }
        }
        
        best_match
    }
}
