use crate::ffi::llama_cpp;
use crate::native::core::NativeLlama;
use cluaiz_shared::StructuralDNA;
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::atomic::Ordering;
use tracing::{info, error, warn};

pub static mut SKIP_PTR: *const std::sync::atomic::AtomicBool = std::ptr::null();

pub fn stream_tokens(
    llama: &NativeLlama,
    prompt: &str, 
    max_tokens: usize, 
    dna: &StructuralDNA,
    mut callback: Box<dyn FnMut(String) -> bool + Send + 'static>
) -> anyhow::Result<()> {
    unsafe {
        // 🛑 ROOT FIX: Reset interrupt signal when entering generation to ensure pivot works!
        llama.interrupt_signal.store(false, Ordering::SeqCst);
        
        let is_pivot = prompt.starts_with("[PIVOT_CONTINUE]");
        let actual_prompt = if is_pivot {
            prompt.trim_start_matches("[PIVOT_CONTINUE]").trim_start().to_string()
        } else {
            prompt.to_string()
        };
        
        let mem = llama_cpp::llama_get_memory(llama.ctx_ptr);
        if !is_pivot {
            llama_cpp::llama_memory_seq_rm(mem, 0, -1, -1);
        }

        let templater = cluaiz_shared::prompting::templater::TemplateManager::default();
        let mut formatted_prompt = if is_pivot {
            // 🛑 ROOT FIX: If we interrupted mid-generation, the model might have been thinking.
            // Appending a new turn without closing </think> corrupts the attention map of 1-bit models.
            // We forcefully close the thought block before starting the new turn.
            format!("\n</think>\n{}", templater.format_turn(dna, &actual_prompt))
        } else {
            templater.format(dna, &actual_prompt)
        };

        let booster = cluaiz_shared::hardware::governor::HardwareGovernor::load_booster_settings().unwrap_or_default();
        let suppress_thinking = booster.think_mode == cluaiz_shared::hardware::schema::booster::FeatureState::Off;
        
        if !suppress_thinking && !formatted_prompt.contains("<think>") {
            formatted_prompt.push_str("<think>\n");
        }
        
        let mut in_think_block = false;
        let mut suppressed_count = 0;

        let vocab = llama_cpp::llama_model_get_vocab(llama.model_ptr);
        let n_vocab = llama_cpp::llama_vocab_n_tokens(vocab);

        if n_vocab <= 0 {
            return Err(anyhow::anyhow!("💀 Invalid model vocabulary"));
        }

        let c_prompt = CString::new(formatted_prompt.clone())?;
        let mut tokens = vec![0i32; formatted_prompt.len() + 8];
        let n_tokens = llama_cpp::llama_tokenize(
            vocab, 
            c_prompt.as_ptr(), 
            formatted_prompt.len() as i32, 
            tokens.as_mut_ptr(), 
            tokens.len() as i32, 
            !is_pivot, 
            true
        );
        
        if n_tokens < 0 {
            return Err(anyhow::anyhow!("Tokenization failed"));
        }
        tokens.truncate(n_tokens as usize);

        let batch_size = (tokens.len() as i32).max(512);
        let mut batch = llama_cpp::llama_batch_init(batch_size, 0, 1);

        let start_pos = if is_pivot {
            llama_cpp::llama_memory_seq_pos_max(llama_cpp::llama_get_memory(llama.ctx_ptr), 0) + 1
        } else {
            0
        };

        for (i, token) in tokens.iter().enumerate() {
            *batch.token.add(i) = *token;
            *batch.pos.add(i) = start_pos + i as i32;
            *batch.n_seq_id.add(i) = 1;
            *(*batch.seq_id.add(i)).add(0) = 0;
            *batch.logits.add(i) = if i == tokens.len() - 1 { 1 } else { 0 };
        }
        batch.n_tokens = tokens.len() as i32;

        if llama_cpp::llama_decode(llama.ctx_ptr, batch) != 0 {
            llama_cpp::llama_batch_free(batch);
            return Err(anyhow::anyhow!("Initial decode failed"));
        }

        let sampler_chain = crate::native::sampler::build_sampler_chain(dna, &tokens)?;

        let is_lookahead = llama.speculative_decoding_mode == 1 || llama.speculative_decoding_mode == 2;
        let mut history: Vec<i32> = tokens.clone();
        let mut lookahead_logs = Vec::new();
        let mut utf8_buffer = Vec::new();

        let mut n_cur = start_pos + tokens.len() as i32;
        let mut n_gen = 0;

        let mut next_token_id = llama_cpp::llama_sampler_sample(sampler_chain, llama.ctx_ptr, -1);
        let mut injected_tokens_queue: std::collections::VecDeque<i32> = std::collections::VecDeque::new();

        while n_gen < max_tokens as i32 {
            if llama.interrupt_signal.load(Ordering::SeqCst) || cluaiz_shared::GLOBAL_CANCEL_SIGNAL.load(Ordering::SeqCst) {
                break;
            }

            // ⚡ Check global UI interrupt signal to skip thinking via pointer (solves FFI state isolation)
            let mut should_skip = false;
            unsafe {
                if !SKIP_PTR.is_null() {
                    should_skip = (*SKIP_PTR).swap(false, Ordering::SeqCst);
                } else {
                    // Fallback to library-local static if pointer not set (though usually they won't match)
                    should_skip = cluaiz_shared::GLOBAL_SKIP_THINKING_SIGNAL.swap(false, Ordering::SeqCst);
                }
            }

            if should_skip {
                let force_str = "\n</think>\n\nAnswer:\n";
                let c_force = CString::new(force_str).unwrap_or_default();
                let mut force_token_arr = [0i32; 64];
                let n_force = llama_cpp::llama_tokenize(
                    vocab, c_force.as_ptr(), force_str.len() as i32,
                    force_token_arr.as_mut_ptr(), force_token_arr.len() as i32,
                    false, false // MUST BE FALSE to prevent failure on unknown pseudo-special tokens
                );
                
                if n_force > 0 {
                    for i in 0..n_force {
                        injected_tokens_queue.push_back(force_token_arr[i as usize]);
                    }
                    eprintln!("🔥 [DEBUG] INJECTED {} TOKENS!", n_force);
                } else {
                    eprintln!("🔥 [DEBUG] TOKENIZE FAILED: {}", n_force);
                    // Fallback: Just push \n (token 198) and answer
                    // (But token IDs differ per model, so we can't hardcode)
                }
            }

            let mut is_injecting = false;
            if let Some(injected_id) = injected_tokens_queue.pop_front() {
                next_token_id = injected_id;
                is_injecting = true;
            }

            crate::native::context::shift_context(
                llama.ctx_ptr,
                &mut n_cur,
                llama.n_ctx,
                tokens.len(),
                llama.context_shifting_mode,
                &mut lookahead_logs
            );

            history.push(next_token_id);
            
            let mut buf = [0u8; 128];
            let n_bytes = llama_cpp::llama_token_to_piece(
                vocab, next_token_id, buf.as_mut_ptr() as *mut c_char, buf.len() as i32, 0, true
            );
            
            if n_bytes > 0 {
                utf8_buffer.extend_from_slice(&buf[..n_bytes as usize]);
                let mut piece = String::new();
                match std::str::from_utf8(&utf8_buffer) {
                    Ok(s) => {
                        piece = s.to_string();
                        utf8_buffer.clear();
                    }
                    Err(e) => {
                        let valid_len = e.valid_up_to();
                        if valid_len > 0 {
                            piece = String::from_utf8_lossy(&utf8_buffer[..valid_len]).to_string();
                            utf8_buffer.drain(..valid_len);
                        }
                        if let Some(error_len) = e.error_len() {
                            utf8_buffer.drain(..error_len);
                        }
                    }
                }
                
                if !piece.is_empty() {
                    if suppress_thinking {
                        for tag in &["<think>", "<thought>", "<|thought_start|>"] {
                            if piece.contains(tag) {
                                in_think_block = true;
                                piece = piece.replace(tag, "");
                            }
                        }
                        for tag in &["</think>", "</thought>", "<|thought_end|>", "<channel|>"] {
                            if piece.contains(tag) {
                                in_think_block = false;
                                piece = piece.replace(tag, "");
                            }
                        }
                        for tag in &["<turn|>", "<|im_end|>", "<end_of_turn>", "<|im_start|>", "<start_of_turn>"] {
                            piece = piece.replace(tag, "");
                        }

                        if !in_think_block && !piece.is_empty() {
                            if !callback(piece) { break; }
                        }
                    } else {
                        for tag in &["<turn|>", "<|im_end|>", "<end_of_turn>", "<|im_start|>", "<start_of_turn>"] {
                            piece = piece.replace(tag, "");
                        }
                        if !piece.is_empty() {
                            // 🚀 MID-GENERATION SKILL INJECTION (PAUSE & PIVOT)
                            if let Ok(router) = cluaiz_shared::skills::router::GLOBAL_SKILL_ROUTER.read() {
                                if let Some(skill_path) = router.check_trigger(&piece) {
                                    eprintln!("\n🔥 [SOVEREIGN OPS] Skill Triggered In-Flight: {:?}", skill_path.file_name().unwrap_or_default());
                                    let md_path = skill_path.join("SKILL.md");
                                    if let Ok(content) = std::fs::read_to_string(&md_path) {
                                        let inject_str = format!("\n[System Memory Injection: {}]\n", content);
                                        if let Ok(c_force) = CString::new(inject_str.clone()) {
                                            let mut force_token_arr = vec![0i32; inject_str.len() + 256];
                                            let n_force = llama_cpp::llama_tokenize(
                                                vocab, c_force.as_ptr(), inject_str.len() as i32,
                                                force_token_arr.as_mut_ptr(), force_token_arr.len() as i32,
                                                false, false
                                            );
                                            if n_force > 0 {
                                                for i in 0..n_force {
                                                    injected_tokens_queue.push_back(force_token_arr[i as usize]);
                                                }
                                                eprintln!("🧠 [SOVEREIGN OPS] Injected {} raw subwords directly into KV-Cache at n_cur={}.", n_force, n_cur);
                                            }
                                        }
                                    }
                                }
                            }

                            if !callback(piece) { break; }
                        }
                    }
                }
            }

            if llama_cpp::llama_vocab_is_eog(vocab, next_token_id) {
                break;
            }

            let mut drafts = crate::native::speculative::generate_drafts(
                &history,
                vocab,
                is_lookahead,
                is_injecting,
                injected_tokens_queue.is_empty(),
                llama.n_ctx,
                n_cur,
                &mut lookahead_logs
            );

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

            let decode_ret = llama_cpp::llama_decode(llama.ctx_ptr, batch);
            if decode_ret != 0 {
                break;
            }

            n_cur += 1;
            if !in_think_block {
                n_gen += 1;
            } else {
                suppressed_count += 1;
                if suppressed_count >= 4096 {
                    break;
                }
            }
            cluaiz_shared::hardware::telemetry::get_pulse().tps_counter.fetch_add(1, Ordering::SeqCst);

            let mut n_match = 0;
            let mut eos_detected = false;
            next_token_id = llama_cpp::llama_sampler_sample(sampler_chain, llama.ctx_ptr, 0);

            for (i, &draft_token) in drafts.iter().enumerate() {
                if next_token_id == draft_token {
                    n_match += 1;
                    history.push(next_token_id);
                    
                    let n_b = llama_cpp::llama_token_to_piece(
                        vocab, next_token_id, buf.as_mut_ptr() as *mut c_char, buf.len() as i32, 0, true
                    );
                    
                    if n_b > 0 {
                        utf8_buffer.extend_from_slice(&buf[..n_b as usize]);
                        let mut piece = String::new();
                        match std::str::from_utf8(&utf8_buffer) {
                            Ok(s) => {
                                piece = s.to_string();
                                utf8_buffer.clear();
                            }
                            Err(e) => {
                                let valid_len = e.valid_up_to();
                                if valid_len > 0 {
                                    piece = String::from_utf8_lossy(&utf8_buffer[..valid_len]).to_string();
                                    utf8_buffer.drain(..valid_len);
                                }
                                if let Some(error_len) = e.error_len() {
                                    utf8_buffer.drain(..error_len);
                                }
                            }
                        }

                        if !piece.is_empty() {
                            if suppress_thinking {
                                for tag in &["<think>", "<thought>", "<|thought_start|>"] {
                                    if piece.contains(tag) {
                                        in_think_block = true;
                                        piece = piece.replace(tag, "");
                                    }
                                }
                                for tag in &["</think>", "</thought>", "<|thought_end|>", "<channel|>"] {
                                    if piece.contains(tag) {
                                        in_think_block = false;
                                        piece = piece.replace(tag, "");
                                    }
                                }
                                for tag in &["<turn|>", "<|im_end|>", "<end_of_turn>", "<|im_start|>", "<start_of_turn>"] {
                                    piece = piece.replace(tag, "");
                                }

                                if !in_think_block && !piece.is_empty() {
                                    // 🚀 MID-GENERATION SKILL INJECTION (PAUSE & PIVOT - Think Block)
                                    if let Ok(router) = cluaiz_shared::skills::router::GLOBAL_SKILL_ROUTER.read() {
                                        if let Some(skill_path) = router.check_trigger(&piece) {
                                            eprintln!("\n🔥 [SOVEREIGN OPS] Skill Triggered In-Flight (Thinking): {:?}", skill_path.file_name().unwrap_or_default());
                                            let md_path = skill_path.join("SKILL.md");
                                            if let Ok(content) = std::fs::read_to_string(&md_path) {
                                                let inject_str = format!("\n[System Memory Injection: {}]\n", content);
                                                if let Ok(c_force) = CString::new(inject_str.clone()) {
                                                    let mut force_token_arr = vec![0i32; inject_str.len() + 256];
                                                    let n_force = llama_cpp::llama_tokenize(
                                                        vocab, c_force.as_ptr(), inject_str.len() as i32,
                                                        force_token_arr.as_mut_ptr(), force_token_arr.len() as i32,
                                                        false, false
                                                    );
                                                    if n_force > 0 {
                                                        for i in 0..n_force {
                                                            injected_tokens_queue.push_back(force_token_arr[i as usize]);
                                                        }
                                                        eprintln!("🧠 [SOVEREIGN OPS] Injected {} raw subwords directly into KV-Cache at n_cur={}.", n_force, n_cur);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    callback(piece);
                                }
                            } else {
                                for tag in &["<turn|>", "<|im_end|>", "<end_of_turn>", "<|im_start|>", "<start_of_turn>"] {
                                    piece = piece.replace(tag, "");
                                }
                                if !piece.is_empty() {
                                    if !callback(piece) { break; }
                                }
                            }
                        }
                    }

                    n_cur += 1;
                    if !in_think_block {
                        n_gen += 1;
                    } else {
                        suppressed_count += 1;
                        if suppressed_count >= 4096 {
                            eos_detected = true;
                            break;
                        }
                    }
                    cluaiz_shared::hardware::telemetry::get_pulse().tps_counter.fetch_add(1, Ordering::SeqCst);

                    if llama_cpp::llama_vocab_is_eog(vocab, next_token_id) {
                        eos_detected = true;
                        break;
                    }

                    next_token_id = llama_cpp::llama_sampler_sample(sampler_chain, llama.ctx_ptr, (i + 1) as i32);
                } else {
                    break;
                }
            }

            let mem = llama_cpp::llama_get_memory(llama.ctx_ptr);
            llama_cpp::llama_memory_seq_rm(mem, 0, n_cur, -1);

            if eos_detected {
                break;
            }
        }

        llama_cpp::llama_sampler_free(sampler_chain);
        llama_cpp::llama_batch_free(batch);
    }

    Ok(())
}
