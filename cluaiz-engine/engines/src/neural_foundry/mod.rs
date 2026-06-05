// cluaiz-engine: Core Foundry - The Cluaiz Engine Core
// Final integration of Registry, Intelligence, Runtime, and Security.

pub mod registry;
pub mod intelligence;
pub mod runtime;
pub mod security;
pub mod ingestion;

use registry::SkillRegistry;
use intelligence::skill_router::SkillRouter;
use runtime::wasm_host::WasmHost;
use runtime::mcp_gateway::McpGateway;
use security::guard::{PermissionGuard, PermissionLevel};
use tracing::{info, warn};
use cluaiz_shared::hardware::memory::kv_cache::stitching::CluaizSignal;
use neural_core::interfaces::memory_contract::MappedBuffer;
use std::sync::{Mutex, Arc};

// Removed fixed MAX_ACTIVE_SKILLS limit. Bounding is now purely dynamic based on hardware capacity.
// Constant threshold for the ONNX semantic similarity match.
const SIMILARITY_THRESHOLD: f32 = 0.8;

pub struct IntentResult {
    pub responses: Vec<String>,
    pub signals: Vec<CluaizSignal>,
}

pub struct CoreFoundry {
    pub registry: SkillRegistry,
    pub router: SkillRouter,
    pub wasm_runtime: WasmHost,
    pub mcp_gateway: McpGateway,
    pub guard: PermissionGuard,
    pub active_skill_ids: Mutex<Vec<String>>,
}

impl Default for CoreFoundry {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreFoundry {
    pub fn new() -> Self {
        Self {
            registry: SkillRegistry::new(),
            router: SkillRouter::new(),
            wasm_runtime: WasmHost::new(),
            mcp_gateway: McpGateway::new(),
            guard: PermissionGuard::new(),
            active_skill_ids: Mutex::new(Vec::new()),
        }
    }

    /// Initializes the foundry by scanning the skills directory.
    pub fn initialize(&mut self, skills_dir: &str) {
        println!("[CLUAIZ] Initializing Core Foundry from: {}", skills_dir);
        self.registry.load_from_directory(skills_dir);
    }

    /// The Cluaiz Flow: Prompt -> Multi-Route -> Execute
    pub async fn process_intent(&self, prompt: &str) -> anyhow::Result<IntentResult> {
        let skill_ids = self.router.match_intent(prompt, &self.registry);
        let mut result = IntentResult { responses: Vec::new(), signals: Vec::new() };

        if skill_ids.is_empty() {
            return Ok(result);
        }

        info!("🧬 [CoreFoundry] Multi-Skill Fusion Active: {} skills detected.", skill_ids.len());

        for (i, skill_id) in skill_ids.iter().enumerate() {
            // Note: Bounding is no longer hardcoded by active skills count limit, 
            // but is bounded below strictly by the dynamic RAM capacity pulse lock.

            // 1. Fetch skill from registry
            let skill = match self.registry.skills.iter().find(|s| &s.manifest.id == skill_id) {
                Some(s) => s,
                None => continue,
            };
            
            // 2. Dynamic Memory Management (RAM/VRAM Bounding)
            {
                // Fetch real-time hardware telemetry to ensure we don't cause OOM.
                let pulse = cluaiz_shared::hardware::telemetry::get_pulse();
                let pulse_lock = pulse.pulse.read().unwrap();
                let used_mb = pulse_lock.ram.used_gb * 1024.0;
                let util = pulse_lock.ram.utilization_pct as f64;
                let available_ram_mb = if util > 0.1 {
                    (used_mb / (util / 100.0)) - used_mb
                } else {
                    8192.0 // Fallback to 8GB free if telemetry is spinning up
                };
                
                // Estimate skill size (mock 50MB per skill KV cache for architecture demonstration)
                let skill_est_size_mb = 50.0; 

                let mut active_ids = self.active_skill_ids.lock().unwrap();
                
                // If adding this skill exceeds safe bounds, perform LRU eviction dynamically
                while (active_ids.len() as f32 + 1.0) * skill_est_size_mb >= available_ram_mb as f32 * 0.8 {
                    if !active_ids.is_empty() {
                        let evicted_id = active_ids.remove(0);
                        println!("[CLUAIZ] [VRAM] Bounding limit hit. Evicting LRU skill: {}", evicted_id);
                    } else {
                        break;
                    }
                }

                active_ids.retain(|id| id != skill_id);
                active_ids.push(skill_id.to_string());
            }

            // 3. Map Cluaiz Signal (Zero-Copy Dual-Cache)
            let permissions = crate::neural_foundry::security::permission_schema::PermissionSchema::load();
            
            if let Some(gen_model) = permissions.get_active_chat_model() {
                let gen_model_safe = gen_model.replace(":", "-");
                let cache_dir = skill.path.join(".cache");
                let kv_cache_path = cache_dir.join(format!("{}.kvcache.bin", gen_model_safe));
                
                if kv_cache_path.exists() {
                    if let Ok(mapped_buffer) = MappedBuffer::from_file(&kv_cache_path) {
                        result.signals.push(CluaizSignal {
                            raw_data: Arc::new(mapped_buffer),
                            token_count: skill.manifest.Core_metadata.as_ref().map_or(0, |m| m.token_count),
                            head_dim: skill.manifest.Core_metadata.as_ref().map_or(0, |m| m.head_dim),
                        });
                    }
                } else {
                    warn!("⚠️ [CoreFoundry] {} missing for skill {}. Sovereign Compiler Daemon should generate this in the background.", kv_cache_path.display(), skill_id);
                }
            } else {
                warn!("⚠️ [CoreFoundry] No text model assigned in Permission.json. Skipping Zero-Copy injection for skill {}.", skill_id);
            }
            
            // 4. Logic execution (WASM)
            self.guard.validate_action(&skill.manifest, PermissionLevel::ReadOnly)?;
            let logic_path = skill.path.join("logic.wasm");
            if logic_path.exists() {
                match self.wasm_runtime.execute_skill_logic(&logic_path, "run", prompt).await {
                    Ok(resp) => result.responses.push(resp),
                    Err(e) => tracing::error!("⚠️ [CoreFoundry] Logic failed for '{}': {}", skill_id, e),
                }
            }
        }
        
        Ok(result)
    }
}
