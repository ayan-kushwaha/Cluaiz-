use cluaiz_shared::hardware::governor::HardwareGovernor;
use cluaiz_shared::hardware::system_control::HardwareOrchestrator;
use color_eyre::Result;
use colored::Colorize;
use reqwest::Client;
use std::time::Duration;

pub async fn execute(command: crate::BrainCommand) -> Result<()> {
    match command {
        crate::BrainCommand::On { address } => {
            let target = address.unwrap_or_else(|| "local".to_string());
            println!("\n  {} Enabling Cluaizd FFI Brain (Connection: {})...", "🧠".cyan(), target.bold());
            
            // Load, modify, and persist
            if let Ok(mut control) = HardwareGovernor::load_system_control() {
                control.brain.cluaizd_connect_ffi = target.clone();
                if let Err(e) = HardwareOrchestrator::persist_sovereign_state(&control) {
                    eprintln!("  {} Failed to save system control: {}", "❌".red(), e);
                } else {
                    println!("  {} Brain FFI configuration updated successfully.\n", "✅".green());
                }
            } else {
                eprintln!("  {} Failed to load system control config.", "❌".red());
            }
        }
        crate::BrainCommand::Off => {
            println!("\n  {} Disabling Cluaizd FFI Brain...", "🧠".cyan());
            if let Ok(mut control) = HardwareGovernor::load_system_control() {
                control.brain.cluaizd_connect_ffi = "off".to_string();
                if let Err(e) = HardwareOrchestrator::persist_sovereign_state(&control) {
                    eprintln!("  {} Failed to save system control: {}", "❌".red(), e);
                } else {
                    println!("  {} Brain FFI connection disabled successfully.\n", "✅".green());
                }
            } else {
                eprintln!("  {} Failed to load system control config.", "❌".red());
            }
        }
        crate::BrainCommand::Only => {
            println!("\n  {} Enabling Pure Brain Mode (Engine Suspended)...", "🧠".cyan());
            if let Ok(mut control) = HardwareGovernor::load_system_control() {
                control.brain.cluaizd_connect_ffi = "only_brain".to_string();
                if let Err(e) = HardwareOrchestrator::persist_sovereign_state(&control) {
                    eprintln!("  {} Failed to save system control: {}", "❌".red(), e);
                } else {
                    println!("  {} Pure Brain Mode activated. VRAM will not be reserved.\n", "✅".green());
                }
            } else {
                eprintln!("  {} Failed to load system control config.", "❌".red());
            }
        }
        crate::BrainCommand::Status => {
            println!("\n  {} Checking Cluaizd Brain Status...", "🧠".cyan());
            if let Ok(control) = HardwareGovernor::load_system_control() {
                let ffi_status = &control.brain.cluaizd_connect_ffi;
                let enabled = control.brain.is_enabled();
                let is_local = control.brain.is_local();
                
                println!("  * Configuration Flag: {}", ffi_status.bold().magenta());
                println!("  * Enabled: {}", if enabled { "YES".green() } else { "NO".red() });
                println!("  * Mode: {}", if is_local { "Local FFI (LMDB)" } else { "Remote Network (gRPC/HTTP)" });
                
                if enabled {
                    let check_addr = if is_local {
                        "http://localhost:7331".to_string()
                    } else {
                        let mut addr = ffi_status.clone();
                        if !addr.starts_with("http://") && !addr.starts_with("https://") {
                            addr = format!("http://{}", addr);
                        }
                        addr
                    };
                    
                    println!("  * Probing Daemon at: {}...", check_addr.cyan());
                    
                    let client = Client::builder()
                        .timeout(Duration::from_millis(1000))
                        .build()
                        .unwrap_or_default();
                        
                    match client.get(&format!("{}/health", check_addr)).send().await {
                        Ok(res) if res.status().is_success() => {
                            println!("  * Daemon Connection: {} (Healthy)\n", "ONLINE".green());
                        }
                        Ok(res) => {
                            println!("  * Daemon Connection: {} (HTTP Status: {})\n", "WARNING".yellow(), res.status());
                        }
                        Err(e) => {
                            println!("  * Daemon Connection: {} (Error: {})\n", "OFFLINE/UNREACHABLE".red(), e);
                        }
                    }
                } else {
                    println!();
                }
            } else {
                eprintln!("  {} Failed to load system control config.\n", "❌".red());
            }
        }
    }
    Ok(())
}
