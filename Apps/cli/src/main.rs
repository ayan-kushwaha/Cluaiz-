use color_eyre::Result;
use std::fs::File;
use tokio::spawn;
use engines::DownloadEvent;
use colored::Colorize;
use clap::{Parser, Subcommand};

mod core;
mod ui;
mod assets;
mod theme;
mod app_enums;
mod cli;

use crate::core::app::App;

// ── Cluaiz CLI Definition ──────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "cluaiz", about = "Universal Neural Kernel", version = "0.1.0", disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,

    #[arg(long, hide = true)]
    benchmark: bool,

    #[arg(long, hide = true)]
    calibrate: bool,
}

#[derive(Subcommand)]
enum CliCommand {
    /// Pull & run a model. Downloads if not cached.
    Run {
        /// Model ID  (e.g. bonsai:8b)
        model_id: String,
    },
}

// ──────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    // ══ PHASE 1 — HEADLESS FAST PATH ══════════════════════════════════════
    // These commands exit BEFORE bootstrap so no stray output contaminates
    // the clean terminal UX.

    let raw_args: Vec<String> = std::env::args().collect();

    match raw_args.get(1).map(|s| s.as_str()) {
        Some("help") | Some("-h") | Some("--help") => {
            return crate::cli::help::print_help();
        }
        Some("run") => {
            // Validate arg count before any heavy init
            let model_id = match raw_args.get(2) {
                Some(id) => id.clone(),
                None => {
                    println!(
                        "\n  {} Usage: cluaiz run <model-id>\n  {} Example: cluaiz run bonsai:8b\n",
                        "⚠️ ".yellow(),
                        "💡".cyan()
                    );
                    return Ok(());
                }
            };
            // Minimal init for network commands (log redirect only, no TUI bootstrap)
            if let Ok(log_file) = File::create("cluaiz_Core.log") {
                let _ = tracing_subscriber::fmt()
                    .with_writer(log_file)
                    .with_ansi(false)
                    .try_init();
            }
            color_eyre::install()?;
            return crate::cli::run::execute(&model_id).await;
        }
        _ => {}
    }

    // ══ PHASE 2 — FULL BOOT (TUI Dashboard path) ══════════════════════════

    // 🚀 SILENCE THE VOID: Redirect all logs to file before anything else
    if let Ok(log_file) = File::create("cluaiz_Core.log") {
        let _ = tracing_subscriber::fmt()
            .with_writer(log_file)
            .with_ansi(false)
            .try_init();
    }

    color_eyre::install()?;

    // 🚀 Cluaiz BOOTSTRAP
    crate::core::bootstrapper::Bootstrapper::ignite().await?;

    // 📡 Hardware IGNITION
    engines::telemetry::ignite_watchtower();

    // Parse remaining flags (--benchmark, --calibrate)
    let cli_args = Cli::parse();

    if cli_args.benchmark {
        engines::telemetry::health_check::CluaizHealthChecker::run_full_benchmark();
        return Ok(());
    }

    if cli_args.calibrate {
        println!("  {} [Cluaiz] Commencing Hardware DNA Extraction...", "🧬".cyan());
        archer_shared::hardware::HardwareGovernor::auto_calibrate()
            .map_err(|e| color_eyre::eyre::eyre!("Calibration failed: {}", e))?;
        println!("  {} [Cluaiz] SiliconTruth synchronized.", "✅".green());
        return Ok(());
    }

    // ── NATIVE ONBOARDING FLOW ────────────────────────────────────────────
    let profile_over = if !::archer_shared::onboarding::should_skip_onboarding() {
        Some(crate::ui::apps::onboarding::native::run_native_flow()?)
    } else {
        None
    };

    // 🛡️ Panic Guard: Ensure terminal recovery on crash
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crate::core::flow::FlowEngine::restore();
        let _ = ratatui::restore();
        hook(info);
    }));

    // ── Cluaiz PRIMARY FLOW ───────────────────────────────────────────────
    let app = App::new(profile_over)?;

    let tx = app.tx.clone();
    let hardware = app.state.hardware.clone();
    let ram = app.state.ram_gb;

    // 🧠 Background Initialization: Load recommendations asynchronously
    spawn(async move {
        let _models = tokio::task::spawn_blocking(move || {
            engines::CoreRoster::get_recommendations(&hardware.to_Hardware_truth(), ram)
        }).await.unwrap_or_default();

        let _ = tx.send(DownloadEvent::Complete("INITIAL_LOAD".to_string())).await;
    });

    let app_result = app.run().await;

    // ── Cluaiz TEARDOWN ───────────────────────────────────────────────────
    let _ = crate::core::flow::FlowEngine::restore();

    app_result
}
