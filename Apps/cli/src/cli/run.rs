use color_eyre::Result;
use colored::Colorize;
use engines::models::registry::CoreRoster;

/// `cluaiz run <model-id>` — pulls the model and initiates a native chat session.
pub async fn execute(model_id: &str, _interactive: bool) -> Result<()> {
    // 🎨 Display the Sovereign Logo
    let logo = crate::assets::logos::logo_gallery::LOGO_VARIANTS[9];
    println!("{}", logo.cyan());

    println!("\n  {} [Cluaiz] Initializing Sovereign Kernel for '{}'...", "⚙️".yellow(), model_id.bold());

    // 1. Resolve Metadata & Pull Model
    let roster = CoreRoster::load_roster();
    let mut manifest = roster.into_iter().find(|m| m.id.to_lowercase() == model_id.to_lowercase());

    if manifest.is_none() {
        println!("  {} Model missing in local vault. Synchronizing with Neural Registry...", "🌐".yellow());
        let remote_models = CoreRoster::fetch_external_registry(None).await.map_err(|e| color_eyre::eyre::eyre!(e))?;
        manifest = remote_models.into_iter().find(|m| m.id.to_lowercase() == model_id.to_lowercase());
    }

    let manifest = manifest.ok_or_else(|| color_eyre::eyre::eyre!("ID '{}' not found in any registry.", model_id))?;

    // 2. Provision Weights
    let home_dir = ::dirs::home_dir().ok_or_else(|| color_eyre::eyre::eyre!("Could not resolve Home Directory"))?;
    let cluaiz_root = home_dir.join(".cluaiz").join("models");
    let manager = engines::models::manager::ModelManager::new(engines::models::registry::REGISTRY_URL.to_string(), cluaiz_root.clone());

    manager.pull_model(model_id).await.map_err(|e| color_eyre::eyre::eyre!(e))?;

    // 3. Hardware Orchestration (The Neural Bridge)
    let safe_id = manifest.id.replace(':', "-");
    let model_path = cluaiz_root.join(&manifest.category).join(&safe_id);
    let model_file = model_path.join(&manifest.huggingface_filename);
    
    if !model_file.exists() {
        return Err(color_eyre::eyre::eyre!("Model file not found at: {:?}", model_file));
    }

    let dna = cluaiz_shared::StructuralDNA::default();
    let context = cluaiz_shared::CluaizContext::boot(dna, cluaiz_shared::TemplateManager::default());

    let engine = engines::runtime::execution::hub::HardwareOrchestrator::instantiate(
        model_file.to_str().unwrap(),
        context
    ).await.map_err(|e| color_eyre::eyre::eyre!(e))?;

    println!("  {} Handshake Success. Entering Sovereign Dashboard...\n", "✅".green());

    // 4. Launch Dashboard in Sovereign Mode (Pre-loaded with the model)
    use crate::core::state::AppState;
    use tokio::sync::mpsc;
    
    // 🧬 Load Real Tokenizer from the model folder
    let tokenizer_path = model_path.join("tokenizer.json");
    let tokenizer = if tokenizer_path.exists() {
        tokenizers::Tokenizer::from_file(&tokenizer_path).ok()
    } else {
        None
    };

    let mut state = AppState::new(None);
    // Pre-load the engine and tokenizer into the state
    {
        let mut lock = state.Core_engine.router.lock().await;
        lock.active_backend = engines::api::router::Backend::Cluaiz(engine);
        lock.tokenizer = tokenizer;
    }
    state._active_model_id = Some(manifest.id.clone());

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut mode = crate::app_enums::Mode::Running;

    // 🚀 Start the Dashboard UI
    crate::core::dashboard::DashboardEngine::run_native(
        &mut state,
        &tx,
        &mut rx,
        &mut mode
    )?;

    Ok(())
}

