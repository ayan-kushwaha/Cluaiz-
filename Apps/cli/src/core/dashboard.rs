use crate::core::state::{ActivityBlock, AppState};
use color_eyre::Result;
use colored::Colorize;
use crossterm::execute;
use crossterm::cursor;
use engines::DownloadEvent;
use inquire::{
    ui::{Attributes, Color, RenderConfig, Styled},
    Select, Text,
};
use std::io::{stdout, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

// ── 📦 MODULAR APPS ──
use crate::ui::apps::registry::RegistryApp;

pub struct DashboardEngine;

impl DashboardEngine {
    pub fn run_native(
        state: &mut AppState,
        tx: &mpsc::UnboundedSender<DownloadEvent>,
        rx: &mut mpsc::UnboundedReceiver<DownloadEvent>,
        mode: &mut crate::app_enums::Mode,
    ) -> Result<()> {
        // ══ 🔒 Cluaiz RENDER CONFIG ══
        let config = RenderConfig::default();

        // ── 🧬 ATOMIC Core DISCOVERY (Cluaiz Startup Scan) ──
        if state.sorted_models.is_empty() {
            state.sorted_models = engines::CoreRoster::get_recommendations(
                &state.hardware.to_hardware_truth(),
                state.ram_gb,
            );
        }

        // ── 📡 Cluaiz TELEMETRY IGNITION (Ghost Observer Singleton) ──
        let state_pulse = cluaiz_shared::hardware::telemetry::get_pulse();
        let _pulse_ref = state_pulse.clone();
        let app_start_time = std::time::Instant::now();
        let last_inference_duration = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let last_ttft = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let peak_power = Arc::new(std::sync::atomic::AtomicU64::new(0));
        
        let duration_ref = last_inference_duration.clone();
        let ttft_ref = last_ttft.clone();
        let pwr_ref = peak_power.clone();

        // 📡 Cluaiz PULSE CONTROL: Visibility gate to prevent chat history bleeding
        let show_dashboard = Arc::new(AtomicBool::new(false));
        let _show_ref = show_dashboard.clone();
 
        let engine_ref = state.Core_engine.clone();
 
        let tokens_generated = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let tps_ref = Arc::new(std::sync::atomic::AtomicU64::new(0)); 
        
        let pulse_worker_ref = state_pulse.clone();
        let show_worker_ref = show_dashboard.clone();
        let tokens_worker_ref = tokens_generated.clone();
        let tps_worker_ref = tps_ref.clone();
        let duration_worker_ref = last_inference_duration.clone();
        let ttft_worker_ref = last_ttft.clone();
        let pwr_worker_ref = peak_power.clone();

        std::thread::spawn(move || {
            let engine = engine_ref; 
            use crossterm::{
                cursor, execute,
                style::{self, Stylize},
                terminal,
            };
            
            let pulse_ref = pulse_worker_ref;
            let show_ref = show_worker_ref;
            let tokens_ref = tokens_worker_ref;
            let tps_ref = tps_worker_ref;
            let mut stdout = std::io::stdout();
            // Guard variables to avoid redundant redraws
            let mut prev_tokens: usize = 0;
            let mut prev_tps_bits: u64 = 0;
            let mut prev_peak_bits: u64 = 0;
            loop {
                // 🛑 ATOMIC GATE: Skip rendering during bot inference phases
                if !show_ref.load(std::sync::atomic::Ordering::SeqCst) {
                    // Reset guard state so next activation forces a render
                    prev_tokens = 0;
                    prev_tps_bits = 0;
                    prev_peak_bits = 0;
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    continue;
                }
                // 🧪 MICRO-LOCKING: Copy data and drop lock immediately to unblock background updates
                let (
                    cpu,
                    cpu_temp,
                    _cpu_ghz,
                    ram_used,
                    ram_pct,
                    _vram_used,
                    _vram_total,
                    vram_pct,
                    gpu_temp,
                    _tps,
                ) = {
                    let pulse_lock = pulse_ref.pulse.read().unwrap();
                    let primary_gpu_temp = pulse_lock.gpus.get(0).map(|g| g.temperature_c).unwrap_or(0.0);
                    let current_pwr = pulse_lock.gpus.get(0).map(|g| g.power_draw_watts).unwrap_or(0.0);
                    
                    // 🔋 Peak Power Tracking
                    let old_peak_bits = pwr_worker_ref.load(Ordering::SeqCst);
                    let old_peak = f64::from_bits(old_peak_bits);
                    if (current_pwr as f64) > old_peak {
                        pwr_worker_ref.store((current_pwr as f64).to_bits(), Ordering::SeqCst);
                    }

                    (
                        pulse_lock.cpu.utilization_pct,
                        pulse_lock.cpu.temperature_c,
                        pulse_lock.cpu.clock_ghz,
                        pulse_lock.ram.used_gb,
                        pulse_lock.ram.utilization_pct,
                        pulse_lock.vram_used_gb,
                        pulse_lock.vram_total_gb,
                        pulse_lock.vram_pressure_pct,
                        primary_gpu_temp,
                        {
                            let current_count = pulse_ref.tps_counter.load(Ordering::SeqCst);
                            let last_count = tokens_ref.swap(current_count, Ordering::SeqCst);
                            let diff = current_count.saturating_sub(last_count);
                            diff as f64 * 6.66 // Adjusted for 150ms interval
                        },
                    )
                };

                let uptime = app_start_time.elapsed().as_millis() / 500;
                let inf_duration_bits = duration_worker_ref.load(Ordering::SeqCst);
                let inf_duration = f64::from_bits(inf_duration_bits);
                
                let ttft_bits = ttft_worker_ref.load(Ordering::SeqCst);
                let ttft = f64::from_bits(ttft_bits);

                let peak_pwr_bits = pwr_worker_ref.load(Ordering::SeqCst);
                let peak_pwr = f64::from_bits(peak_pwr_bits);
                let is_blink_on = uptime % 2 == 0;

                                // If nothing changed since last draw, skip rendering to avoid flicker
                let cur_tokens = pulse_ref.tps_counter.load(Ordering::SeqCst);
                let cur_tps_bits = tps_ref.load(Ordering::SeqCst);
                let cur_peak_bits = pwr_worker_ref.load(Ordering::SeqCst);
                if cur_tokens == prev_tokens && cur_tps_bits == prev_tps_bits && cur_peak_bits == prev_peak_bits {
                    // No metric update – sleep and continue
                    std::thread::sleep(std::time::Duration::from_millis(150));
                    continue;
                }
                // Update guard state
                prev_tokens = cur_tokens;
                prev_tps_bits = cur_tps_bits;
                prev_peak_bits = cur_peak_bits;
                
                // 🧠 Neural State Handshake
                let is_loaded = engine.is_loaded.load(std::sync::atomic::Ordering::SeqCst);
                let loading_err = engine.loading_error.blocking_lock();
                let (neural_label, neural_color): (&str, style::Color) = if is_loaded {
                    ("LIVE", style::Color::Green)
                } else if loading_err.is_some() {
                    ("LINK FAIL", style::Color::Red)
                } else {
                    ("", style::Color::Black) // 🧼 SILENCE: Remove LOADING... text
                };

                let status_dot = if is_blink_on { "●" } else { " " };
                let _cpu_color = if cpu < 50.0 { style::Color::Green } else if cpu < 80.0 { style::Color::Yellow } else { style::Color::Red };
                let _gpu_color = if (vram_pct as f32) < 50.0 { style::Color::Green } else if (vram_pct as f32) < 80.0 { style::Color::Yellow } else { style::Color::Red };
                let _ram_color = if ram_pct < 50.0 { style::Color::Green } else if ram_pct < 80.0 { style::Color::Yellow } else { style::Color::Red };
                let status_color = if cpu < 50.0 && (vram_pct as f32) < 50.0 { style::Color::Green } else if cpu < 80.0 || (vram_pct as f32) < 80.0 { style::Color::Yellow } else { style::Color::Red };

                // Surgical Overwrite: Force-inject metrics into the ABSOLUTE BOTTOM (rows-1)
                if show_ref.load(Ordering::SeqCst) {
                    if let Ok((_cur_x, mut cur_y)) = cursor::position() {
                        let (cols, rows) = terminal::size().unwrap_or((80, 24));
                        
                        // 🚀 PROACTIVE SCROLL: If prompt is at the very bottom, push it up to make room for footer
                        if cur_y >= rows - 1 {
                            let _ = execute!(stdout, terminal::ScrollUp(1), cursor::MoveUp(1));
                            cur_y -= 1; 
                        }

                        // ── 🎨 RESPONSIVE TELEMETRY ENGINE ──
                        let is_compact = cols < 120;
                        let is_minimal = cols < 85;

                        let total_tokens = pulse_ref.tps_counter.load(Ordering::SeqCst);
                        let old_tps_bits = tps_ref.load(Ordering::SeqCst);
                        let last_tps = f64::from_bits(old_tps_bits);

                        let _ = execute!(
                            stdout,
                            cursor::SavePosition,
                            cursor::MoveTo(0, rows - 1),
                            terminal::Clear(terminal::ClearType::CurrentLine),
                            style::Print(Stylize::bold(Stylize::dim("[ "))),
                            style::Print(Stylize::bold(status_dot.with(status_color))),
                        );

                        if !neural_label.is_empty() && !is_minimal {
                            let _ = execute!(stdout, style::Print(format!(" {} │ ", neural_label).with(neural_color).bold()));
                        }

                        if is_compact {
                            // Shortened version but STILL has all data to prevent terminal wrap loop!
                            let _ = execute!(stdout, style::Print(format!(" CPU:{:.0}% │ GPU:{:.0}% │ RAM:{:.0}%", cpu, vram_pct, ram_pct).dim()));
                            if total_tokens > 0 {
                                let _ = execute!(stdout, style::Print(format!(" │ TPS:{:.1} │ TKN:{} │ TIM:{:.0}s │ PWR:{:.0}W", last_tps, total_tokens, inf_duration, peak_pwr).dim().bold()));
                            }
                        } else {
                            // Full expanded version
                            let _ = execute!(stdout, style::Print(format!(" CPU: {:.0}°C ({:.0}%) │ GPU: {:.0}°C ({:.0}%) │ RAM: {:.1}GB ({:.0}%)", cpu_temp, cpu, gpu_temp, vram_pct, ram_used, ram_pct).dim()));
                            if total_tokens > 0 {
                                let _ = execute!(stdout, style::Print(format!(" │ TPS: {:.1} │ TKN: {} │ TTFT: {:.2}s │ TIM: {:.1}s │ PWR: {:.0}W", last_tps, total_tokens, ttft, inf_duration, peak_pwr).dim().bold()));
                            }
                        }

                        let _ = execute!(
                            stdout,
                            style::Print(Stylize::bold(Stylize::dim(" ]"))),
                            cursor::RestorePosition
                        );
                        let _ = stdout.flush();
                    }
                } else {
                    // 🛑 SILENCE: Clear the absolute bottom when deactivated
                    let (_cols, rows) = terminal::size().unwrap_or((80, 24));
                    let _ = execute!(
                        stdout,
                        cursor::SavePosition,
                        cursor::MoveTo(0, rows - 1),
                        terminal::Clear(terminal::ClearType::CurrentLine),
                        cursor::RestorePosition
                    );
                    let _ = stdout.flush();
                }

                // 🏎️ TPS CALCULATION: Moving Average
                let current_count = pulse_ref.tps_counter.load(Ordering::SeqCst);
                let last_count = tokens_ref.load(Ordering::SeqCst);
                let diff = current_count.saturating_sub(last_count);
                tokens_ref.store(current_count, Ordering::SeqCst);

                let current_tps = diff as f64 / 0.150;
                let old_tps_bits = tps_ref.load(Ordering::SeqCst);
                let old_tps = f64::from_bits(old_tps_bits);
                
                // 🛑 PERSISTENCE GUARD: If main loop has frozen the TPS (avg_tps), don't decay it to 0.0
                let smoothed_tps = if current_tps > 0.0 {
                    (old_tps * 0.7) + (current_tps * 0.3)
                } else if current_count > 0 && old_tps > 0.0 {
                    old_tps // Hold the frozen record
                } else {
                    0.0
                };

                tps_ref.store(smoothed_tps.to_bits(), Ordering::SeqCst);

                std::thread::sleep(std::time::Duration::from_millis(150));
            }
        });
        // 🚀 CLUAIZ AUTO-BOOT: Activate the latest engine silently only if no model is loaded
        let is_engine_loaded = state.Core_engine.is_loaded.load(std::sync::atomic::Ordering::SeqCst);
        if state._active_model_id.is_none() && !is_engine_loaded {
            let auto_boot_name = state.sorted_models.iter().filter(|m| m.is_cached).next().map(|m| m.manifest.name.clone());
            if let Some(name) = auto_boot_name {
                println!("\n  {} Auto-Booting Neural Kernel: {}...", "🚀".magenta(), name.bold());
                let _ = Self::handle_model_switch(state, tx, rx, "", Some(&name));
            }
        }
        // 🖊️ INPUT FIX: Ensure cursor is on a fresh line before inquire renders
        println!();

        let mut last_booster_modified = std::fs::metadata(dirs::home_dir().unwrap_or_default().join(".cluaiz").join("engine").join("system_booster.json")).and_then(|m| m.modified()).ok();

        loop {
            show_dashboard.store(false, std::sync::atomic::Ordering::SeqCst);
            
            let input = Text::new(">")
                .with_placeholder("Type your message or @ & / for menu")
                .with_render_config(config.clone())
                .prompt();

            // 🛑 DEACTIVATE: Hide telemetry immediately after input to keep the log clean
            show_dashboard.store(false, std::sync::atomic::Ordering::SeqCst);
            
            let (_cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
            let mut stdout = std::io::stdout();
            let _ = execute!(stdout, cursor::SavePosition, cursor::MoveTo(0, rows - 1), crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine), cursor::RestorePosition);
            let _ = stdout.flush();

            if input.is_ok() {
                print!("\x1B[1A\x1B[2K");
                stdout.flush()?;
            }

            let now = std::time::Instant::now();
            let delta = now.duration_since(state.last_input_time).as_millis();
            state.last_input_time = now;

            match input {
                Ok(val) => {
                    let val = val.trim();
                    if val.is_empty() {
                        continue;
                    }

                    // 📋 PASTE DETECTION & MERGING
                    if delta < 50 {
                        state.chat_paste_buffer.push(val.to_string());
                        continue;
                    }

                    let final_message = if !state.chat_paste_buffer.is_empty() {
                        let mut merged = state.chat_paste_buffer.join("\n");
                        merged.push('\n');
                        merged.push_str(val);
                        state.chat_paste_buffer.clear();
                        merged
                    } else {
                        val.to_string()
                    };

                    print!("\x1B[1A\x1B[2K");
                    stdout.flush()?;

                    if final_message.starts_with('/') {
                        Self::handle_command(state, tx, mode, &final_message[1..])?;
                        if *mode == crate::app_enums::Mode::Quit {
                            break;
                        }
                    } else if final_message.starts_with('@') {
                        Self::handle_model_switch(state, tx, rx, &final_message[1..], None)?;
                    } else {
                        // show_dashboard.store(true, std::sync::atomic::Ordering::SeqCst);
                        
                        // ── 👤 USER MESSAGE ──
                        use crossterm::style::Stylize;
                        let icon = Stylize::bold(Stylize::cyan("👤"));
                        println!("{} {}", icon, final_message.clone().white());
                        state.activity_stream.push(ActivityBlock::Chat(
                            "USER".to_string(),
                            final_message.to_string(),
                        ));
                        state.rendered_actions_count += 1;

                        // ── 🧿 NEURAL DISPATCH ──
                        let _ = std::io::Write::flush(&mut std::io::stdout());

                        // ── 🤖 REAL Core STREAMING ────────────────────────
                        let full_response =
                            std::sync::Arc::new(std::sync::Mutex::new(String::new()));
                        let full_clone = full_response.clone();
                        let _first_token = true;
                        // 🧠 Think-mode state machine
                        let in_think = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                        let _in_think_cb = in_think.clone();
                        // 🛑 EOS detection: stops display when model generates stop token
                        let reached_eos = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                        let eos_cb = reached_eos.clone();

                        let start_time = std::time::Instant::now();
                        let initial_tokens = state_pulse.tps_counter.load(Ordering::SeqCst);
                        
                        // Reset session metrics
                        pwr_ref.store(0.0f64.to_bits(), Ordering::SeqCst);
                        ttft_ref.store(0.0f64.to_bits(), Ordering::SeqCst);

                        let pulse_for_snapshot = state_pulse.clone();
                        let _pwr_cb = pwr_ref.clone();
                        let ttft_cb = ttft_ref.clone();
                        
                        // ── 🔥 HOT RELOAD ENGINE SETTINGS ──
                        let booster_path = dirs::home_dir().unwrap_or_default().join(".cluaiz").join("engine").join("system_booster.json");
                        if let Ok(meta) = std::fs::metadata(&booster_path) {
                            if let Ok(modified) = meta.modified() {
                                let mut needs_reload = false;
                                if let Some(last) = last_booster_modified {
                                    if modified > last {
                                        needs_reload = true;
                                    }
                                }
                                last_booster_modified = Some(modified);

                                if needs_reload {
                                    if let Some(model_id) = state._active_model_id.clone() {
                                        if let Some(model) = state.sorted_models.iter().find(|m| m.manifest.id == model_id) {
                                            if let Some(local_path) = &model.manifest.local_path {
                                                let path = std::path::PathBuf::from(local_path);
                                                println!("\r\x1B[2K\x1B[0m{} Hot-Reloading Neural Engine based on new settings...", crossterm::style::Stylize::magenta("🚀"));
                                                let rt = tokio::runtime::Handle::current();
                                                tokio::task::block_in_place(|| {
                                                    let _ = rt.block_on(state.Core_engine.load_model(path));
                                                });
                                                print!("\x1B[1A\x1B[2K\r"); // clear message
                                                let _ = std::io::stdout().flush();
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let stream_result = tokio::task::block_in_place(|| {
                            let mut lock = state.Core_engine.router.blocking_lock();
                            
                            // 🧬 Dynamic Config Fetch: Zero Hardcoding
                            let stop_seqs = lock.get_active_dna().map(|d| d.stop_sequences.clone()).unwrap_or_default();
                            
                            // 🎭 Orchestration: Format prompt based on model DNA
                            let formatted_prompt = if let Some(ref dna) = lock.active_dna {
                                let tm = cluaiz_shared::TemplateManager::default();
                                tm.format(dna, &final_message)
                            } else {
                                final_message.clone()
                            };

                            // 🧬 DYNAMIC TOKEN ALLOCATION: Calculate space based on DNA Context Window
                            let ctx_window = lock.get_active_dna().and_then(|d| d.max_context_length).unwrap_or(2048);
                            let prompt_tokens = 0; // We no longer rely on external tokenizers for length prediction
                            
                            let max_t = lock.get_active_dna()
                                .and_then(|d| d.inference_params.get("max_tokens"))
                                .and_then(|v| v.parse::<usize>().ok())
                                .unwrap_or(8192); // 🚀 DYNAMIC: Allow large stream, context shifting will handle KV bounds.

                            // 🔇 SURGICAL SILENCE: freopen stderr→NUL only during inference.
                            #[cfg(windows)]
                            unsafe {
                                extern "C" { fn __acrt_iob_func(idx: u32) -> *mut libc::FILE; }
                                libc::freopen(
                                    "NUL\0".as_ptr() as *const libc::c_char,
                                    "w\0".as_ptr() as *const libc::c_char,
                                    __acrt_iob_func(2),
                                );
                            }
                            #[cfg(not(windows))]
                            unsafe {
                                libc::freopen(
                                    "/dev/null\0".as_ptr() as *const libc::c_char,
                                    "w\0".as_ptr() as *const libc::c_char,
                                    libc::fdopen(2, "w\0".as_ptr() as *const libc::c_char),
                                );
                            }
                            
                            let mut first_token = true;
                            let pulse_clone = state_pulse.clone(); // 🧬 Clone for closure move
                            let result = lock.generate_stream(
                                &formatted_prompt, // 🚀 Use formatted_prompt here!
                                max_t,
                                Box::new(move |token: String| {
                                    // 🛑 Stop if already past EOS
                                    if eos_cb.load(Ordering::SeqCst) { return; }

                                    // 🛑 Deep-Suffix Scan
                                    if let Ok(mut res) = full_clone.lock() {
                                        let clean_res = (res.clone() + &token).replace("\n", "").replace("\r", "").replace(" ", "");
                                        if stop_seqs.iter().any(|s| {
                                            let clean_s = s.replace("\n", "").replace("\r", "").replace(" ", "");
                                            !clean_s.is_empty() && clean_res.ends_with(&clean_s)
                                        }) {
                                            eos_cb.store(true, Ordering::SeqCst);
                                            return;
                                        }

                                        // 🤖 First-Token Handshake
                                        if first_token {
                                            let ttft = start_time.elapsed().as_secs_f64();
                                            ttft_cb.store(ttft.to_bits(), Ordering::SeqCst);

                                            let mut out = std::io::stdout();
                                            let _ = out.write_all(format!("\r\x1B[2K\x1B[0m{} ", crossterm::style::Stylize::magenta("🤖")).as_bytes());
                                            let _ = out.flush();
                                            first_token = false;
                                        }

                                        // Normal token → Display
                                        let current_full = res.clone() + &token;
                                        if current_full.contains("<think>") && !current_full.contains("</think>") {
                                             print!("\x1B[90m{}\x1B[0m", token);
                                        } else if token.contains("</think>") {
                                             print!("\x1B[90m{}\x1B[0m", token);
                                        } else {
                                             print!("{}", token);
                                        }
                                        let _ = std::io::stdout().flush();
                                        res.push_str(&token);
                                        pulse_clone.tps_counter.fetch_add(1, Ordering::SeqCst);
                                    }
                                }),
                            );
                            
                            // 🔊 RESTORE stderr → CONOUT$ (always the active Windows console)
                            #[cfg(windows)]
                            unsafe {
                                extern "C" { fn __acrt_iob_func(idx: u32) -> *mut libc::FILE; }
                                libc::freopen(
                                    "CONOUT$\0".as_ptr() as *const libc::c_char,
                                    "w\0".as_ptr() as *const libc::c_char,
                                    __acrt_iob_func(2),
                                );
                            }
                            #[cfg(not(windows))]
                            unsafe {
                                libc::freopen(
                                    "/dev/tty\0".as_ptr() as *const libc::c_char,
                                    "w\0".as_ptr() as *const libc::c_char,
                                    libc::fdopen(2, "w\0".as_ptr() as *const libc::c_char),
                                );
                            }
                            result
                        });

                        let end_time = std::time::Instant::now();
                        let duration = end_time.duration_since(start_time).as_secs_f64();
                        let final_tokens = pulse_for_snapshot.tps_counter.load(Ordering::SeqCst);
                        let tokens_in_this_run = final_tokens.saturating_sub(initial_tokens);
                        let avg_tps = if duration > 0.0 { tokens_in_this_run as f64 / duration } else { 0.0 };

                        // 📈 FREEZE FINAL RECORD IN DASHBOARD BAR
                        tps_ref.store(avg_tps.to_bits(), Ordering::SeqCst);
                        duration_ref.store(duration.to_bits(), Ordering::SeqCst);

                        let response = if let Err(e) = stream_result {
                            use crossterm::style::Stylize;
                            let err_msg = format!("{} ERROR: {}", Stylize::red("🤖"), e);
                            println!("{}", err_msg);
                            err_msg
                        } else {
                            let res = full_response.lock().unwrap().clone();
                            if res.is_empty() {
                                use crossterm::style::Stylize;
                                let empty_msg =
                                    format!("{} ERROR: Generated empty response.", Stylize::red("🤖"));
                                println!("{}", empty_msg);
                                empty_msg
                            } else {
                                res
                            }
                        };

                        state.activity_stream.push(ActivityBlock::Chat(
                            "ARCHER".to_string(),
                            response.to_string(),
                        ));

                        // 🚀 REVEAL DASHBOARD: Manifest the record after completion
                        show_dashboard.store(true, Ordering::SeqCst);
                        
                        let ttft_secs = f64::from_bits(ttft_ref.load(Ordering::SeqCst));
                        let registry = cluaiz_shared::hardware::governor::HardwareGovernor::load_process_registry();
                        let my_pid = std::process::id().to_string();
                        let vram_used_gb = registry.get(&my_pid).map(|i| i.vram_gb).unwrap_or(0.0);

                        println!("\n  {} │ {} tokens │ {:.1} TPS │ {:.2}s │ TTFT: {:.2}s │ VRAM Used: {:.2} GB", 
                            colored::Colorize::magenta("⚡ System Benchmark"), 
                            colored::Colorize::cyan(tokens_in_this_run.to_string().as_str()), 
                            avg_tps, 
                            duration,
                            ttft_secs,
                            vram_used_gb
                        );
                        println!(); // ensure prompt starts on fresh line
                    }
                }
                Err(_) => {
                    *mode = crate::app_enums::Mode::Quit;
                    break;
                }
            }
        }
        Ok(())
    }

    fn handle_command(
        state: &mut AppState,
        tx: &mpsc::UnboundedSender<DownloadEvent>,
        mode: &mut crate::app_enums::Mode,
        cmd: &str,
    ) -> Result<()> {
        match cmd {
            "menu" | "" => {
                let config = RenderConfig::default()
                    .with_prompt_prefix(Styled::new("🏠︎").with_fg(Color::LightCyan))
                    .with_highlighted_option_prefix(
                        Styled::new("⮞")
                            .with_fg(Color::LightCyan)
                            .with_attr(Attributes::BOLD),
                    );
                let options = vec!["Model List", "Settings", "Help", "Quit"];
                let ans = Select::new("Main Menu:", options)
                    .with_render_config(config)
                    .prompt()?;

                print!("\x1B[1A\x1B[2K\r");
                stdout().flush()?;

                match ans {
                    "Model List" => RegistryApp::show(state, tx)?,
                    "Settings" => println!("  {} Settings coming soon...", "⚙️".yellow()),
                    "Help" => println!("  {} Help coming soon...", "ℹ️".blue()),
                    "Quit" => *mode = crate::app_enums::Mode::Quit,
                    _ => {}
                }
            }
            "quit" | "exit" => *mode = crate::app_enums::Mode::Quit,
            "clear" => {
                print!("\x1B[2J\x1B[1;1H");
                state.printed_logo = false;
            }
            cmd if cmd.starts_with("run ") => {
                let model_id = &cmd[4..];
                println!("  {} Silicon Dispatch: '{}'...", "🧬".cyan(), model_id);
                
                let manager = engines::models::manager::ModelManager::new(
                    engines::models::registry::REGISTRY_URL.to_string(),
                    std::path::PathBuf::from("models")
                );

                tokio::task::block_in_place(|| {
                    let rt = tokio::runtime::Handle::current();
                    match rt.block_on(manager.pull_model(model_id)) {
                        Ok(_) => println!("  {} Link Established.", "✅".green()),
                        Err(e) => println!("  {} Dispatch Failed: {}", "❌".red(), e),
                    }
                });
            }
            _ => {
                println!("  {} Unknown command: /{}", "❌".red(), cmd);
            }
        }
        Ok(())
    }

    fn handle_model_switch(
        state: &mut AppState,
        _tx: &mpsc::UnboundedSender<DownloadEvent>,
        rx: &mut mpsc::UnboundedReceiver<DownloadEvent>,
        _filter: &str,
        auto_boot_target: Option<&str>,
    ) -> Result<()> {
        state.handle_events(rx);
        let config = RenderConfig::default()
            .with_prompt_prefix(
                Styled::new("@")
                    .with_fg(Color::LightCyan)
                    .with_attr(Attributes::BOLD),
            )
            .with_highlighted_option_prefix(Styled::new("⮞").with_fg(Color::LightCyan));

        let ans = if let Some(target) = auto_boot_target {
            target.to_string()
        } else {
            loop {
                let master_options = vec![
                    "🧠 Switch Model".to_string(),
                    "⚡ Engine Modes".to_string(),
                    "🚀 System Booster".to_string(),
                ];
                let master_ans = match Select::new("Action:", master_options)
                    .with_render_config(config.clone())
                    .prompt() {
                    Ok(ans) => ans,
                    Err(inquire::InquireError::OperationCanceled) | Err(inquire::InquireError::OperationInterrupted) => {
                        print!("\x1B[1A\x1B[2K\r");
                        stdout().flush()?;
                        return Ok(());
                    }
                    Err(e) => return Err(e.into()),
                };
                    
                print!("\x1B[1A\x1B[2K\r");
                stdout().flush()?;

                if master_ans.contains("Engine Modes") {
                    let modes = vec![
                        "⚡ Flash Mode (High Speed)".to_string(),
                        "🧠 Think Mode (Deep Reasoning)".to_string(),
                        "🚀 Boot Mode (Auto-Start Engine)".to_string(),
                    ];
                    let mode_ans = match Select::new("Select Mode:", modes)
                        .with_render_config(config.clone())
                        .prompt() {
                        Ok(ans) => ans,
                        Err(inquire::InquireError::OperationCanceled) | Err(inquire::InquireError::OperationInterrupted) => {
                            print!("\x1B[1A\x1B[2K\r"); // Erase <canceled>
                            stdout().flush()?;
                            continue; // One step back
                        }
                        Err(e) => return Err(e.into()),
                    };
                        
                    print!("\x1B[1A\x1B[2K\r");
                    stdout().flush()?;
                    
                    let mut booster = cluaiz_shared::hardware::governor::HardwareGovernor::load_booster_settings().unwrap_or_default();
                    if mode_ans.contains("Flash Mode") {
                        booster.mode_run = cluaiz_shared::hardware::schema::booster::BoosterMode::Edge;
                        booster.think_mode = cluaiz_shared::hardware::schema::booster::FeatureState::Off;
                    } else if mode_ans.contains("Think Mode") {
                        booster.mode_run = cluaiz_shared::hardware::schema::booster::BoosterMode::MaxBoost;
                        booster.think_mode = cluaiz_shared::hardware::schema::booster::FeatureState::On;
                    } else if mode_ans.contains("Boot Mode") {
                        booster.mode_run = cluaiz_shared::hardware::schema::booster::BoosterMode::Balance;
                        booster.think_mode = cluaiz_shared::hardware::schema::booster::FeatureState::Auto;
                    }
                    
                    let _ = cluaiz_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster);
                    
                    println!("  {} {} activated and saved to system_booster.json.", "✅".green(), mode_ans.bold());
                    return Ok(());
                } else if master_ans.contains("System Booster") {
                    let mut booster_path = dirs::home_dir().unwrap_or_default();
                    booster_path.push(".cluaiz");
                    booster_path.push("engine");
                    booster_path.push("system_booster.json");

                    loop {
                        let mut booster = cluaiz_shared::hardware::governor::HardwareGovernor::load_booster_settings().unwrap_or_default();
                        
                        let compute_mode_str = match booster.n_gpu_layers {
                            0 => "CPU Only".to_string(),
                            -1 => "GPU (Full Offload)".to_string(),
                            n => format!("Hybrid ({} Layers)", n),
                        };

                        let mut options = vec![
                            format!("Neural Mode (Current: {:?})", booster.mode_run),
                            format!("Compute Device (Current: {})", compute_mode_str),
                            format!("Turbo Quant (Current: {:?})", booster.turbo_quant),
                            format!("Flash Attention (Current: {:?})", booster.flash_attention),
                            format!("Speculative Decoding (Current: {:?})", booster.speculative_decoding),
                            format!("Auto Round (Current: {:?})", booster.auto_round),
                            format!("DFlash (FlashKDA) (Current: {:?})", booster.dflash),
                            format!("Context Shifting (Current: {:?})", booster.context_shifting),
                            format!("Force VRAM Reclaim (Current: {:?})", booster.force_vram_reclaim),
                            format!("KV Cache Quantization (Current: {:?})", booster.kv_cache_quantization),
                        ];
                        options.push("🔙 Back to Menu".to_string());
                        
                        let target_ans = match Select::new("Configure Setting:", options)
                            .with_render_config(config.clone())
                            .prompt() {
                            Ok(ans) => ans,
                            Err(_) => {
                                print!("\x1B[1A\x1B[2K\r"); // Erase <canceled>
                                stdout().flush()?;
                                break; // One step back (returns to master menu)
                            }
                        };
                        print!("\x1B[1A\x1B[2K\r");
                        stdout().flush()?;

                        if target_ans == "🔙 Back to Menu" {
                            break; // One step back
                        }

                        let key_part = target_ans.split(" (").next().unwrap_or("").to_string();

                        // If they select Compute Device, show device sub-menu directly
                        if key_part.as_str() == "Compute Device" {
                            let device_options = vec![
                                "GPU (Full Offload)".to_string(),
                                "CPU Only".to_string(),
                                "Custom Layers".to_string(),
                            ];
                            let selected_device = match Select::new("Select Compute Device:", device_options)
                                .with_render_config(config.clone())
                                .prompt() {
                                Ok(ans) => ans,
                                Err(_) => {
                                    print!("\x1B[1A\x1B[2K\r");
                                    stdout().flush()?;
                                    continue;
                                }
                            };
                            print!("\x1B[1A\x1B[2K\r");
                            stdout().flush()?;

                            match selected_device.as_str() {
                                "GPU (Full Offload)" => {
                                    booster.n_gpu_layers = -1;
                                }
                                "CPU Only" => {
                                    booster.n_gpu_layers = 0;
                                }
                                "Custom Layers" => {
                                    let layers_input = inquire::CustomType::<i32>::new("Enter number of GPU layers (-1 for full offload):")
                                        .with_default(-1)
                                        .with_render_config(config.clone())
                                        .prompt();
                                    match layers_input {
                                        Ok(layers) => {
                                            booster.n_gpu_layers = layers;
                                        }
                                        Err(_) => {
                                            print!("\x1B[1A\x1B[2K\r");
                                            stdout().flush()?;
                                            continue;
                                        }
                                    }
                                    print!("\x1B[1A\x1B[2K\r");
                                    stdout().flush()?;
                                }
                                _ => {}
                            }

                            if let Ok(_) = cluaiz_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster) {
                                println!("  {} System Booster updated: Compute Device = {}", "✅".green(), selected_device.bold());
                            } else {
                                println!("  {} Failed to save system booster settings.", "❌".red());
                            }
                            continue;
                        }

                        // Special sub-menus for context shifting & reclaim
                        if key_part.as_str() == "Context Shifting" {
                            let shift_modes = vec![
                                "Off".to_string(),
                                "Minimal".to_string(),
                                "Standard".to_string(),
                                "Aggressive".to_string(),
                                "Extreme".to_string(),
                                "Auto".to_string(),
                            ];
                            let selected_shift = match Select::new("Select Context Shifting:", shift_modes)
                                .with_render_config(config.clone())
                                .prompt() {
                                Ok(ans) => ans,
                                Err(_) => {
                                    print!("\x1B[1A\x1B[2K\r");
                                    stdout().flush()?;
                                    continue;
                                }
                            };
                            print!("\x1B[1A\x1B[2K\r");
                            stdout().flush()?;

                            booster.context_shifting = match selected_shift.as_str() {
                                "Off" => cluaiz_shared::hardware::schema::booster::ContextShiftingMode::Off,
                                "Minimal" => cluaiz_shared::hardware::schema::booster::ContextShiftingMode::Minimal,
                                "Standard" => cluaiz_shared::hardware::schema::booster::ContextShiftingMode::Standard,
                                "Aggressive" => cluaiz_shared::hardware::schema::booster::ContextShiftingMode::Aggressive,
                                "Extreme" => cluaiz_shared::hardware::schema::booster::ContextShiftingMode::Extreme,
                                _ => cluaiz_shared::hardware::schema::booster::ContextShiftingMode::Auto,
                            };

                            if let Ok(_) = cluaiz_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster) {
                                println!("  {} System Booster updated: Context Shifting = {}", "✅".green(), selected_shift.bold());
                            } else {
                                println!("  {} Failed to save system booster settings.", "❌".red());
                            }
                            continue;
                        }

                        if key_part.as_str() == "KV Cache Quantization" {
                            let kv_options = vec![
                                "16-bit (Lossless / High Precision)".to_string(),
                                "8-bit (50% VRAM Saving / Balanced)".to_string(),
                                "4-bit (75% VRAM Saving / High Compression)".to_string(),
                                "Auto (Dynamic Quantization)".to_string(),
                            ];
                            let selected_kv = match Select::new("Select KV Cache Quantization:", kv_options)
                                .with_render_config(config.clone())
                                .prompt() {
                                Ok(ans) => ans,
                                Err(_) => {
                                    print!("\x1B[1A\x1B[2K\r");
                                    stdout().flush()?;
                                    continue;
                                }
                            };
                            print!("\x1B[1A\x1B[2K\r");
                            stdout().flush()?;

                            booster.kv_cache_quantization = match selected_kv.as_str() {
                                s if s.starts_with("16-bit") => cluaiz_shared::hardware::schema::booster::KvCacheQuantization::Kv16,
                                s if s.starts_with("8-bit") => cluaiz_shared::hardware::schema::booster::KvCacheQuantization::Kv8,
                                s if s.starts_with("4-bit") => cluaiz_shared::hardware::schema::booster::KvCacheQuantization::Kv4,
                                _ => cluaiz_shared::hardware::schema::booster::KvCacheQuantization::Auto,
                            };

                            if let Ok(_) = cluaiz_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster) {
                                println!("  {} System Booster updated: KV Cache Quantization = {}", "✅".green(), selected_kv.bold());
                            } else {
                                println!("  {} Failed to save system booster settings.", "❌".red());
                            }
                            continue;
                        }

                        let values = vec!["On".to_string(), "Off".to_string(), "Auto".to_string()];
                        
                        let val_ans = match Select::new(&format!("Set {}:", key_part), values)
                            .with_render_config(config.clone())
                            .prompt() {
                            Ok(ans) => ans,
                            Err(_) => {
                                print!("\x1B[1A\x1B[2K\r"); // Erase <canceled>
                                stdout().flush()?;
                                continue; // One step back (stays in System Booster list)
                            }
                        };
                        print!("\x1B[1A\x1B[2K\r");
                        stdout().flush()?;

                        let feature_state = match val_ans.as_str() {
                            "On" => cluaiz_shared::hardware::schema::booster::FeatureState::On,
                            "Off" => cluaiz_shared::hardware::schema::booster::FeatureState::Off,
                            _ => cluaiz_shared::hardware::schema::booster::FeatureState::Auto,
                        };

                        match key_part.as_str() {
                            "Neural Mode" => {
                                let mut modes = vec![
                                    "edge".to_string(), 
                                    "multitasking".to_string(), 
                                    "balance".to_string(), 
                                    "max_boost".to_string(), 
                                    "ultra_max_boost".to_string()
                                ];

                                // 🌌 VRAM GUARD: Only show HyperCluster if VRAM >= 40GB
                                let total_vram = {
                                    let pulse_lock = state.live_pulse.pulse.read().unwrap();
                                    pulse_lock.vram_total_gb
                                };
                                if total_vram >= 40.0 {
                                    modes.push("hyper_cluster".to_string());
                                }

                                let selected_mode = match Select::new("Select Neural Mode:", modes)
                                    .with_render_config(config.clone())
                                    .prompt() {
                                    Ok(ans) => ans,
                                    Err(_) => continue,
                                };
                                booster.mode_run = match selected_mode.as_str() {
                                    "edge" => cluaiz_shared::hardware::schema::booster::BoosterMode::Edge,
                                    "multitasking" => cluaiz_shared::hardware::schema::booster::BoosterMode::Multitasking,
                                    "balance" => cluaiz_shared::hardware::schema::booster::BoosterMode::Balance,
                                    "max_boost" => cluaiz_shared::hardware::schema::booster::BoosterMode::MaxBoost,
                                    "ultra_max_boost" => cluaiz_shared::hardware::schema::booster::BoosterMode::UltraMaxBoost,
                                    "hyper_cluster" => cluaiz_shared::hardware::schema::booster::BoosterMode::HyperCluster,
                                    _ => cluaiz_shared::hardware::schema::booster::BoosterMode::Balance,
                                };
                            },
                            "Turbo Quant" => booster.turbo_quant = feature_state,
                            "Flash Attention" => booster.flash_attention = feature_state,
                            "Speculative Decoding" => booster.speculative_decoding = feature_state,
                            "Auto Round" => booster.auto_round = feature_state,
                            "DFlash (FlashKDA)" => {
                                booster.dflash = match val_ans.as_str() {
                                    "On" => cluaiz_shared::hardware::schema::booster::SmartState::Static("On".to_string()),
                                    "Off" => cluaiz_shared::hardware::schema::booster::SmartState::Static("Off".to_string()),
                                    _ => cluaiz_shared::hardware::schema::booster::SmartState::Static("Auto".to_string()),
                                };
                            },
                            "Force VRAM Reclaim" => booster.force_vram_reclaim = feature_state,
                            _ => {}
                        }
                        
                        if let Ok(_) = cluaiz_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster) {
                            println!("  {} System Booster updated: {} = {}", "✅".green(), key_part.cyan(), val_ans.bold());
                        } else {
                            println!("  {} Failed to save system booster settings.", "❌".red());
                        }
                    }
                    continue; // Go back to Master Menu after exiting System Booster
                }

                let downloaded: Vec<_> = state.sorted_models.iter().filter(|m| m.is_cached).collect();

                if downloaded.is_empty() {
                    println!(
                        "  {} No downloaded models found. Install from /menu.",
                        "ℹ️".blue()
                    );
                    return Ok(());
                }

                let options: Vec<String> = downloaded.iter().map(|m| m.manifest.name.clone()).collect();

                let starting_index = if let Some(active_id) = &state._active_model_id {
                    downloaded
                        .iter()
                        .position(|m| m.manifest.id == *active_id)
                        .unwrap_or(0)
                } else {
                    0
                };

                let selection = match Select::new("Switch to model:", options)
                    .with_render_config(config.clone())
                    .with_starting_cursor(starting_index)
                    .prompt() {
                    Ok(ans) => ans,
                    Err(inquire::InquireError::OperationCanceled) | Err(inquire::InquireError::OperationInterrupted) => {
                        print!("\x1B[1A\x1B[2K\r"); // Erase <canceled>
                        stdout().flush()?;
                        continue; // One step back (goes to Action menu)
                    }
                    Err(e) => return Err(e.into()),
                };

                print!("\x1B[1A\x1B[2K\r");
                stdout().flush()?;
                break selection; // Breaks the master loop and returns the selected model
            }
        };

        let downloaded: Vec<_> = state.sorted_models.iter().filter(|m| m.is_cached).collect();

        if let Some(model) = downloaded.iter().find(|m| m.manifest.name == ans) {
            if state._active_model_id.as_ref() == Some(&model.manifest.id) {
                println!(
                    "  {} {} is already active.",
                    "ℹ️".blue(),
                    model.manifest.name.bold()
                );
                return Ok(());
            }

            println!("  {} Loading {}, please wait a moment...", "⏳".yellow(), model.manifest.name.bold());

            if let Some(path_str) = &model.manifest.local_path {
                let path = std::path::PathBuf::from(path_str);

                // 🧬 Cluaiz DISPATCH:
                // High bit-depth -> Native Rust
                // 1-bit BitNet -> MANDATORY Llama (Binary)
                let runtime = if model.manifest.bit_depth < 2.0 {
                    cluaiz_shared::BackendType::RuntimeB
                } else {
                    cluaiz_shared::BackendType::RuntimeA
                };

                let result = tokio::task::block_in_place(|| {
                    let handle = tokio::runtime::Handle::current();
                    match handle.block_on(engines::CoreRouter::load_model(
                        path,
                        runtime.clone(),
                    )) {
                        Ok(router) => {
                            let mut lock = state.Core_engine.router.blocking_lock();
                            *lock = router;
                            Ok(())
                        }
                        Err(e) => {
                            // ⚠️ NATIVE FALLBACK: Only for standard models (Bit-depth >= 2.0)!
                            // BitNet MUST NOT use RuntimeA (Candle) as it will crash with tensor errors.
                            if runtime == cluaiz_shared::BackendType::RuntimeB
                                && model.manifest.bit_depth >= 2.0
                            {
                                let path_inner = std::path::PathBuf::from(path_str);
                                handle
                                    .block_on(engines::CoreRouter::load_model(
                                        path_inner,
                                        cluaiz_shared::BackendType::RuntimeA
                                    ))
                                    .map(|router| {
                                        let mut lock = state.Core_engine.router.blocking_lock();
                                        *lock = router;
                                    })
                            } else {
                                Err(e)
                            }
                        }
                    }
                });

                match result {
                    Ok(_) => {
                        model.manifest.id.clone();
                        state._active_model_id = Some(model.manifest.id.clone());
                        println!("  {} Mounted successfully.", "✅".green());
                    }
                    Err(e) => println!("  {} Load failed: {}", "❌".red(), e),
                }
            }
            state
                .activity_stream
                .push(ActivityBlock::ModelMounted(model.manifest.name.clone()));
        }

        Ok(())
    }
}
