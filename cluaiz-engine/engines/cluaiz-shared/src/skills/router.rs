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
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub triggers: SkillTriggers,
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
}

impl SkillRouter {
    pub fn new() -> Self {
        Self {
            keyword_index: HashMap::new(),
            loaded_manifests: HashMap::new(),
        }
    }

    /// Scans the ~/.cluaiz/skills/ directory and builds the FST/Trie index
    pub fn boot_index(&mut self) -> Result<()> {
        let home_dir = dirs::home_dir().ok_or_else(|| color_eyre::eyre::eyre!("No home dir"))?;
        let skills_dir = home_dir.join(".cluaiz").join("skills");
        
        if !skills_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(skills_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let manifest_path = path.join("manifest.json");
                if manifest_path.exists() {
                    if let Ok(content) = fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = serde_json::from_str::<SkillManifest>(&content) {
                            // Register into memory index
                            for keyword in &manifest.triggers.semantic {
                                self.keyword_index.insert(keyword.to_lowercase(), path.clone());
                            }
                            self.loaded_manifests.insert(manifest.id.clone(), manifest);
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
}
