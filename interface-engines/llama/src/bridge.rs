//! bridge.rs: The Neural Soul Linker (Internalized).
//! This is an internal driver for archer-llama to handle specialized 1-bit (Bonsai) tensors.

use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_void};
use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};

type CreateBackendFn = unsafe extern "C" fn(path: *const c_char) -> *mut c_void;
type GenerateFn = unsafe extern "C" fn(backend: *mut c_void, prompt: *const c_char, max_tokens: usize) -> *mut c_char;
type FreeStringFn = unsafe extern "C" fn(s: *mut c_char);
type DestroyBackendFn = unsafe extern "C" fn(backend: *mut c_void);

pub struct InternalPrismBridge {
    _library: Library,
    backend_ptr: *mut c_void,
    fn_generate: GenerateFn,
    fn_free_string: FreeStringFn,
    fn_destroy_backend: DestroyBackendFn,
}

impl InternalPrismBridge {
    pub fn load_specialized(model_path: &str) -> std::result::Result<Self, String> {
        // We look for prism-compatible binaries inside the engine's internal bin path
        let lib_path = if cfg!(windows) {
            "engines/llama/bin/archer_prism.dll"
        } else {
            "engines/llama/bin/libarcher_prism.so"
        };

        if !Path::new(lib_path).exists() {
            return Err(format!("❌ Prism-Inference Kernel not found at {}. Specialized 1-bit models require the Prism core.", lib_path));
        }

        unsafe {
            let lib = Library::new(lib_path).map_err(|e| format!("Failed to load Prism library: {}", e))?;
            
            let create_fn: Symbol<CreateBackendFn> = lib.get(b"create_backend\0")
                .map_err(|_| "Missing symbol: create_backend")?;
            
            let fn_generate: Symbol<GenerateFn> = lib.get(b"backend_generate\0")
                .map_err(|_| "Missing symbol: backend_generate")?;
                
            let fn_free_string: Symbol<FreeStringFn> = lib.get(b"free_string\0")
                .map_err(|_| "Missing symbol: free_string")?;
                
            let fn_destroy_backend: Symbol<DestroyBackendFn> = lib.get(b"destroy_backend\0")
                .map_err(|_| "Missing symbol: destroy_backend")?;

            let c_path = CString::new(model_path).map_err(|_| "Invalid model path")?;
            let backend_ptr = create_fn(c_path.as_ptr());

            if backend_ptr.is_null() {
                return Err("Failed to initialize Prism backend instance".to_string());
            }

            Ok(Self {
                fn_generate: *fn_generate,
                fn_free_string: *fn_free_string,
                fn_destroy_backend: *fn_destroy_backend,
                _library: lib,
                backend_ptr,
            })
        }
    }

    pub fn generate(&mut self, prompt: &str, max_tokens: usize) -> std::result::Result<String, String> {
        let c_prompt = CString::new(prompt).map_err(|_| "Invalid prompt string")?;
        unsafe {
            let res_ptr = (self.fn_generate)(self.backend_ptr, c_prompt.as_ptr(), max_tokens);
            if res_ptr.is_null() { return Err("Generation failed".into()); }
            let c_res = CStr::from_ptr(res_ptr);
            let response = c_res.to_string_lossy().into_owned();
            (self.fn_free_string)(res_ptr);
            Ok(response)
        }
    }
}

impl Drop for InternalPrismBridge {
    fn drop(&mut self) {
        unsafe { (self.fn_destroy_backend)(self.backend_ptr); }
    }
}
