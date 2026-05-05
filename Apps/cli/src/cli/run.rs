use color_eyre::Result;
use colored::Colorize;

/// `cluaiz run <model-id>` — pulls the model (downloads if missing), then confirms ready.
pub async fn execute(model_id: &str) -> Result<()> {
    println!(
        "\n  {} Silicon Dispatch: Pulling '{}'\n",
        "🧬".cyan(),
        model_id.bold()
    );

    let manager = engines::models::manager::ModelManager::new(
        engines::models::registry::REGISTRY_URL.to_string(),
        std::path::PathBuf::from("models"),
    );

    match manager.pull_model(model_id).await {
        Ok(_) => {
            println!(
                "  {} Model '{}' is ready.\n  {} Run 'cluaiz' to open the dashboard.",
                "✅".green(),
                model_id.bold(),
                "💡".cyan()
            );
        }
        Err(e) => {
            println!("  {} Dispatch failed: {}", "❌".red(), e);
        }
    }

    Ok(())
}
