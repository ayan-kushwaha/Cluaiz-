use minijinja::{Environment, context};

#[derive(Debug, Clone)]
pub struct TemplateManager {
    pub jinja_template: String,
    pub is_fallback: bool,
}

impl TemplateManager {
    pub fn apply_chat_template(&self, prompt: &str) -> String {
        // "Agnostic Fallback Template" (Crash Safety)
        let fallback = format!("<bos><|turn>user\n{}<turn|>\n<|turn>model\n", prompt.trim());

        if self.is_fallback || self.jinja_template.is_empty() {
            return fallback;
        }
        
        let mut env = Environment::new();
        if env.add_template("chat", &self.jinja_template).is_err() {
            return fallback;
        }
        
        // Define standard message payload using minijinja native structures
        let messages = vec![
            context! {
                role => "user",
                content => prompt.trim(),
            }
        ];

        let rendered = match env.get_template("chat") {
            Ok(tmpl) => tmpl.render(context! {
                messages => messages,
                add_generation_prompt => true,
                enable_thinking => true,
            }),
            Err(_) => return fallback,
        };

        match rendered {
            Ok(output) => output,
            Err(e) => {
                tracing::warn!("⚠️ Jinja Render failed: {e}. Reverting to fallback.");
                fallback
            }
        }
    }
}
