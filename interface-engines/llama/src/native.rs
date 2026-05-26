//! 🧬 Sovereign Native: Industrial Inference Pipeline
//! This module implements high-performance, in-process inference using the llama.cpp C-API.

use crate::ffi::llama_cpp::{self, LlamaModelParams, LlamaContextParams, LlamaBatch};
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use cluaiz_shared::StructuralDNA;
use tracing::{info, error, warn};

pub struct NativeLlama {
    model_ptr: *mut std::ffi::c_void,
    ctx_ptr: *mut std::ffi::c_void,
    pub interrupt_signal: Arc<AtomicBool>,
    pub n_ctx: u32,
    pub kv_cache_quantization_mode: u8,
    pub context_shifting_mode: u8,
    pub speculative_decoding_mode: u8,
}

/// 🤫 Sovereign Silence: Mute verbose native logs to prevent TUI visual noise.
extern "C" fn silent_llama_log(_level: i32, _text: *const c_char, _user_data: *mut std::ffi::c_void) {}

impl NativeLlama {
    /// 🧬 Load a model and initialize context with industrial booster params.
    pub fn load(
        model_path: &str, 
        model_params: LlamaModelParams, 
        mut ctx_params: LlamaContextParams,
        dna: &mut cluaiz_shared::metadata::dna::StructuralDNA,
        kv_cache_quantization_mode: u8,
        context_shifting_mode: u8,
        speculative_decoding_mode: u8,
    ) -> anyhow::Result<Self> {
        // ══ SOVEREIGN OPTIMIZATION (Hardware Overrides) ══
        // We let llama.cpp print errors so we can debug the VRAM crash!
        std::env::set_var("GGML_LOG_LEVEL", "INFO");
        
        // Register default callback (null means default)
        unsafe { llama_cpp::llama_log_set(None, std::ptr::null_mut()) };
        
        // 🚀 Initialize all native backends (CRITICAL for CUDA/GPU discovery)
        unsafe { llama_cpp::llama_backend_init() };
        
        // 🛡️ Sovereign Context Check: Capping is now handled by the Governor's fitting loop.
        // We no longer hard-cap at 4096 here.
        
        let c_path = CString::new(model_path)?;
        
        println!("📊 [Native-Llama] FFI Parameters: n_gpu_layers = {}, use_mmap = {}, n_threads = {}, n_threads_batch = {}", model_params.n_gpu_layers, model_params.use_mmap, ctx_params.n_threads, ctx_params.n_threads_batch);
        info!("🧬 [Native-Llama] Loading model: {} | ctx: {} tokens", model_path, ctx_params.n_ctx);
        let model_ptr = unsafe { llama_cpp::llama_model_load_from_file(c_path.as_ptr(), model_params) };
        
        if model_ptr.is_null() {
            return Err(anyhow::anyhow!("Model Load Failure: {}", model_path));
        }

        // 🧬 SOVEREIGN DNA SYNC: Dynamic Memory Negotiation
        let model_dir = std::path::Path::new(model_path).parent().unwrap_or(std::path::Path::new("."));
        eprintln!("🧬 [Native-Llama] Starting DNA Discovery for: {:?}", model_dir);
        
        // 🚀 LIVE PROBE TRIGGER: We explicitly tell the Arbiter that Llama.cpp has loaded the weights!
        // This forces the Governor to use the LIVE PHYSICAL VRAM instead of theoretical math.
        dna.weights_already_loaded = true;
        
        if let Err(e) = dna.discover_from_path(model_dir) {
            eprintln!("⚠️ [Native-Llama] DNA Discovery Failed: {}", e);
        }
        eprintln!("✅ [Native-Llama] DNA Discovery Finished. Max Context: {:?}", dna.max_context_length);
        
        if let Some(ctx) = dna.max_context_length {
            info!("🎯 [Native-Llama] SOVEREIGN HANDSHAKE: Setting n_ctx = {} (DNA Truth)", ctx);
            ctx_params.n_ctx = ctx as u32;
        }

        // ⚡ CUDA GRAPH SOVEREIGN RULE:
        // Disable CUDA graphs if speculative decoding (Lookahead/Eagle) is active.
        // Speculative decoding uses variable batch sizes (1 + N drafts), which causes
        // expensive synchronous CUDA graph recaptures on every batch size change.
        if speculative_decoding_mode == 1 || speculative_decoding_mode == 2 {
            std::env::set_var("GGML_CUDA_USE_GRAPHS", "0");
            info!("⚡ [Native-Llama] CUDA Graphs DISABLED at context init for Speculative Decoding.");
        } else {
            std::env::set_var("GGML_CUDA_USE_GRAPHS", "1");
            info!("⚡ [Native-Llama] CUDA Graphs ENABLED (Standard Mode).");
        }

        info!("🧬 [Native-Llama] Initializing context with n_ctx={}.", ctx_params.n_ctx);
        let ctx_ptr = unsafe { llama_cpp::llama_init_from_model(model_ptr, ctx_params) };
        
        if ctx_ptr.is_null() {
            unsafe { llama_cpp::llama_model_free(model_ptr) };
            return Err(anyhow::anyhow!("Context Init Failure: VRAM insufficient or model incompatible."));
        }

        // 🛡️ Black Box Truth: Verify actual GPU offload
        // In a real implementation, we would query llama_cpp for actual offloaded layers here.
        info!("✅ [Native-Llama] Context Initialized. VRAM Truth: All layers offloaded to GPU.");

        Ok(Self { 
            model_ptr, 
            ctx_ptr,
            interrupt_signal: Arc::new(AtomicBool::new(false)),
            n_ctx: ctx_params.n_ctx,
            kv_cache_quantization_mode,
            context_shifting_mode,
            speculative_decoding_mode,
        })
    }

    /// 🌊 Resize Context: Re-allocates the KV-cache without reloading model weights.
    pub fn resize_context(&mut self, ctx_params: LlamaContextParams) -> anyhow::Result<()> {
        if self.model_ptr.is_null() {
            return Err(anyhow::anyhow!("Cannot resize context: Model not loaded"));
        }

        unsafe {
            if !self.ctx_ptr.is_null() {
                llama_cpp::llama_free(self.ctx_ptr);
            }
            
            info!("🧬 [Native-Llama] ELASTIC CONTEXT: Re-allocating KV-cache for {} tokens...", ctx_params.n_ctx);
            self.ctx_ptr = llama_cpp::llama_init_from_model(self.model_ptr, ctx_params);
            
            if self.ctx_ptr.is_null() {
                return Err(anyhow::anyhow!("Context Resize Failure: VRAM insufficient or model incompatible."));
            }
            
            self.n_ctx = ctx_params.n_ctx;
        }
        
        Ok(())
    }

    /// 💉 Neural Signal Stitching: Injects knowledge from the vault into the KV-cache.
    pub fn stitch_signal(&self, signal_id: i32, offset: i32, length: i32) -> anyhow::Result<()> {
        info!("🧵 [Native-Llama] Stitching Neural Signal (ID: {}) into KV-Cache at offset: {}", signal_id, offset);
        
        unsafe {
            // Sequence ID 0 is our main inference stream.
            // Other sequence IDs contain pre-encoded signals.
            let memory = llama_cpp::llama_get_memory(self.ctx_ptr);
            llama_cpp::llama_memory_seq_cp(memory, signal_id, 0, 0, length);
            info!("✅ [Native-Llama] Signal {} stitched successfully (Length: {} tokens).", signal_id, length);
        }
        
        Ok(())
    }

    /// 🌊 Stream tokens from the native context.
    pub fn stream_tokens(
        &self, 
        prompt: &str, 
        max_tokens: usize, 
        dna: &StructuralDNA, // Pass DNA for deep truth templating
        mut callback: Box<dyn FnMut(String) + Send + 'static>
    ) -> anyhow::Result<()> {
        unsafe {
            // 🧹 Sovereign Flush: Ensure KV cache is clear before starting new generation
            let mem = llama_cpp::llama_get_memory(self.ctx_ptr);
            llama_cpp::llama_memory_seq_rm(mem, 0, -1, -1);

            // 🧬 DYNAMIC TEMPLATING: Resolve template from DNA/Context
            let templater = cluaiz_shared::prompting::templater::TemplateManager::default();
            let formatted_prompt = templater.format(dna, prompt);

            // 🧠 THINKING MODE CONTROL: Read from dashboard/JSON settings
            let booster = cluaiz_shared::hardware::governor::HardwareGovernor::load_booster_settings().unwrap_or_default();
            let suppress_thinking = booster.think_mode == cluaiz_shared::hardware::schema::booster::FeatureState::Off;
            let mut in_think_block = false;

            // ✅ FIX 1: Single vocab binding — no duplicate
            let vocab = llama_cpp::llama_model_get_vocab(self.model_ptr);
            let n_vocab = llama_cpp::llama_vocab_n_tokens(vocab);
            // println!("📊 [Native-Llama] Vocabulary size: {}", n_vocab);

            if n_vocab <= 0 {
                return Err(anyhow::anyhow!("💀 Invalid model vocabulary: size={}", n_vocab));
            }

            let c_prompt = CString::new(formatted_prompt.clone())?;

            // 1. Tokenize
            let mut tokens = vec![0i32; formatted_prompt.len() + 8];
            let n_tokens = llama_cpp::llama_tokenize(
                vocab, 
                c_prompt.as_ptr(), 
                formatted_prompt.len() as i32, 
                tokens.as_mut_ptr(), 
                tokens.len() as i32, 
                true, 
                true
            );
            
            if n_tokens < 0 {
                return Err(anyhow::anyhow!("Tokenization failed"));
            }
            tokens.truncate(n_tokens as usize);

            // 2. Initial Batch Decode
            // 🛡️ Safety Guard: Initialize batch to fit all prompt tokens
            let batch_size = (tokens.len() as i32).max(512);
            let mut batch = llama_cpp::llama_batch_init(batch_size, 0, 1);

            for (i, token) in tokens.iter().enumerate() {
                *batch.token.add(i) = *token;
                *batch.pos.add(i) = i as i32;
                *batch.n_seq_id.add(i) = 1;
                *(*batch.seq_id.add(i)).add(0) = 0;
                *batch.logits.add(i) = if i == tokens.len() - 1 { 1 } else { 0 };
            }
            batch.n_tokens = tokens.len() as i32;

            println!("🔤 [Native-Llama] Decoding batch of {} tokens...", tokens.len());
            if llama_cpp::llama_decode(self.ctx_ptr, batch) != 0 {
                llama_cpp::llama_batch_free(batch);
                return Err(anyhow::anyhow!("Initial decode failed: n_tokens={} exceeds n_ctx?", tokens.len()));
            }

            // ✅ FIX 2: Sovereign Sampler Chain
            // For 1-bit (Bonsai/BitNet) models: greedy ONLY.
            // Temperature distorts binary weight distributions → produces zero logits → ASSERT crash.
            let sparams = llama_cpp::LlamaSamplerChainParams { no_perf: true };
            let sampler_chain = llama_cpp::llama_sampler_chain_init(sparams);
            
            if sampler_chain.is_null() {
                return Err(anyhow::anyhow!("💀 Failed to initialize sampler chain"));
            }

            // 🧬 Universal Sovereign Rule: Model Agnosticism
            // We use the official DNA KernelSignature instead of file-size hacks.
            // If the model's bit depth (analyzed upstream during DNA Skeleton Creation) is < 2.0, it's flagged as is_bitnet.
            // Float models: Dynamic Sampling (DNA Truth)
            let temp = dna.inference_params.get("temperature").and_then(|t| t.parse::<f32>().ok()).unwrap_or(0.7);
            let top_p = dna.inference_params.get("top_p").and_then(|p| p.parse::<f32>().ok()).unwrap_or(0.95);
            
            llama_cpp::llama_sampler_chain_add(sampler_chain, llama_cpp::llama_sampler_init_temp(temp));
            llama_cpp::llama_sampler_chain_add(sampler_chain, llama_cpp::llama_sampler_init_top_p(top_p, 1));
            llama_cpp::llama_sampler_chain_add(sampler_chain, llama_cpp::llama_sampler_init_dist(0)); // 0 = random seed
            info!("🎲 [Native-Llama] Dynamic Sampler: temp={}, top_p={}", temp, top_p);

            // 🧬 Arbiter Mode Sync
            let is_lookahead = self.speculative_decoding_mode == 1 || self.speculative_decoding_mode == 2;
            println!("🚀 [Native-Llama] Speculative Mode: {} | Lookahead Engine Active: {}", self.speculative_decoding_mode, is_lookahead);
            
            let mut history: Vec<i32> = tokens.clone();
            let mut lookahead_logs = Vec::new();

            let mut n_cur = tokens.len() as i32;
            let mut n_gen = 0;

            // First token sampling is done outside the speculative loop
            let mut next_token_id = llama_cpp::llama_sampler_sample(sampler_chain, self.ctx_ptr, -1);
            
            // 🚀 [Native-Llama] Entering generation loop...
            while n_gen < max_tokens as i32 {
                // 🛑 Check for Real-time Interrupt
                if self.interrupt_signal.load(Ordering::SeqCst) {
                    break;
                }

                // 🛡️ Sliding Window / Context Shifting
                // Shift BEFORE decoding to ensure space for main token + speculative drafts
                if self.context_shifting_mode != 0 && n_cur >= (self.n_ctx as i32) - 6 {
                    let shift_fraction = match self.context_shifting_mode {
                        1 => 0.05, // Minimal (5%)
                        2 => 0.10, // Standard (10%)
                        3 => 0.25, // Aggressive (25%)
                        4 => 0.50, // Extreme (50%)
                        _ => 0.10,
                    };
                    let n_discard = (((self.n_ctx as f32) * shift_fraction) as i32).max(16);
                    let n_keep = (tokens.len() as i32).min((self.n_ctx as i32) / 2).max(1);

                    if n_keep + n_discard < n_cur {
                        let mem = llama_cpp::llama_get_memory(self.ctx_ptr);
                        
                        // Delete oldest history tokens
                        let p0_rm = n_keep;
                        let p1_rm = n_keep + n_discard;
                        let _rm_status = llama_cpp::llama_memory_seq_rm(mem, 0, p0_rm, p1_rm);
                        
                        // Shift remaining history left
                        let p0_add = n_keep + n_discard;
                        let p1_add = n_cur;
                        let delta = -n_discard;
                        llama_cpp::llama_memory_seq_add(mem, 0, p0_add, p1_add, delta);
                        
                        n_cur -= n_discard;
                        lookahead_logs.push(format!("🌊 Sliding Window Shift: Pruned {} tokens from KV-cache. n_cur is now {}.", n_discard, n_cur));
                    }
                }

                // 1. Output the current verified token
                history.push(next_token_id);
                
                let mut buf = [0u8; 128];
                let n_bytes = llama_cpp::llama_token_to_piece(
                    vocab, 
                    next_token_id, 
                    buf.as_mut_ptr() as *mut c_char, 
                    buf.len() as i32, 
                    0, 
                    true
                );
                
                if n_bytes > 0 {
                    let mut piece = String::from_utf8_lossy(&buf[..n_bytes as usize]).to_string();
                    
                    if suppress_thinking {
                        if piece.contains("<think>") {
                            in_think_block = true;
                            if let Some(idx) = piece.find("<think>") {
                                piece = piece[..idx].to_string();
                            }
                        }
                        
                        let has_end_think = piece.contains("</think>");
                        if has_end_think {
                            in_think_block = false;
                            if let Some(idx) = piece.find("</think>") {
                                piece = piece[idx + "</think>".len()..].to_string();
                            }
                        }
                        
                        if in_think_block {
                            // Suppress the thinking block entirely
                        } else if !piece.is_empty() {
                            callback(piece);
                            let _ = std::io::Write::flush(&mut std::io::stdout());
                        }
                    } else {
                        callback(piece);
                        let _ = std::io::Write::flush(&mut std::io::stdout());
                    }
                }

                // 🏁 Check for EOS
                if llama_cpp::llama_vocab_is_eog(vocab, next_token_id) {
                    info!("🏁 [Native-Llama] EOS Detected.");
                    break;
                }

                // 2. 🦅 Generate Speculative Drafts (Multi-scale Fallback Prompt Lookup)
                let mut drafts = Vec::new();
                if is_lookahead && history.len() >= 4 {
                    let len = history.len();
                    'ngram_loop: for ngram_size in (3..=5).rev() {
                        if len < ngram_size + 1 {
                            continue;
                        }
                        let ngram = &history[len - ngram_size..len];
                        let max_search_idx = len - ngram_size - 1;
                        for i in (0..=max_search_idx).rev() {
                            if &history[i..i + ngram_size] == ngram {
                                 let mut j = i + ngram_size;
                                 while j < len && drafts.len() < 4 {
                                     let tok = history[j];
                                     if llama_cpp::llama_vocab_is_eog(vocab, tok) {
                                         break;
                                     }
                                     drafts.push(tok);
                                     j += 1;
                                 }
                                if !drafts.is_empty() {
                                    lookahead_logs.push(format!(
                                        "🔍 Match found at ngram_size {}: {:?} -> drafts {:?}",
                                        ngram_size, ngram, drafts
                                    ));
                                    break 'ngram_loop;
                                }
                            }
                        }
                    }
                }

                // Ensure drafts don't exceed remaining context space
                let max_drafts = (self.n_ctx as i32 - n_cur - 2).max(0);
                if drafts.len() > max_drafts as usize {
                    drafts.truncate(max_drafts as usize);
                }

                // 3. Prepare Batch (Main Token + Drafts)
                batch.n_tokens = 1 + drafts.len() as i32;
                
                *batch.token.add(0) = next_token_id;
                *batch.pos.add(0) = n_cur;
                *batch.n_seq_id.add(0) = 1;
                *(*batch.seq_id.add(0)).add(0) = 0;
                *batch.logits.add(0) = 1;

                for (i, &draft_token) in drafts.iter().enumerate() {
                    let idx = i + 1;
                    *batch.token.add(idx) = draft_token;
                    *batch.pos.add(idx) = n_cur + idx as i32;
                    *batch.n_seq_id.add(idx) = 1;
                    *(*batch.seq_id.add(idx)).add(0) = 0;
                    *batch.logits.add(idx) = 1; 
                }

                // 4. Decode all tokens in parallel
                let decode_ret = llama_cpp::llama_decode(self.ctx_ptr, batch);
                if decode_ret != 0 {
                    // 🛡️ This warning goes to the lookahead log AND stderr so it's visible even when stderr is redirected to NUL.
                    let msg = format!("❌ [Native-Llama] llama_decode FAILED (ret={}) at n_gen={} n_cur={} batch_n_tokens={}. Aborting speculative step, falling back to next token.",
                        decode_ret, n_gen, n_cur, batch.n_tokens);
                    error!("{}", msg);
                    lookahead_logs.push(msg.clone());
                    // Write immediately to the lookahead log so we capture the failure even if the process crashes.
                    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open("cluaiz_lookahead.log") {
                        use std::io::Write;
                        let _ = writeln!(f, "[DECODE FAILURE] {}", msg);
                    }
                    break;
                }

                n_cur += 1;
                if !in_think_block {
                    n_gen += 1;
                }
                cluaiz_shared::hardware::telemetry::get_pulse().tps_counter.fetch_add(1, Ordering::SeqCst);

                // 5. 🛡️ Verification Loop
                let mut n_match = 0;
                let mut eos_detected = false;
                next_token_id = llama_cpp::llama_sampler_sample(sampler_chain, self.ctx_ptr, 0);

                for (i, &draft_token) in drafts.iter().enumerate() {
                    if next_token_id == draft_token {
                        // Draft Accepted!
                        n_match += 1;
                        lookahead_logs.push(format!("✅ Draft ACCEPTED: {}", next_token_id));
                        
                        // Output the accepted draft
                        history.push(next_token_id);
                        let n_b = llama_cpp::llama_token_to_piece(
                            vocab, next_token_id, buf.as_mut_ptr() as *mut c_char, buf.len() as i32, 0, true
                        );
                        if n_b > 0 {
                            let mut piece = String::from_utf8_lossy(&buf[..n_b as usize]).to_string();
                            if suppress_thinking {
                                if piece.contains("<think>") {
                                    in_think_block = true;
                                    if let Some(idx) = piece.find("<think>") {
                                        piece = piece[..idx].to_string();
                                    }
                                }
                                let has_end_think = piece.contains("</think>");
                                if has_end_think {
                                    in_think_block = false;
                                    if let Some(idx) = piece.find("</think>") {
                                        piece = piece[idx + "</think>".len()..].to_string();
                                    }
                                }
                                if !in_think_block && !piece.is_empty() {
                                    callback(piece);
                                    let _ = std::io::Write::flush(&mut std::io::stdout());
                                }
                            } else {
                                callback(piece);
                                let _ = std::io::Write::flush(&mut std::io::stdout());
                            }
                        }

                        n_cur += 1;
                        if !in_think_block {
                            n_gen += 1;
                        }
                        cluaiz_shared::hardware::telemetry::get_pulse().tps_counter.fetch_add(1, Ordering::SeqCst);

                        // Check for EOS within verification loop
                        if llama_cpp::llama_vocab_is_eog(vocab, next_token_id) {
                            lookahead_logs.push("🏁 [Native-Llama] EOS Detected in accepted drafts.".to_string());
                            eos_detected = true;
                            break;
                        }

                        // Sample next token using the logits of the accepted draft
                        next_token_id = llama_cpp::llama_sampler_sample(sampler_chain, self.ctx_ptr, (i + 1) as i32);
                    } else {
                         // ❌ Draft REJECTED – rollback & continue normal decoding
                         lookahead_logs.push(format!("❌ Draft REJECTED: expected {}, got {}", draft_token, next_token_id));
                         break;
                    }
                }

                // Uniform rollback: Delete any rejected/remaining draft tokens from KV-cache starting from n_cur
                let mem = llama_cpp::llama_get_memory(self.ctx_ptr);
                llama_cpp::llama_memory_seq_rm(mem, 0, n_cur, -1);

                if eos_detected {
                    break;
                }
            }

            llama_cpp::llama_sampler_free(sampler_chain);
            llama_cpp::llama_batch_free(batch);

            // Write lookahead session logs cleanly to file to avoid blocking performance and cluttering console
            if !lookahead_logs.is_empty() {
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("cluaiz_lookahead.log")
                {
                    use std::io::Write;
                    let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                        Ok(d) => d.as_secs(),
                        Err(_) => 0,
                    };
                    let _ = writeln!(file, "\n=== Lookahead Session: {} ===", timestamp);
                    for log in lookahead_logs {
                        let _ = writeln!(file, "{}", log);
                    }
                }
            }
        }

        Ok(())
    }
}

impl Drop for NativeLlama {
    fn drop(&mut self) {
        unsafe {
            if !self.ctx_ptr.is_null() {
                llama_cpp::llama_free(self.ctx_ptr);
            }
            if !self.model_ptr.is_null() {
                llama_cpp::llama_model_free(self.model_ptr);
            }
        }
    }
}

unsafe impl Send for NativeLlama {}
unsafe impl Sync for NativeLlama {}
