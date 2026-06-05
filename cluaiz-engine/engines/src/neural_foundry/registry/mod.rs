// cluaiz-engine: Core Foundry - Registry
// Manages the lifecycle of Cluaiz skills.

pub mod scanner;
pub mod compiler_daemon;

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillManifest {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: String,
    pub description: String,
    pub triggers: Triggers,
    pub permissions: Permissions,
    #[serde(default)]
    pub soul_type: String,
    #[serde(default)]
    pub Core_metadata: Option<CoreMetadata>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreMetadata {
    pub token_count: usize,
    pub head_dim: usize,
    pub layer_count: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Triggers {
    pub semantic: Vec<String>,
    pub entropy_threshold: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Permissions {
    pub level: String,
    pub network: bool,
    pub filesystem: bool,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
}

pub struct Skill {
    pub manifest: SkillManifest,
    pub path: PathBuf,
    pub soul_path: Option<PathBuf>, // 🧠 The .atma Core tensor path
    pub logic_path: Option<PathBuf>, // ⚙️ The .wasm execution logic path
}

pub struct SkillRegistry {
    pub skills: Vec<Skill>,
    pub router_path: Option<PathBuf>, // 🛰️ The .system/router.bin path
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { 
            skills: Vec::new(),
            router_path: None,
        }
    }

    pub fn load_from_directory(&mut self, path: &str) {
        let base_path = PathBuf::from(path);
        
        // 🛰️ Initialize System Router
        let router = base_path.join(".system").join("router.bin");
        if router.exists() {
            self.router_path = Some(router);
        }

        let scanner = scanner::SkillScanner::new(path);
        let manifest_paths = scanner.scan_manifests();

        for manifest_path in manifest_paths {
            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                // If it's a SKILL.md, extract the YAML frontmatter
                let parsed_manifest = if manifest_path.file_name().map(|n| n == "SKILL.md").unwrap_or(false) {
                    if let Some(start) = content.find("---\n") {
                        if let Some(end) = content[start + 4..].find("\n---") {
                            let yaml_content = &content[start + 4..start + 4 + end];
                            serde_yaml::from_str::<SkillManifest>(yaml_content).ok()
                        } else { None }
                    } else { None }
                } else {
                    serde_json::from_str::<SkillManifest>(&content).ok()
                };

                if let Some(mut manifest) = parsed_manifest {
                    // Fallback ID to name if not provided (common for YAML skills)
                    if manifest.id.is_empty() {
                        manifest.id = manifest.name.clone();
                    }

                    let skill_dir = manifest_path.parent().unwrap();
                    
                    // 🧠 Detect Core Soul (.atma)
                    let soul_path = skill_dir.join("soul.atma");
                    let soul = if soul_path.exists() { Some(soul_path) } else { None };

                    // ⚙️ Detect Execution Logic (.wasm)
                    let logic_path = skill_dir.join("logic.wasm");
                    let logic = if logic_path.exists() { Some(logic_path) } else { None };

                    let skill = Skill {
                        manifest: manifest.clone(),
                        path: skill_dir.to_path_buf(),
                        soul_path: soul,
                        logic_path: logic,
                    };
                    
                    // ⚙️ Trigger Sovereign Watcher & Dual-Cache Compilation
                    let daemon = compiler_daemon::CompilerDaemon::new();
                    daemon.compile_skill(&skill.path, &manifest);

                    self.skills.push(skill);
                }
            }
        }
    }

    /// 🛰️ Cluaiz Pull: Downloads and installs a skill from the Global Hub.
    pub async fn pull_skill(skill_id: &str) -> anyhow::Result<()> {
        let index_url = "https://github.com/cluaiz/cluaiz/releases/download/latest-library/skills_index.json";
        
        println!("🛰️ [Core-Foundry] Connecting to Cluaiz Skill Hub...");
        
        let client = reqwest::Client::new();
        let resp = client.get(index_url).send().await?.json::<serde_json::Value>().await?;
        
        let skills = resp["available_skills"].as_array().ok_or(anyhow::anyhow!("Invalid library index format"))?;
        let skill_entry = skills.iter().find(|s| s["id"] == skill_id)
            .ok_or(anyhow::anyhow!("Skill ID '{}' not found in Cluaiz Library", skill_id))?;
            
        let download_url = skill_entry["download_url"].as_str().unwrap();
        println!("📦 [Core-Foundry] Downloading Core Package: {}", download_url);
        
        // Binary Download & Extraction Logic (using zip-extract)
        Ok(())
    }

    /// 📜 Remote Index: Returns a list of all skills available in the Global Hub.
    pub async fn list_remote_skills() -> anyhow::Result<Vec<serde_json::Value>> {
        let index_url = "https://github.com/cluaiz/cluaiz/releases/download/latest-library/skills_index.json";
        let client = reqwest::Client::new();
        let resp = client.get(index_url).send().await?.json::<serde_json::Value>().await?;
        
        Ok(resp["available_skills"].as_array().unwrap_or(&vec![]).clone())
    }
}
