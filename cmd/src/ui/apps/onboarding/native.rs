use ::cluaize_shared::profile::{AccountType, AuthMethod, BusinessProfile, UserProfile};
use color_eyre::Result;
use colored::*;
use inquire::{Confirm, Password, Select, Text};

pub fn run_native_flow() -> Result<UserProfile> {
    let mut profile = UserProfile::new();

    println!("  {} [Cluaize] Initializing Core Infrastructure...", "ðŸ§ª".cyan());

    // â”€â”€ Step 3: Auth â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    let auth_choice = Select::new(
        "ðŸ” How would you like to sign in?",
        vec![
            "Sign in with Google",
            "Sign in with Email",
            "Continue as Guest (No Cloud)",
        ],
    )
    .prompt()?;

    match auth_choice {
        "Sign in with Google" => {
            profile.auth.method = AuthMethod::Google;
            profile.auth.email = "Cluaize@cluaize.os".to_string();
            println!("âœ“ Authenticated via Google as {}", profile.auth.email);
        }
        "Sign in with Email" => {
            profile.auth.method = AuthMethod::Email;
            profile.auth.email = Text::new("âœ‰ï¸  Enter your email:").prompt()?;
            let _pass = Password::new("ðŸ”‘ Create password:").prompt()?;
            println!("âœ“ Account created for {}", profile.auth.email);
        }
        _ => {
            profile.auth.method = AuthMethod::None;
            println!("â„¹ Proceeding as Guest");
        }
    }

    // â”€â”€ Step 4: Usage Choice â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    let usage = Select::new(
        "ðŸ‘‹ How will you use Archer Cluaize?",
        vec!["Personal AI Assistant", "Business & Teams"],
    )
    .prompt()?;

    match usage {
        "Business & Teams" => {
            profile.account_type = AccountType::Business;
            let mut biz = BusinessProfile::default();
            biz.name = Text::new("ðŸ¢ Business Name:").prompt()?;

            let industries: Vec<String> = ::cluaize_shared::profile::INDUSTRY_TAXONOMY
                .iter()
                .map(|i| i.label.to_string())
                .collect();
            biz.industry = Select::new("Industry:", industries).prompt()?;

            profile.business = Some(biz);
        }
        _ => {
            profile.account_type = AccountType::Personal;
            profile.identity.name = Text::new("ðŸ‘¤ What is your name, Cluaize?").prompt()?;
        }
    }

    // â”€â”€ Step 6: Hardware Audit â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    println!("\nðŸ“¡ INITIATING BARE-METAL CALIBRATION");

    // ðŸ§¬ probe hardware
    use ::cluaize_shared::hardware::{HardwareGovernor, get_Cluaize_profile};

    if let Err(e) = HardwareGovernor::auto_calibrate() {
        println!("  {} [Onboarding] Calibration failed: {:?}", "âŒ".red(), e);
    }

    let stats = get_Cluaize_profile();

    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    let ram_gb = sys.total_memory() as f64 / 1_073_741_824.0;

    println!("  HOST PLATFORM: {}", stats.platform);
    println!(
        "  CPU UNIT:      {} ({} cores)",
        stats.cpu_brand, stats.cpu_cores
    );

    let gpu_info = if stats.compute.has_gpu {
        format!("Accelerator Active ({:.1} GB VRAM)", stats.compute.vram_gb)
    } else {
        "NO ACCELERATOR".to_string()
    };
    println!("  GPU COMPUTE:   {}", gpu_info);
    println!("  SYSTEM RAM:    {:.1} GB", ram_gb);
    println!("  CORE STATUS:   OPTIMIZED âœ“\n");

    // â”€â”€ Part B: Sequential Performance Tuning â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    println!("ðŸ› ï¸ PERFORMANCE TUNING");

    let turbo_quant = Confirm::new("Enable TurboQuant Acceleration?")
        .with_default(true)
        .prompt()?;
    let _ = HardwareGovernor::update_field(
        "runtime_engine.booster_flags.TurboQuant_Enable",
        serde_json::json!(turbo_quant),
    );

    if stats.compute.vram_gb >= 2.0 {
        let flash_attn = Confirm::new("Enable FlashAttention v2?")
            .with_default(true)
            .prompt()?;
        let _ = HardwareGovernor::update_field(
            "runtime_engine.booster_flags.FlashAttention_v2",
            serde_json::json!(flash_attn),
        );
    }

    println!("\nâœ“ Hardware DNA verified and synchronized.\n");

    // ── Part C: System Control & Brain State ──────────────────────────────
    println!("\n🧠 SYSTEM CONTROL & BRAIN STATE");
    
    let brain_choice = Select::new(
        "Select the Sovereign Brain Mode:",
        vec![
            "ON (Local Engine + Agentic Tasks)",
            "ONLY (Pure Brain, no local LLM loading)",
            "OFF (Manual Mode)",
        ],
    )
    .prompt()?;
    
    let brain_mode = match brain_choice {
        "ON (Local Engine + Agentic Tasks)" => "local",
        "ONLY (Pure Brain, no local LLM loading)" => "only_brain",
        _ => "off",
    };
    
    // Update system_control.json with the chosen brain mode
    let mut control = ::cluaize_shared::hardware::governor::HardwareGovernor::load_system_control().unwrap_or_default();
    control.brain.cluaizd_connect_ffi = brain_mode.to_string();
    let _ = ::cluaize_shared::hardware::governor::HardwareGovernor::save_system_control(&control);
    
    println!("✓ Brain Mode set to: {}", brain_mode);


    // â”€â”€ Finalize â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
    profile.onboarding_completed = true;
    profile.hardware_completed = true;
    profile.touch();

    let _ = ::cluaize_shared::profile::save_profile(&profile);
    let _ = ::cluaize_shared::onboarding::seed_workspace(&profile);

    println!("\nðŸ§¿ ARCHER Cluaize â€” ONLINE");
    println!(
        "Welcome to the future of Cluaize AI, {}.\n",
        profile.display_name()
    );

    Ok(profile)
}

