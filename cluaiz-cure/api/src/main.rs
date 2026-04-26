//! ═══════════════════════════════════════════════════════════════════════
//!  CURE API Gateway — The Single Entry Point (Port 8000)
//! ═══════════════════════════════════════════════════════════════════════
//!  Every client (Desktop, Mobile, Web, Robot, Developer) talks to CURE
//!  through this gateway. Nothing else is exposed.
//! ═══════════════════════════════════════════════════════════════════════

mod state;
mod handlers;
mod routes;

use colored::*;
use kernel::CureKernel;
use std::env;
use std::sync::Arc;
use storage::EmbeddedManager;

use state::AppState;

#[tokio::main]
async fn main() {
    // ── Initialize logging ──
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .compact()
        .init();

    // ── Determine CURE root directory ──
    let cure_root = env::current_dir().expect("Failed to determine current directory");

    // ── Initialize the CURE pillars ──
    tracing::info!("🔧 Initializing CURE Engine...");

    let embedded = EmbeddedManager::new(cure_root.clone());
    let kernel = CureKernel::new();

    // ── Boot Embedded databases ──
    embedded.boot_all().await;

    // ── Create shared state ──
    let state = Arc::new(AppState { kernel, embedded });

    // ── Build the Router ──
    let app = routes::build(state);

    // ── Bind to Port 8000 ──
    let addr = "0.0.0.0:8000";

    println!("\n{}", "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓".bright_blue());
    println!("{} {}", "┃".bright_blue(), "🧬 CURE Engine".bright_cyan().bold());
    println!("{} {}", "┃".bright_blue(), format!("v{} — Cluaiz Universal Runtime Engine", env!("CARGO_PKG_VERSION")).bright_black());
    println!("{}", "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫".bright_blue());
    println!("{} {}", "┃".bright_blue(), format!("{}{}", "🌐 Gateway:   ".bright_black(), "http://localhost:8000".bright_green().bold()));
    println!("{} {}", "┃".bright_blue(), format!("{}{}", "💚 Status:    ".bright_black(), "ALL SYSTEMS ONLINE".bright_green().bold()));
    println!("{} {}", "┃".bright_blue(), format!("{}{}", "🧠 Kernel:    ".bright_black(), "ACTIVE".magenta().bold()));
    println!("{} {}", "┃".bright_blue(), format!("{}{}", "💾 Storage:   ".bright_black(), "EMBEDDED NATIVE (100MB RAM)".yellow().bold()));
    println!("{}", "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫".bright_blue());
    println!("{} {}", "┃".bright_blue(), "📡 Endpoints:".bright_magenta().bold());
    println!("{} {}", "┃".bright_blue(), format!("{}{}{}", "    POST ".bright_cyan().bold(), "/chat             ".white(), "→ Chat with CURE".bright_black()));
    println!("{} {}", "┃".bright_blue(), format!("{}{}{}", "    GET  ".bright_cyan().bold(), "/sessions         ".white(), "→ List chat sessions".bright_black()));
    println!("{} {}", "┃".bright_blue(), format!("{}{}{}", "    GET  ".bright_cyan().bold(), "/status/embedded  ".white(), "→ DB memory status".bright_black()));
    println!("{} {}", "┃".bright_blue(), format!("{}{}{}", "    GET  ".bright_cyan().bold(), "/hardware         ".white(), "→ Check system RAM/CPU".bright_black()));
    println!("{} {}", "┃".bright_blue(), format!("{}{}{}", "    POST ".bright_cyan().bold(), "/models/download  ".white(), "→ Fetch from HF".bright_black()));
    println!("{} {}", "┃".bright_blue(), format!("{}{}{}", "    POST ".bright_cyan().bold(), "/models/load      ".white(), "→ Activate Model".bright_black()));
    println!("{}", "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫".bright_blue());
    println!("{} {}", "┃".bright_blue(), format!("{}{}{}{}", "🚫 ".white(), "Python BANNED".red(), "   |   🚫 ".white(), "Docker BANNED".red()));
    println!("{} {}", "┃".bright_blue(), format!("{}{}{}", "✨ ".yellow(), "Nothing Need.".bright_white().italic(), " Just CURE.".bright_green().bold()));
    println!("{}", "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛\n".bright_blue());

    tracing::info!("🌐 CURE Gateway listening on {}", addr);

    // ── Start the server ──
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("❌ Failed to bind to port 8000. Is another process using it?");

    axum::serve(listener, app)
        .await
        .expect("❌ CURE API server crashed unexpectedly");
}
