use color_eyre::Result;
use colored::Colorize;
use clap::{Parser, Subcommand};

mod core;
mod ui;
mod assets;
mod theme;
mod app_enums;
mod cli;

use crate::core::bootstrapper::Bootstrapper;

// ── Cluaiz CLI Definition ──────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "cluaiz", about = "Cluaiz-OS: Sovereign Neural Kernel", version = env!("CARGO_PKG_VERSION"), disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Subcommand)]
enum CliCommand {
    /// Pull & run a model. Downloads if not cached.
    Run {
        /// Model ID (e.g. gemma2:2b, bonsai:8b)
        model_id: String,
        
        /// Enter interactive chat mode (Default: true)
        #[arg(short, long, default_value_t = true)]
        interactive: bool,
    },

    /// List all downloaded models in the vault.
    List,

    /// Remove a model from the local vault.
    Rm {
        /// Model ID to remove
        model_id: String,
    },

    /// Show hardware status and silicon health.
    Status,

    /// Re-scan hardware and synchronize SiliconTruth profile.
    Calibrate,

    /// Run a full hardware performance benchmark.
    Benchmark,

    /// Show the dynamic help screen.
    Help,
}

// ──────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    // ══ SOVEREIGN KERNEL SILENCE ══
    // These env vars suppress CUDA Graph + ggml verbose logs at the source.
    // NOTE: We do NOT redirect stderr here because inquire (input box) uses stderr to render.
    // Stderr is selectively redirected only during inference (in dashboard.rs generate_stream).
    // 🚀 GGML_CUDA_USE_GRAPHS=1: Enables 40% speed boost.
    std::env::set_var("GGML_CUDA_USE_GRAPHS", "1");
    std::env::set_var("GGML_LOG_LEVEL", "ERROR");

    color_eyre::install()?;

    let cli = Cli::parse();

    // 🚀 SILENCE THE VOID: Redirect all logs to file before anything else
    if let Ok(log_file) = std::fs::File::create("cluaiz_Core.log") {
        let _ = tracing_subscriber::fmt()
            .with_writer(log_file)
            .with_ansi(false)
            .try_init();
    }

    // 🚀 Cluaiz BOOTSTRAP (Local Dev-Sync & Registry Verification)
    if let Err(e) = Bootstrapper::ignite().await {
        eprintln!("\n  {} [Cluaiz] Bootstrap Failed: {}\n", "❌".red(), e);
        std::process::exit(1);
    }

    // 🚀 SILICON IGNITION: Optimize hardware before execution
    let _ = engines::system_booster::SystemBooster::ignite();

    match cli.command {
        Some(CliCommand::Run { model_id, interactive }) => {
            if let Err(e) = crate::cli::run::execute(&model_id, interactive).await {
                eprintln!("\n  {} [Cluaiz] Run Error: {}\n", "❌".red(), e);
                std::process::exit(1);
            }
        }
        Some(CliCommand::List) => {
            if let Err(e) = crate::cli::list::execute().await {
                eprintln!("\n  {} [Cluaiz] List Error: {}\n", "❌".red(), e);
                std::process::exit(1);
            }
        }
        Some(CliCommand::Rm { model_id }) => {
            if let Err(e) = crate::cli::rm::execute(&model_id).await {
                eprintln!("\n  {} [Cluaiz] Removal Error: {}\n", "❌".red(), e);
                std::process::exit(1);
            }
        }
        Some(CliCommand::Status) => {
            engines::telemetry::health_check::CluaizHealthChecker::run_full_benchmark();
        }
        Some(CliCommand::Calibrate) => {
             println!("\n  {} [Silicon] Initiating Hardware Re-Scan...", "🛰️".cyan());
             engines::hardware::system_control_manager::detect_hardware();
             println!("  {} [Silicon] SiliconTruth profile synchronized.\n", "✅".green());
        }
        Some(CliCommand::Benchmark) => {
            println!("\n  {} [Performance] Starting Full System Benchmark...", "🚀".magenta());
            engines::telemetry::health_check::CluaizHealthChecker::run_full_benchmark();
        }
        Some(CliCommand::Help) => {
            if let Ok(reg) = crate::core::commands::CommandRegistry::load() {
                reg.generate_help();
            } else {
                println!("  {} Error loading commands.json", "❌".red());
            }
        }
        None => {
            // Default to Dashboard if no command provided
            start_dashboard().await?;
        }
    }

    Ok(())
}

async fn start_dashboard() -> Result<()> {
    // 📡 Hardware IGNITION
    engines::telemetry::ignite_watchtower();

    // 🛡️ Panic Guard: Ensure terminal recovery on crash
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crate::core::flow::FlowEngine::restore();
        let _ = ratatui::restore();
        hook(info);
    }));

    // ── Cluaiz PRIMARY FLOW ──
    let app = crate::core::app::App::new(None, None)?;
    app.run().await?;

    let _ = crate::core::flow::FlowEngine::restore();
    Ok(())
}

