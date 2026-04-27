// extern crate shared;
extern crate archer_shared;

use color_eyre::Result;
use std::fs::File;
use tokio::spawn;
use engines::DownloadEvent;

mod core;
mod ui;
mod assets;
mod theme;
mod app_enums;

use crate::core::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // 🚀 SILENCE THE VOID: Redirect all logs to file to prevent TUI corruption
    if let Ok(log_file) = File::create("cluaiz_neural.log") {
        let _ = tracing_subscriber::fmt()
            .with_writer(log_file)
            .with_ansi(false)
            .try_init();
    }

    color_eyre::install()?;

    // 📡 SILICON IGNITION: Fire up the Watchtower Telemetry Server
    engines::telemetry::ignite_watchtower();

    // ── CLI ARGUMENTS PROCESSING ──────────────────────────────────────────
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--benchmark") {
        engines::telemetry::health_check::SovereignHealthChecker::run_full_benchmark();
        return Ok(());
    }

    // ── NATIVE ONBOARDING FLOW ────────────────────────────────────────────
    // Runs in standard terminal mode (no Ratatui yet)
    let profile_over = if !::archer_shared::onboarding::should_skip_onboarding() {
        Some(crate::ui::apps::onboarding::native::run_native_flow()?)
    } else {
        None
    };

    // 🛡️ Panic Guard: Ensure terminal recovery on crash
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Attempt graceful restore — FlowEngine may not be alive yet
        let _ = crate::core::flow::FlowEngine::restore();
        let _ = ratatui::restore();
        hook(info);
    }));

    // ── SOVEREIGN PRIMARY FLOW ──────────────────────────────────────────
    // FlowEngine::new() enters Alternate Screen + Raw Mode automatically.
    let app = App::new(profile_over)?;

    let tx = app.tx.clone();
    let hardware = app.state.hardware.clone();
    let ram = app.state.ram_gb;

    // 🧠 Background Initialization: Load recommendations asynchronously
    spawn(async move {
        let _models = tokio::task::spawn_blocking(move || {
            engines::NeuralRoster::get_recommendations(&hardware.to_silicon_truth(), ram)
        }).await.unwrap_or_default();
        
        let _ = tx.send(DownloadEvent::Complete("INITIAL_LOAD".to_string())).await;
    });

    let app_result = app.run().await;

    // ── SOVEREIGN TEARDOWN ────────────────────────────────────────────
    let _ = crate::core::flow::FlowEngine::restore();

    app_result
}
