#![allow(warnings)]

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
    /// Manage Sovereign AI Skills
    Skill {
        #[command(subcommand)]
        command: SkillCommand,
    },
    /// Pull & run a model. Downloads if not cached.
    Run {
        /// Model ID (e.g. gemma2:2b, bonsai:8b)
        model_id: String,
        
        /// Enter interactive chat mode (Default: true)
        #[arg(short, long, default_value_t = true, action = clap::ArgAction::Set)]
        interactive: bool,
    },

    /// List all downloaded models in the vault.
    List,

    /// Download and register a model into the local vault.
    Pull {
        /// Model ID (e.g. gemma2:2b, unsloth/Qwen3.5-4B-GGUF)
        model_id: String,
    },

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
    Benchmark {
        /// Optional model ID to benchmark (runs all if omitted)
        model_id: Option<String>,

        /// Number of times to run each prompt.
        #[arg(short, long, default_value_t = 1)]
        runs: usize,
    },

    /// Show the dynamic help screen.
    Help,

    /// Show active neural engines in memory.
    Ps,

    /// View or configure the system performance booster settings.
    Booster {
        /// Set KV-Cache Quantization level (auto, kv16, kv8, kv4)
        #[arg(long)]
        kv_quant: Option<String>,

        /// Set Context Shifting / Sliding Window mode (auto, off, minimal, standard, aggressive, extreme)
        #[arg(long)]
        context_shift: Option<String>,

        /// Set execution performance mode (edge, multitasking, balance, max_boost, ultra_max_boost, hyper_cluster)
        #[arg(long)]
        mode: Option<String>,

        /// Enable/Disable Hybrid Speculative Decoding (on, off, auto)
        #[arg(long)]
        spec_decode: Option<String>,
    },

    /// Ingest a document natively for semantic chunking and RAG.
    Ingest {
        /// The file path to ingest
        file_path: String,
    },

    /// Test JIT KV Cache compilation and memory footprint
    TestJit,

    /// Manage the Cluaizd Brain Connection
    Brain {
        #[command(subcommand)]
        command: BrainCommand,
    },

    /// Setup Cluaiz Node Profile and Identity
    Setup {
        #[command(subcommand)]
        command: SetupCommand,
    },
}

#[derive(Subcommand)]
pub enum SetupCommand {
    /// Generate and register Purpose Vectorization for the Node Profile
    Profile,
}

#[derive(Subcommand)]
pub enum BrainCommand {
    /// Enable the FFI Database connection (defaults to local, or specify a remote address)
    On {
        /// Remote database IP:Port (e.g. 10.0.0.5:8080)
        address: Option<String>,
    },
    /// Disable the FFI Database connection
    Off,
    /// Pure Brain Mode: Enable local DB but suspend Engine LLM loading to save VRAM
    Only,
    /// View the connection status and background daemon health
    Status,
}

#[derive(Subcommand)]
enum SkillCommand {
    /// Install a skill from the cluaiz-skills registry
    Install {
        /// Name of the skill to install (e.g., 'web-search-github')
        skill_name: String,
    },
    /// List all locally installed skills
    List,
    /// Manage Global Dual-Cache Artifacts
    Cache {
        #[command(subcommand)]
        command: SkillCacheCommand,
    },
}

#[derive(Subcommand)]
pub enum SkillCacheCommand {
    /// List all active and orphaned dual-caches
    Ls,
    /// Clear orphaned caches (or target a specific model cache)
    Clear {
        /// The model cache ID to target (optional)
        model_id: Option<String>,
        
        /// Clear all orphaned caches globally
        #[arg(long)]
        all: bool,
        
        /// Force deletion even if model is active
        #[arg(short = 'f', long)]
        force: bool,
    },
}

// ──────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    // ══ FORCE UTF-8 FOR WINDOWS CONSOLE ══
    #[cfg(windows)]
    unsafe {
        extern "system" {
            fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
        }
        SetConsoleOutputCP(65001);
    }
    // ══ SOVEREIGN KERNEL SILENCE ══
    // These env vars suppress CUDA Graph + ggml verbose logs at the source.
    // NOTE: We do NOT redirect stderr here because inquire (input box) uses stderr to render.
    // Stderr is selectively redirected only during inference (in dashboard.rs generate_stream).
    // 🚀 GGML_CUDA_USE_GRAPHS=1: Enables 40% speed boost.
    std::env::set_var("GGML_CUDA_USE_GRAPHS", "1");
    std::env::set_var("GGML_LOG_LEVEL", "ERROR");

    color_eyre::install()?;

    let cli = Cli::parse();

    // 🚀 SILENCE THE VOID: Redirect all logs to file at the project root
    let log_path = {
        let mut path = std::env::current_dir().unwrap_or_default();
        let mut root = None;
        for _ in 0..5 {
            if path.join("Apps").exists() && path.join("interface-engines").exists() {
                root = Some(path.clone());
                break;
            }
            if let Some(parent) = path.parent() {
                path = parent.to_path_buf();
            } else {
                break;
            }
        }
        if let Some(r) = root {
            r.join("cluaiz_Core.log")
        } else {
            std::path::PathBuf::from("cluaiz_Core.log")
        }
    };

    if let Ok(log_file) = std::fs::File::create(&log_path) {
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

    // 🚀 Check Pure Brain Mode
    let mut pure_brain = false;
    if let Ok(control) = cluaiz_shared::hardware::governor::HardwareGovernor::load_system_control() {
        if control.brain.is_pure_brain() {
            pure_brain = true;
        }
    }

    // 🚀 SILICON IGNITION: Optimize hardware before execution
    if !pure_brain {
        let _ = engines::system_booster::SystemBooster::ignite();
    }

    match cli.command {
        Some(CliCommand::Run { model_id, interactive }) => {
            if let Err(e) = crate::cli::run::execute(&model_id, interactive).await {
                eprintln!("\n  {} [Cluaiz] Run Error: {}\n", "❌".red(), e);
                std::process::exit(1);
            }
        }
        Some(CliCommand::Pull { model_id }) => {
            if let Err(e) = crate::cli::pull::execute(&model_id).await {
                eprintln!("\n  {} [Cluaiz] Pull Error: {}\n", "❌".red(), e);
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
        Some(CliCommand::Benchmark { model_id, runs }) => {
            if let Err(e) = crate::cli::benchmark::execute(model_id, runs).await {
                eprintln!("\n  {} [Cluaiz] Benchmark Error: {}\n", "❌".red(), e);
                std::process::exit(1);
            }
        }
        Some(CliCommand::Help) => {
            if let Ok(reg) = crate::core::commands::CommandRegistry::load() {
                reg.generate_help();
            } else {
                println!("  {} Error loading commands.json", "❌".red());
            }
        }
        Some(CliCommand::Ps) => {
            if let Err(e) = crate::cli::ps::execute().await {
                eprintln!("\n  {} [Cluaiz] Process Status Error: {}\n", "❌".red(), e);
                std::process::exit(1);
            }
        }
        Some(CliCommand::Booster { kv_quant, context_shift, mode, spec_decode }) => {
            if let Err(e) = crate::cli::booster::execute(kv_quant, context_shift, mode, spec_decode).await {
                eprintln!("\n  {} [Cluaiz] Booster Config Error: {}\n", "❌".red(), e);
                std::process::exit(1);
            }
        }
        Some(CliCommand::Skill { command }) => {
            if let Err(e) = crate::cli::skill::execute(command).await {
                eprintln!("\n  {} [Cluaiz] Skill Manager Error: {}\n", "❌".red(), e);
                std::process::exit(1);
            }
        }
        Some(CliCommand::Ingest { file_path }) => {
            if let Err(e) = crate::cli::ingest::execute(&file_path).await {
                eprintln!("\n  {} [Cluaiz] Ingestion Error: {}\n", "❌".red(), e);
                std::process::exit(1);
            }
        }
        Some(CliCommand::TestJit) => {
            if let Err(e) = crate::cli::test_jit::execute().await {
                eprintln!("\n  {} [Cluaiz] JIT Test Error: {}\n", "❌".red(), e);
                std::process::exit(1);
            }
        }
        Some(CliCommand::Brain { command }) => {
            if let Err(e) = crate::cli::brain::execute(command).await {
                eprintln!("\n  {} [Cluaiz] Brain Manager Error: {}\n", "❌".red(), e);
                std::process::exit(1);
            }
        }
        Some(CliCommand::Setup { command }) => {
            if let Err(e) = crate::cli::setup::execute(command).await {
                eprintln!("\n  {} [Cluaiz] Setup Error: {}\n", "❌".red(), e);
                std::process::exit(1);
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

