use serde::Deserialize;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct CommandMetadata {
    pub name: String,
    pub usage: String,
    pub description: String,
    pub category: String,
    pub example: String,
}

#[derive(Debug, Deserialize)]
pub struct CommandRegistry {
    pub version: String,
    pub commands: Vec<CommandMetadata>,
}

impl CommandRegistry {
    /// 📂 Industrial Load: Pulls command truth from the local assets.
    pub fn load() -> Result<Self> {
        let mut path = std::env::current_dir()?;
        path.push("assets");
        path.push("commands.json");
        
        if !path.exists() {
            // Fallback for dev runs where cwd might be different
            path = PathBuf::from("Apps/cli/assets/commands.json");
        }

        let content = std::fs::read_to_string(&path)?;
        let registry: CommandRegistry = serde_json::from_str(&content)?;
        Ok(registry)
    }

    /// 🏛️ Help Generator: Dynamically builds the help screen from the JSON registry.
    pub fn generate_help(&self) {
        use colored::Colorize;

        println!("\n  {} Cluaiz-OS Sovereign CLI v{}", "🚀".magenta(), self.version.bold());
        println!("  Source: {}\n", "commands.json".cyan());

        let categories = ["core", "models", "system"];
        
        for cat in categories {
            println!("  {}", cat.to_uppercase().bold().yellow());
            for cmd in self.commands.iter().filter(|c| c.category == cat) {
                println!("    {:<12} {}", cmd.name.green().bold(), cmd.description.dimmed());
                println!("    {} {}\n", "Usage:".dimmed(), cmd.usage.italic());
            }
        }
        
        println!("  Use {} to launch the neural cockpit.\n", "cluaiz".bold().magenta());
    }
}
