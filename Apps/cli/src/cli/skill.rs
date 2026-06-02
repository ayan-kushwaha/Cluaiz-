use color_eyre::Result;
use colored::Colorize;
use std::path::PathBuf;
use crate::SkillCommand;
use reqwest;
use std::fs;
use std::io::Write;

pub async fn execute(command: SkillCommand) -> Result<()> {
    match command {
        SkillCommand::Install { skill_name } => {
            install_skill(&skill_name).await?;
        }
        SkillCommand::List => {
            list_skills().await?;
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
    
    let mut skill_path = skill_name.to_string();
    
    if let Ok(resp) = registry_resp {
        if resp.status().is_success() {
            if let Ok(registry_json) = resp.json::<serde_json::Value>().await {
                if let Some(skills_obj) = registry_json.get("skills").and_then(|s| s.as_object()) {
                    let mut found = false;
                    
                    // First try direct ID match
                    if let Some(skill_data) = skills_obj.get(skill_name) {
                        if let Some(path) = skill_data.get("path").and_then(|p| p.as_str()) {
                            skill_path = path.to_string();
                            found = true;
                        }
                    } 
                    
                    // If not found by ID, search by name or folder
                    if !found {
                        for (id, skill_data) in skills_obj {
                            let name = skill_data.get("name").and_then(|n| n.as_str()).unwrap_or("");
                            let path = skill_data.get("path").and_then(|p| p.as_str()).unwrap_or("");
                            let folder_name = path.split('/').last().unwrap_or("");
                            
                            if name == skill_name || folder_name == skill_name || id == skill_name {
                                skill_path = path.to_string();
                                found = true;
                                break;
                            }
                        }
                    }
                    
                    if found {
                        println!("  {} [Registry] Found skill at path: {}", "✅".green(), skill_path.bold());
                    } else {
                        println!("  {} [Registry] Skill '{}' not found in official registry. Attempting direct fallback...", "⚠️".yellow(), skill_name.bold());
                    }
                }
            }
        }
    }

    // 2. Fetch the files from the resolved path
    let base_url = format!("https://raw.githubusercontent.com/cluaiz/skills/main/{}", skill_path);
    let files_to_try = vec![
        "manifest.json",
        "SKILL.md",
        "README.md",
        "state.prompt-cache",
        "logic.wasm",
        "connector.mcp",
    ];

    let mut downloaded = 0;
    let client = reqwest::Client::new();

    for file in files_to_try {
        let url = format!("{}/{}", base_url, file);
        let resp = client.get(&url).send().await;
        
        if let Ok(response) = resp {
            if response.status().is_success() {
                if let Ok(bytes) = response.bytes().await {
                    let file_path = skills_dir.join(file);
                    let mut out = fs::File::create(&file_path)?;
                    out.write_all(&bytes)?;
                    println!("    {} Downloaded: {}", "⬇️".blue(), file);
                    downloaded += 1;
                }
            }
        }
    }

    if downloaded > 0 {
        println!("\n  {} [Cluaiz] Skill '{}' successfully installed and registered!\n", "✅".green(), skill_name.bold());
    } else {
        println!("\n  {} [Cluaiz] Failed to find skill '{}' in the official registry.\n", "❌".red(), skill_name.bold());
        // Clean up empty dir
        let _ = fs::remove_dir(&skills_dir);
    }

    Ok(())
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
