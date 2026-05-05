use color_eyre::Result;
use tokio::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Instant;
use engines::{DownloadEvent, InferenceEvent};

use crate::core::state::{AppState, OsState, UserProfile, ActivityBlock};
use crate::theme::Theme;
use crate::app_enums::{Mode};
use crate::core::dashboard::DashboardEngine;
use crate::core::flow::FlowEngine;
use colored::Colorize;

pub struct App {
    pub state: AppState,
    pub tab: crate::app_enums::Tab,
    pub mode: Mode,
    pub theme: Theme,
    pub tx: mpsc::Sender<DownloadEvent>,
    pub rx: mpsc::Receiver<DownloadEvent>,
    pub _inf_tx: mpsc::Sender<InferenceEvent>,
    pub _abort_handle: Option<Arc<AtomicBool>>,
    pub _last_frame_time: Instant,
    pub flow: FlowEngine,
}

impl App {
    pub fn new(profile_override: Option<UserProfile>) -> color_eyre::Result<Self> {
        let (tx, rx) = mpsc::channel(32);
        let (inf_tx, inf_rx) = mpsc::channel(32);
        let flow = FlowEngine::new()?;
        let app = Self {
            state: AppState::new(profile_override),
            tab: crate::app_enums::Tab::All,
            mode: Mode::Running,
            theme: Theme::default(),
            tx,
            rx,
            _inf_tx: inf_tx,
            _abort_handle: None,
            _last_frame_time: Instant::now(),
            flow,
        };
        let _ = inf_rx;
        Ok(app)
    }

    pub async fn run(mut self) -> Result<()> {
        while self.mode != Mode::Quit {
            match self.state.os_state {
                OsState::Onboarding(_) => {
                    crate::ui::apps::onboarding::native::run_native_flow()?;
                    self.state.os_state = OsState::Dashboard;
                }
                OsState::Dashboard => {
                    // ── 1. Unified Interface Initialization ──
                    if !self.state.printed_logo {
                        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
                        let _ = crossterm::terminal::disable_raw_mode();
                        print!("\x1B[2J\x1B[1;1H"); // Clear and home
                        crate::assets::logos::logo::print_native_logo(self.state.logo_index);
                        println!();
                        println!("  {} {}", "CLUAIZ".cyan().bold(), "v0.1.0".bright_black());
                        println!("  {} {}", "API Gateway: ".dimmed(), "http://0.0.0.0:8000 ↗".cyan().bold());
                        println!("  {} {}", "Dashboard:   ".dimmed(), "http://0.0.0.0:3030 ↗".yellow().bold());
                        self.state.printed_logo = true;
                    }
 
                    // ── 2. Background Event Processing ──
                    while let Ok(event) = self.rx.try_recv() {
                        self.handle_kernel_event(event).await;
                    }
                    crate::ui::apps::stream::commit_to_stdout(&mut self.state);

                    // ── 3. Native Dashboard Interaction ──
                    DashboardEngine::run_native(&mut self.state, &self.tx, &mut self.mode)?;
                }
            }
        }
        Ok(())
    }

    async fn handle_kernel_event(&mut self, event: DownloadEvent) {
        match event {
            DownloadEvent::Progress(prog, _current, _total, _speed, _eta) => {
                self.state.download_progress = prog as f64;
            }
            DownloadEvent::Complete(id) => {
                if id == "INITIAL_LOAD" {
                    self.state.sorted_models = engines::CoreRoster::get_recommendations(
                        &self.state.hardware.to_Hardware_truth(), self.state.ram_gb
                    );
                } else if self.state.downloading_id.as_ref() == Some(&id) {
                    let name = self.state.sorted_models.iter()
                        .find(|m| m.manifest.id == id)
                        .map(|m| m.manifest.name.clone())
                        .unwrap_or_else(|| id.clone());

                    self.state.downloading_id = None;
                    self.state.activity_stream.push(ActivityBlock::DownloadComplete(name));
                    self.state.sorted_models = engines::CoreRoster::get_recommendations(
                        &self.state.hardware.to_Hardware_truth(), self.state.ram_gb
                    );
                }
            }
            _ => {}
        }
    }
}
