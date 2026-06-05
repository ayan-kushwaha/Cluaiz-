use color_eyre::Result;
use colored::Colorize;
use std::path::PathBuf;
use crate::SkillCommand;
use reqwest;
use std::fs;
use std::io::Write;
use neural_core::interfaces::router_contract::EmbeddingDriver;

pub async fn execute(command: SkillCommand) -> Result<()> {
    match command {
        SkillCommand::Install { skill_name } => {
            install_skill(&skill_name).await?;
        }
        SkillCommand::List => {
            list_skills().await?;
        }
        SkillCommand::Cache { command } => {
            handle_cache_command(command).await?;
        }
    }
    Ok(())
}

async fn handle_cache_command(command: crate::SkillCacheCommand) -> Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| color_eyre::eyre::eyre!("Could not find home directory"))?;
    let skills_dir = home_dir.join(".cluaiz").join("skills");
    
    match command {
        crate::SkillCacheCommand::Ls => {
            println!("\n  {} [Cluaiz Dual-Cache] Scanning Global Skill Memory...", "🧠".cyan());
            if !skills_dir.exists() {
                println!("    No skills installed.");
                return Ok(());
            }
            
            let mut total_size = 0;
            let mut cache_count = 0;
            
            for entry in fs::read_dir(&skills_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let cache_dir = path.join(".cache");
                    if cache_dir.exists() {
                        for cache_entry in fs::read_dir(&cache_dir)? {
                            let cache_entry = cache_entry?;
                            let cache_path = cache_entry.path();
                            if cache_path.is_file() {
                                if let Ok(meta) = cache_entry.metadata() {
                                    total_size += meta.len();
                                    cache_count += 1;
                                    let name = cache_path.file_name().unwrap_or_default().to_string_lossy();
                                    let size_mb = meta.len() as f64 / 1_048_576.0;
                                    println!("    {} {} | Size: {:.2} MB", "🔹".blue(), name.bold(), size_mb);
                                }
                            }
                        }
                    }
                }
            }
            println!("\n    Total Caches: {} | Total Size: {:.2} MB", cache_count, total_size as f64 / 1_048_576.0);
        }
        crate::SkillCacheCommand::Clear { model_id, all, force } => {
            println!("\n  {} [Cluaiz Dual-Cache] Initiating Global Wipe...", "🧹".yellow());
            if !skills_dir.exists() { return Ok(()); }
            
            let mut wiped = 0;
            for entry in fs::read_dir(&skills_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let cache_dir = path.join(".cache");
                    if cache_dir.exists() {
                        for cache_entry in fs::read_dir(&cache_dir)? {
                            let cache_entry = cache_entry?;
                            let cache_path = cache_entry.path();
                            if cache_path.is_file() {
                                let name = cache_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                                let mut should_delete = false;
                                
                                if all {
                                    // TODO: Actually check if orphaned via Permission.json
                                    // For now, if all is passed, delete everything if force is true, else mock orphaned.
                                    should_delete = force; // Using force to override for all
                                } else if let Some(target_id) = &model_id {
                                    if name == *target_id {
                                        should_delete = true;
                                    }
                                }
                                
                                if should_delete {
                                    if fs::remove_file(&cache_path).is_ok() {
                                        println!("    {} Wiped: {}", "❌".red(), name);
                                        wiped += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            println!("\n    Successfully wiped {} caches.\n", wiped);
        }
    }
    Ok(())
}

async fn install_skill(skill_name: &str) -> Result<()> {
    println!("\n  {} [Cluaiz] Contacting Universal Skill Registry...", "📡".cyan());
    
    // Setup ~/.cluaiz/skills/ directory
    let home_dir = dirs::home_dir().ok_or_else(|| color_eyre::eyre::eyre!("Could not find home directory"))?;
    let skills_dir = home_dir.join(".cluaiz").join("skills").join(skill_name);
    
    if !skills_dir.exists() {
        fs::create_dir_all(&skills_dir)?;
    }
    
    println!("  {} [Cluaiz] Installing skill '{}' to {}...", "🚀".green(), skill_name.bold(), skills_dir.display());

    // 1. Fetch the master registry
    let registry_url = "https://raw.githubusercontent.com/cluaiz/skills/main/registry.json";
    let client = reqwest::Client::new();
    let registry_resp = client.get(registry_url).send().await;
    
    let mut download_url = String::new();
    
    if let Ok(resp) = registry_resp {
        if resp.status().is_success() {
            if let Ok(registry_json) = resp.json::<serde_json::Value>().await {
                if let Some(skills_obj) = registry_json.get("skills").and_then(|s| s.as_object()) {
                    if let Some(skill_data) = skills_obj.get(skill_name) {
                        if let Some(latest) = skill_data.get("latest").and_then(|v| v.as_str()) {
                            if let Some(versions) = skill_data.get("versions").and_then(|v| v.as_object()) {
                                if let Some(url) = versions.get(latest).and_then(|u| u.as_str()) {
                                    download_url = url.to_string();
                                    println!("  {} [Registry] Found skill release: v{}", "✅".green(), latest.bold());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if download_url.is_empty() {
        println!("  {} [Registry] Skill '{}' not found or has no valid release in the registry.", "❌".red(), skill_name.bold());
        let _ = fs::remove_dir(&skills_dir);
        return Err(color_eyre::eyre::eyre!("Skill not found in registry"));
    }

    // 2. Download the ZIP release
    println!("  {} [Cluaiz] Downloading release package...", "⬇️".cyan());
    let zip_resp = client.get(&download_url).send().await?;
    
    if !zip_resp.status().is_success() {
        let _ = fs::remove_dir(&skills_dir);
        return Err(color_eyre::eyre::eyre!("Failed to download skill package"));
    }
    
    let zip_bytes = zip_resp.bytes().await?;
    let temp_zip_path = skills_dir.join(format!("{}.zip", skill_name));
    
    let mut file = fs::File::create(&temp_zip_path)?;
    file.write_all(&zip_bytes)?;
    
    // 3. Extract the ZIP using native OS tar (Windows 10+ / Linux / macOS)
    println!("  {} [Cluaiz] Extracting package...", "📦".cyan());
    let status = std::process::Command::new("tar")
        .arg("-xf")
        .arg(&temp_zip_path)
        .arg("-C")
        .arg(&skills_dir)
        .status()?;
        
    if !status.success() {
        let _ = fs::remove_file(&temp_zip_path);
        let _ = fs::remove_dir(&skills_dir);
        return Err(color_eyre::eyre::eyre!("Extraction failed"));
    }
    
    // Cleanup the ZIP file
    let _ = fs::remove_file(&temp_zip_path);

    println!("\n  {} [Cluaiz] Skill '{}' successfully installed and registered!\n", "✅".green(), skill_name.bold());

    // Immediately compile vector
    let manifest_path = skills_dir.join("manifest.json");
    let parsed_manifest = if manifest_path.exists() {
        if let Ok(content) = fs::read_to_string(&manifest_path) {
            serde_json::from_str::<engines::neural_foundry::registry::SkillManifest>(&content).ok()
        } else {
            None
        }
    } else {
        let skill_md_path = skills_dir.join("SKILL.md");
        if skill_md_path.exists() {
            if let Ok(content) = fs::read_to_string(&skill_md_path) {
                if let Some(start) = content.find("---\n") {
                    if let Some(end) = content[start + 4..].find("\n---") {
                        let yaml_content = &content[start + 4..start + 4 + end];
                        serde_yaml::from_str::<engines::neural_foundry::registry::SkillManifest>(yaml_content).ok()
                    } else { None }
                } else { None }
            } else { None }
        } else {
            None
        }
    };

    if let Some(mut manifest) = parsed_manifest {
        if manifest.id.is_empty() {
            manifest.id = manifest.name.clone();
        }
        
        let permissions = engines::neural_foundry::security::permission_schema::PermissionSchema::load();
        if let Some(embedding_model_id) = permissions.get_active_embedding_model() {
            let roster = engines::models::registry::CoreRoster::load_roster();
            if let Some(model_manifest) = roster.iter().find(|m| m.id == embedding_model_id) {
                if let Some(local_path) = &model_manifest.local_path {
                    let model_dir = std::path::Path::new(local_path);
                    let model_file = model_dir.join("model.onnx");
                    let tokenizer_file = model_dir.join("tokenizer.json");
                    if model_file.exists() && tokenizer_file.exists() {
                        println!("  {} [Cluaiz] Compiling skill vector immediately...", "⚙️".cyan());
                        let cache_dir = skills_dir.join(".cache");
                        let _ = fs::create_dir_all(&cache_dir);
                        let safe_filename = embedding_model_id.replace(":", "-");
                        let embedding_cache_path = cache_dir.join(format!("{}.emb.bin", safe_filename));
                        
                        let skill_content = if let Some(fm) = extract_frontmatter(&skills_dir) {
                            fm
                        } else {
                            let semantic_triggers = manifest.triggers.semantic.join(", ");
                            format!(
                                "Skill Name: {}\nDescription: {}\nTriggers: {}",
                                manifest.name, manifest.description, semantic_triggers
                            )
                        };

                        if let Ok(mut engine) = cluaiz_onnx::engine::OnnxEngine::new() {
                            if engine.load_text_model(&model_file.to_string_lossy(), &tokenizer_file.to_string_lossy()).is_ok() {
                                if let Ok(vec) = engine.gen_embedding(&skill_content) {
                                    let data_bytes = unsafe { std::slice::from_raw_parts(vec.as_ptr() as *const f32 as *const u8, vec.len() * 4) };
                                    if let Err(e) = std::fs::write(&embedding_cache_path, data_bytes) {
                                        println!("❌ Failed to write binary embedding: {}", e);
                                    } else {
                                        println!("✅ Real Router Embedding generated: {:?}", embedding_cache_path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn extract_frontmatter(skill_dir: &std::path::Path) -> Option<String> {
    let skill_md_path = skill_dir.join("SKILL.md");
    if skill_md_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&skill_md_path) {
            let lines: Vec<&str> = content.lines().collect();
            let mut start_idx = None;
            let mut end_idx = None;
            for (i, line) in lines.iter().enumerate() {
                if line.trim() == "---" {
                    if start_idx.is_none() {
                        start_idx = Some(i);
                    } else {
                        end_idx = Some(i);
                        break;
                    }
                }
            }
            if let (Some(start), Some(end)) = (start_idx, end_idx) {
                if end > start + 1 {
                    let frontmatter_lines = &lines[start + 1..end];
                    return Some(frontmatter_lines.join("\n"));
                }
            }
        }
    }
    None
}



async fn list_skills() -> Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| color_eyre::eyre::eyre!("Could not find home directory"))?;
    let skills_dir = home_dir.join(".cluaiz").join("skills");

    println!("\n  {} [Cluaiz] Installed Sovereign Skills:", "📦".cyan());
    
    if !skills_dir.exists() {
        println!("    No skills installed yet. Use `cluaiz skill install <name>`.");
        return Ok(());
    }

    let mut found = false;
    for entry in fs::read_dir(skills_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                println!("    {} {}", "🔹".blue(), name.bold());
                found = true;
            }
        }
    }

    if !found {
        println!("    No skills installed yet.");
    }
    println!();

    Ok(())
}
