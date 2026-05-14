use std::collections::HashMap;

/// 🎭 TemplateManager: Handles neural prompt formatting based on model DNA.
#[derive(Debug, Clone)]
pub struct TemplateManager {
    pub templates: HashMap<String, String>,
}

// Default fallback templates if discovery fails (universal baselines)
const FALLBACK_CHATML: &str = "<|im_start|>user\n{{prompt}}<|im_end|>\n<|im_start|>assistant\n";
const FALLBACK_LLAMA3: &str = "<|begin_of_text|><|start_header_id|>user<|end_header_id|>\n\n{{prompt}}<|eot_id|><|start_header_id|>assistant<|end_header_id|>\n\n";

impl Default for TemplateManager {
    fn default() -> Self {
        Self { templates: HashMap::new() }
    }
}

impl TemplateManager {
    pub fn format(&self, dna: &crate::metadata::dna::StructuralDNA, prompt: &str) -> String {
        // 1. Priority: Use discovered template from DNA (JSON driven)
        if let Some(ref template) = dna.chat_template {
            // Normalize Jinja2-style templates to our internal format
            // If it contains messages iteration, we'll try to simplify it for single prompt
            if template.contains("<|im_start|>") {
                return FALLBACK_CHATML.replace("{{prompt}}", prompt);
            }
            if template.contains("<|start_header_id|>") {
                return FALLBACK_LLAMA3.replace("{{prompt}}", prompt);
            }
            if template.contains("<turn|>") {
                return format!("<turn|>user\n{}<turn|>assistant\n", prompt);
            }
            if template.contains("<start_of_turn>") {
                return format!("<start_of_turn>user\n{}<end_of_turn>\n<start_of_turn>model\n", prompt);
            }
        }

        // 2. Secondary: Guess by architecture name
        let arch = dna.model_identity.to_lowercase();
        let final_template = if arch.contains("llama") {
            FALLBACK_LLAMA3
        } else if arch.contains("gemma") {
            "<start_of_turn>user\n{{prompt}}<end_of_turn>\n<start_of_turn>model\n"
        } else {
            FALLBACK_CHATML // Qwen / ChatML default
        };

        final_template.replace("{{prompt}}", prompt)
    }
}
