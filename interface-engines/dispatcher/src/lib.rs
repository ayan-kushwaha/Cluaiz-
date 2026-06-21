use anyhow::{Result, anyhow};
use cluaize_shared::backend::signature::{KernelSignature, GlobalFeatureRegistry, BackendType};
use system_booster::BoosterControl;

use tokio::sync::mpsc;

pub enum EngineResponse {
    TokenStream(mpsc::Receiver<String>),
    FinalResult(String),
    Error(String),
}

/// 🚦 NeuralDispatcher (The Master Router)
/// The core router that owns hardware logic and dispatches prompts across Native IPC and HTTP.
pub struct NeuralDispatcher {
    pub booster_state: BoosterControl,
    pub current_signature: KernelSignature,
    // Future additions: TensorTransducer, NeuralFoundry instances
}

impl NeuralDispatcher {
    pub fn new(booster_state: BoosterControl, signature: KernelSignature) -> Self {
        Self {
            booster_state,
            current_signature: signature,
        }
    }

    /// Primary entry point for real-time token streaming.
    /// Used by both the FFI Named Pipes (Native Desktop) and HTTP SSE (External).
    pub async fn dispatch_stream(&self, prompt: &str, skip_brain: bool) -> EngineResponse {
        // 🚀 Real-time Silicon Probe
        let hardware = cluaize_shared::hardware::HardwareOrchestrator::probe().silicon_truth;
        let backend = GlobalFeatureRegistry::select_runtime(&self.current_signature, &hardware);
        
        tracing::info!("🚦 [Master Router] Routing prompt to backend: {:?}", backend);

        let (tx, rx) = mpsc::channel::<String>(100);
        let prompt_clone = prompt.to_string();

        match backend {
            BackendType::RuntimeB | BackendType::RuntimeC | BackendType::RuntimeA => {
                tokio::task::spawn_blocking(move || {
                    let target_os = std::env::consts::OS;
                    let ext = match target_os {
                        "windows" => "dll",
                        "macos" => "dylib",
                        _ => "so",
                    };
                    let prefix = if target_os == "windows" { "" } else { "lib" };
                    let binary_name = format!("{}cluaize_llama.{}", prefix, ext);
                    
                    let mut binary_path = cluaize_shared::HardwareGovernor::resolve_interface_path()
                        .join("kernels")
                        .join(&binary_name);
                        
                    if !binary_path.exists() {
                        let cargo_name = binary_name.replace("-", "_");
                        binary_path = std::path::PathBuf::from(format!("target/release/{}", cargo_name));
                        if !binary_path.exists() {
                            binary_path = std::path::PathBuf::from(format!("target/debug/{}", cargo_name));
                        }
                    }

                    tracing::info!("🔗 [Dispatcher] Attempting native FFI to {:?}", binary_path);

                    unsafe {
                        #[cfg(windows)]
                        let lib: libloading::Library = {
                            let flags = 0x00000008; 
                            libloading::os::windows::Library::load_with_flags(&binary_path, flags).expect("Llama DLL missing").into()
                        };

                        #[cfg(not(windows))]
                        let lib = libloading::Library::new(&binary_path).expect("Llama SO/Dylib missing");

                        let instantiate_fn: libloading::Symbol<unsafe extern "C" fn(*const std::os::raw::c_char, *const std::ffi::c_void) -> *mut std::ffi::c_void> = 
                            lib.get(b"cluaize_kernel_instantiate").expect("cluaize_kernel_instantiate missing");
                        
                        let model_path = "default".to_string();
                        
                        let c_path = std::ffi::CString::new(model_path).unwrap();
                        let engine_ptr = instantiate_fn(c_path.as_ptr() as *const std::os::raw::c_char, std::ptr::null());
                        
                        if !engine_ptr.is_null() {
                            let gen_stream_fn: libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const std::os::raw::c_char, usize, extern "C" fn(*const std::os::raw::c_char, *mut std::ffi::c_void) -> bool, *mut std::ffi::c_void) -> i32> = 
                                lib.get(b"cluaize_kernel_generate_stream").expect("cluaize_kernel_generate_stream missing");

                            let c_prompt = std::ffi::CString::new(prompt_clone).unwrap();

                            extern "C" fn callback(token_ptr: *const std::os::raw::c_char, user_data: *mut std::ffi::c_void) -> bool {
                                let tx = unsafe { &*(user_data as *const tokio::sync::mpsc::Sender<String>) };
                                let token = unsafe { std::ffi::CStr::from_ptr(token_ptr) }.to_string_lossy().into_owned();
                                tx.blocking_send(token).is_ok()
                            }

                            let tx_ptr = &tx as *const _ as *mut std::ffi::c_void;
                            // 4096 default context for now, dynamic based on booster in the kernel
                            gen_stream_fn(
                                engine_ptr,
                                c_prompt.as_ptr(),
                                4096,
                                callback,
                                tx_ptr,
                            );

                            let free_fn: libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void)> = lib.get(b"cluaize_kernel_free").expect("cluaize_kernel_free missing");
                            free_fn(engine_ptr);
                        }
                    }

                    let _ = tx.blocking_send("\n[DONE]\n".to_string());
                });
                EngineResponse::TokenStream(rx)
            }
            _ => {
                EngineResponse::Error(format!("Unsupported backend architecture: {:?}", backend))
            }
        }
    }

    /// Legacy blocking call, to be deprecated once all clients shift to `dispatch_stream`.
    pub async fn dispatch_prompt(&self, prompt: &str) -> Result<String> {
        let mut stream = match self.dispatch_stream(prompt, false).await {
            EngineResponse::TokenStream(rx) => rx,
            EngineResponse::Error(e) => return Err(anyhow::anyhow!(e)),
            EngineResponse::FinalResult(r) => return Ok(r),
        };
        
        let mut final_text = String::new();
        while let Some(token) = stream.recv().await {
            if token.trim() == "[DONE]" { break; }
            final_text.push_str(&token);
        }
        Ok(final_text)
    }
}

/// 🚥 EmbeddingDispatcher
/// Routes embedding requests to ONNX dynamically via libloading.
pub struct EmbeddingDispatcher {
    active_lib: std::sync::Arc<libloading::Library>,
    engine_ptr: *mut std::ffi::c_void,
}

unsafe impl Send for EmbeddingDispatcher {}
unsafe impl Sync for EmbeddingDispatcher {}

impl EmbeddingDispatcher {
    pub fn new() -> Result<Self> {
        let target_os = std::env::consts::OS;
        let ext = match target_os {
            "windows" => "dll",
            "macos" => "dylib",
            _ => "so",
        };
        let prefix = if target_os == "windows" { "" } else { "lib" };
        let binary_name = format!("{}cluaize-onnx.{}", prefix, ext);
        
        // Use persistence or fallback to target/debug
        let mut binary_path = cluaize_shared::HardwareGovernor::resolve_interface_path()
            .join("kernels")
            .join(&binary_name);
            
        if !binary_path.exists() {
            // Fallback to cargo target directory (cargo outputs with underscores)
            let cargo_name = binary_name.replace("-", "_");
            binary_path = std::path::PathBuf::from(format!("target/release/{}", cargo_name));
            if !binary_path.exists() {
                binary_path = std::path::PathBuf::from(format!("target/debug/{}", cargo_name));
            }
        }

        unsafe {
            #[cfg(windows)]
            let lib: libloading::Library = {
                // LOAD_WITH_ALTERED_SEARCH_PATH (0x00000008) forces Windows to search for dependent DLLs
                // (like onnxruntime_providers_cuda.dll) in the same directory as the kernel DLL being loaded.
                let flags = 0x00000008; 
                let win_lib = libloading::os::windows::Library::load_with_flags(&binary_path, flags)
                    .map_err(|e| anyhow::anyhow!("ONNX Binary Mapping Failed on path {:?}: {}. OS Error: {:?}", binary_path, e, std::io::Error::last_os_error()))?;
                win_lib.into()
            };

            #[cfg(not(windows))]
            let lib = libloading::Library::new(&binary_path)
                .map_err(|e| anyhow::anyhow!("ONNX Binary Mapping Failed on path {:?}: {}. OS Error: {:?}", binary_path, e, std::io::Error::last_os_error()))?;

            
            let init: libloading::Symbol<unsafe extern "C" fn() -> *const std::os::raw::c_char> = lib.get(b"cluaize_kernel_init")
                .map_err(|_| anyhow::anyhow!("Invalid ONNX Kernel: 'cluaize_kernel_init' missing"))?;
            init();

            let instantiate_fn: libloading::Symbol<unsafe extern "C" fn(*const std::os::raw::c_char, *const std::ffi::c_void) -> *mut std::ffi::c_void> = 
                lib.get(b"cluaize_kernel_instantiate")
                .map_err(|_| anyhow::anyhow!("Invalid ONNX Kernel: 'cluaize_kernel_instantiate' missing"))?;
            
            let c_path = std::ffi::CString::new("default")?;
            let engine_ptr = instantiate_fn(c_path.as_ptr() as *const std::os::raw::c_char, std::ptr::null());
            
            if engine_ptr.is_null() {
                return Err(anyhow::anyhow!("ONNX Kernel Instantiation Failed"));
            }

            tracing::info!("✅ [Dispatcher] ONNX Kernel Dynamically Linked.");
            Ok(Self {
                active_lib: std::sync::Arc::new(lib),
                engine_ptr,
            })
        }
    }

    pub fn dispatch_embedding(&self, text: &str) -> Result<Vec<f32>> {
        use neural_core::interfaces::router_contract::EmbeddingDriver;
        tracing::info!("🚥 [Dispatcher] Routing embedding request dynamically to ONNX FFI...");
        self.gen_embedding(text).map_err(|e| anyhow::anyhow!("Embedding Error: {:?}", e))
    }

    pub fn dispatch_multimodal(&self, bytes: &[u8], modality: neural_core::interfaces::router_contract::Modality) -> Result<Vec<f32>> {
        use neural_core::interfaces::router_contract::EmbeddingDriver;
        self.gen_multimodal_embedding(bytes, modality).map_err(|e| anyhow::anyhow!("Multimodal Error: {:?}", e))
    }
}

impl neural_core::interfaces::router_contract::EmbeddingDriver for EmbeddingDispatcher {
    fn gen_embedding(&self, text: &str) -> Result<Vec<f32>, neural_core::interfaces::router_contract::EngineError> {
        unsafe {
            let gen_emb_fn: libloading::Symbol<unsafe extern "C" fn(*mut std::ffi::c_void, *const std::os::raw::c_char, *mut f32, usize, *mut usize) -> i32> = 
                match self.active_lib.get(b"cluaize_kernel_generate_embedding") {
                    Ok(f) => f,
                    Err(_) => return Err(neural_core::interfaces::router_contract::EngineError::EmbeddingFailed("Symbol missing".to_string()))
                };
            
            let c_prompt = std::ffi::CString::new(text).map_err(|_| neural_core::interfaces::router_contract::EngineError::EmbeddingFailed("CString conversion failed".to_string()))?;
            let max_dims = 8192;
            let mut out_buffer = vec![0.0f32; max_dims];
            let mut out_len: usize = 0;

            let status = gen_emb_fn(
                self.engine_ptr, 
                c_prompt.as_ptr() as *const std::os::raw::c_char, 
                out_buffer.as_mut_ptr(),
                max_dims,
                &mut out_len as *mut usize
            );
            
            if status != 0 {
                return Err(neural_core::interfaces::router_contract::EngineError::EmbeddingFailed(format!("Code: {}", status)));
            }
            
            out_buffer.truncate(out_len);
            Ok(out_buffer)
        }
    }

    fn gen_multimodal_embedding(&self, _bytes: &[u8], _modality: neural_core::interfaces::router_contract::Modality) -> Result<Vec<f32>, neural_core::interfaces::router_contract::EngineError> {
        Err(neural_core::interfaces::router_contract::EngineError::UnsupportedModality("Multimodal FFI not implemented yet".to_string()))
    }
}

impl Drop for EmbeddingDispatcher {
    fn drop(&mut self) {
        if !self.engine_ptr.is_null() {
            unsafe {
                if let Ok(free_fn) = self.active_lib.get::<unsafe extern "C" fn(*mut std::ffi::c_void)>(b"cluaize_kernel_free") {
                    free_fn(self.engine_ptr);
                }
            }
        }
    }
}
