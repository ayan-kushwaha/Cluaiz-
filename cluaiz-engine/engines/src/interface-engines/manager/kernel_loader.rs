use std::path::PathBuf;


/// Reads `cluaiz_root` securely via the Cluaiz Hardware Governor.
/// This uses the binary truth (`system_control.bin`) as the ultimate source,
/// exactly as the Cluaiz Architecture intends. Zero custom hardcoding.
fn read_cluaiz_root() -> Option<PathBuf> {
    match archer_shared::HardwareGovernor::load_system_control() {
        Ok(control) => Some(PathBuf::from(control.context.cluaiz_root)),
        Err(e) => {
            tracing::error!("❌ [KernelLoader] Failed to read System Truth: {}", e);
            None
        }
    }
}

/// Kernel Loader
/// Manages pre-compiled binaries (.dll, .so, .dylib) for different OS/Architecture pairs.
/// All paths are resolved dynamically via system_control.json. Zero hardcoding.
pub struct KernelLoader {
    base_dir: PathBuf,
}

impl KernelLoader {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Checks if a kernel binary exists locally for a target OS.
    pub fn exists_for_os(&self, kernel_name: &str, os: &str) -> bool {
        let path = self.resolve_path_for_os(kernel_name, os);
        path.exists()
    }

    /// Checks if a kernel binary exists locally for the current OS.
    pub fn exists(&self, kernel_name: &str) -> bool {
        let path = self.resolve_path(kernel_name);
        path.exists()
    }

    /// Resolves path based on current compilation target (NATIVE).
    pub fn resolve_path(&self, kernel_name: &str) -> PathBuf {
        let os = if cfg!(target_os = "windows") { "Windows" }
            else if cfg!(target_os = "linux") { "Linux" }
            else if cfg!(target_os = "android") { "Android" }
            else if cfg!(target_os = "macos") { "macOS" }
            else if cfg!(target_os = "ios") { "iOS" }
            else { "Unknown" };
        self.resolve_path_for_os(kernel_name, os)
    }

    /// Resolves the absolute path for a kernel binary for a SPECIFIC OS.
    /// Priority: [cluaiz_root]/interface-engines/ → fallback to base_dir/target/release/
    pub fn resolve_path_for_os(&self, kernel_name: &str, os: &str) -> PathBuf {
        let ext = match os {
            "Windows" => "dll",
            "Linux" | "Android" => "so",
            "macOS" | "iOS" => "dylib",
            _ => "bin",
        };

        // Build the canonical file name (matches CI/CD build output)
        let file_name = format!("archer_{}.{}", kernel_name, ext);

        // 1. PRIMARY: Read cluaiz_root from system_control.json — the Single Source of Truth.
        //    Path pattern: <cluaiz_root>/interface-engines/<file_name>
        if let Some(cluaiz_root) = read_cluaiz_root() {
            let Cluaiz_path = cluaiz_root.join("interface-engines").join(&file_name);
            if Cluaiz_path.exists() {
                tracing::info!("🎯 [KernelLoader] Cluaiz path resolved: {:?}", Cluaiz_path);
                return Cluaiz_path;
            }
        }

        // 2. FALLBACK: Local development build output (for dev/testing only).
        let mut dev_path = self.base_dir.clone();
        dev_path.push("target");
        dev_path.push("release");
        dev_path.push(&file_name);
        tracing::warn!("⚠️ [KernelLoader] Cluaiz path not found. Falling back to dev path: {:?}", dev_path);
        dev_path
    }
}

