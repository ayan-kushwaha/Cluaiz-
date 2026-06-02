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
