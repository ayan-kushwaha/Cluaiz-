pub mod engine;
pub use engine::OnnxEngine;

pub mod text;
pub mod audio;
pub mod vision;

// ─── Sovereign FFI Gateway for ONNX ─────────────────────────────────────────

#[no_mangle]
pub extern "C" fn cluaize_kernel_init() -> *const std::os::raw::c_char {
    // ONNX Environment Init
    let _ = ort::init()
        .with_name("cluaize_onnx_env")
        .commit();
    
    tracing::info!("🧿 [ONNX.cpp-Kernel] Sovereign Handshake & Backend Initialized.");
    "cluaize-onnx.cpp-active\0".as_ptr() as *const std::os::raw::c_char
}

#[no_mangle]
pub extern "C" fn cluaize_kernel_instantiate(
    path_ptr: *const std::os::raw::c_char,
    _booster_ptr: *const cluaize_shared::hardware::schema::booster::CluaizeBoosterContext,
) -> *mut OnnxEngine {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let path_str = unsafe { std::ffi::CStr::from_ptr(path_ptr) }
            .to_string_lossy()
            .into_owned();
        
        let mut engine = Box::new(OnnxEngine::new().unwrap());
        
        if path_str != "default" && !path_str.is_empty() {
            // Check if vision or text based on path or logic. For simplicity, if path contains "clip" or "vision", load vision.
            if path_str.to_lowercase().contains("clip") || path_str.to_lowercase().contains("vision") {
                if let Err(e) = engine.load_vision_model(&path_str) {
                    tracing::error!("❌ [ONNX-Lib] Vision Model Load Failed: {}", e);
                    return std::ptr::null_mut();
                }
            } else {
                // For text, assume tokenizer is in the same dir
                let path = std::path::Path::new(&path_str);
                let dir = path.parent().unwrap_or(path);
                let tokenizer_path = dir.join("tokenizer.json").to_string_lossy().into_owned();
                
                if let Err(e) = engine.load_text_model(&path_str, &tokenizer_path) {
                    tracing::error!("❌ [ONNX-Lib] Text Model Load Failed: {}", e);
                    return std::ptr::null_mut();
                }
            }
        }

        tracing::info!("✅ [ONNX-Lib] Engine Instantiated successfully.");
        Box::into_raw(engine)
    }));

    match result {
        Ok(ptr) => ptr,
        Err(_) => {
            tracing::error!("🚨 [FFI-Panic] Caught panic in cluaize_kernel_instantiate (ONNX)!");
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn cluaize_kernel_generate_embedding(
    engine_ptr: *mut OnnxEngine,
    text_ptr: *const std::os::raw::c_char,
    out_ptr: *mut f32,
    max_len: usize,
    out_len: *mut usize,
) -> i32 {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if engine_ptr.is_null() || text_ptr.is_null() || out_ptr.is_null() || out_len.is_null() { return -1; }
        
        let engine = unsafe { &*engine_ptr };
        let text = unsafe { std::ffi::CStr::from_ptr(text_ptr) }.to_string_lossy();
        
        use neural_core::interfaces::router_contract::EmbeddingDriver;
        match engine.gen_embedding(&text) {
            Ok(vec) => {
                let len = std::cmp::min(vec.len(), max_len);
                unsafe {
                    std::ptr::copy_nonoverlapping(vec.as_ptr(), out_ptr, len);
                    *out_len = len;
                }
                0
            }
            Err(e) => {
                tracing::error!("❌ [ONNX-Lib] Embedding generation failed: {}", e);
                -2
            }
        }
    }));

    match result {
        Ok(res) => res,
        Err(_) => -3,
    }
}

#[no_mangle]
pub extern "C" fn cluaize_kernel_free(engine_ptr: *mut OnnxEngine) {
    if !engine_ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(engine_ptr);
        }
    }
}
