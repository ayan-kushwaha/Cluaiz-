//! archer-top: The Sovereign Silicon Watchtower (CLI).
//! High-speed, terminal-native monitoring for the CURE Engine.

use archer_shared::hardware::telemetry;
use std::io::{stdout, Write};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let sensor = telemetry::get_pulse();
    let mut stdout = stdout();

    println!("\x1B[2J\x1B[H"); // Clear screen
    println!("🧿 ARCHER SILICON WATCHTOWER V6.0 - [GHOST MODE ACTIVE]");
    println!("══════════════════════════════════════════════════════════");

    loop {
        // 1. Data Collection
        let per_core_readings: Vec<u32> = sensor
            .per_core_usage
            .iter()
            .map(|c| c.load(Ordering::Relaxed))
            .collect();
        let vram_usage = sensor.vram_pressure_pct.load(Ordering::Relaxed);
        let reading_celsius = sensor.relay_latency_ms.load(Ordering::Relaxed); // Simplified for diagnostic

        // 2. Render UI (ANSI Express)
        print!("\x1B[H"); // Move to top
        println!("🧿 ARCHER SILICON WATCHTOWER V6.0 - Silicon Pulse Target: LOCAL");
        println!("══════════════════════════════════════════════════════════");

        // CPU Grid (Per-Core Audit)
        println!("\n[CPU CORE AUDIT]");
        for (core_index, usage_reading) in per_core_readings.iter().enumerate() {
            let usage_bar = render_bar(*usage_reading as u32, 20);
            print!(
                " Core {:02} [{}] {:.1}%   ",
                core_index, usage_bar, usage_reading
            );
            if (core_index + 1) % 2 == 0 {
                println!();
            }
        }

        // VRAM & Global Sensors
        println!("\n\n[SILICON DIODES]");
        let vram_bar = render_bar(vram_usage, 40);
        println!(" VRAM Pressure: [{}] {}%", vram_bar, vram_usage);
        println!(" CPU Thermal:   {}°C", reading_celsius);

        // Neural Metrics (Placeholder for Engine Link)
        println!("\n[NEURAL KERNEL METRICS]");
        println!(" Relay Latency:  -- ms  (Waiting for Engine Pulse...)");
        println!(" context Cache:  -- MB  (Waiting for Engine Pulse...)");
        println!(" Disk Load:      -- MB/s (Waiting for Engine Pulse...)");

        println!("\n══════════════════════════════════════════════════════════");
        println!(" [Ctrl+C] to exit 'Ghost Mode'");

        stdout.flush()?;
        thread::sleep(Duration::from_millis(250));
    }
}

fn render_bar(percentage: u32, bar_length: usize) -> String {
    let filled_segments = (percentage as f64 / 100.0 * bar_length as f64).round() as usize;
    let mut bar_render = String::with_capacity(bar_length);
    for segment_index in 0..bar_length {
        if segment_index < filled_segments {
            bar_render.push('█');
        } else {
            bar_render.push('░');
        }
    }
    bar_render
}
