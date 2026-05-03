use std::path::PathBuf;

/// Kernel Loader
/// Manages pre-compiled binaries (.dll, .so, .dylib) for different OS/Architecture pairs.
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

    /// Resolves path based on current compilation target (NATIVE)
    pub fn resolve_path(&self, kernel_name: &str) -> PathBuf {
        #[cfg(target_os = "windows")]
        return self.resolve_path_for_os(kernel_name, "Windows");
        
        #[cfg(target_os = "linux")]
        return self.resolve_path_for_os(kernel_name, "Linux");

        #[cfg(target_os = "android")]
        return self.resolve_path_for_os(kernel_name, "Android");
        
        #[cfg(target_os = "macos")]
        return self.resolve_path_for_os(kernel_name, "macOS");

        #[cfg(target_os = "ios")]
        return self.resolve_path_for_os(kernel_name, "iOS");

        self.resolve_path_for_os(kernel_name, "Unknown")
    }

    /// Resolves the absolute path for a kernel binary for a SPECIFIC OS.
    /// Pattern: [cluaiz]/interface-engines/[engine]/[engine].[ext]
    pub fn resolve_path_for_os(&self, kernel_name: &str, os: &str) -> PathBuf {
        let ext = match os {
            "Windows" => "dll",
            "Linux" | "Android" => "so",
            "macOS" | "iOS" => "dylib",
            _ => "bin",
        };
        
        // We use archer_ prefix for the actual file name since that's what the build outputs
        let file_name = format!("archer_{}.{}", kernel_name, ext);

        // 1. Check Global Sovereign Blueprint path first
        #[cfg(target_os = "windows")]
        let global_dir = PathBuf::from("C:\\Cluaiz\\drivers");
        #[cfg(not(target_os = "windows"))]
        let global_dir = PathBuf::from("/Cluaiz/drivers");
        
        let global_path = global_dir.join(&file_name);
        if global_path.exists() {
            return global_path;
        }

        // 2. Fallback to local development path
        let mut local_path = self.base_dir.clone();
        local_path.push("target");
        local_path.push("release");
        local_path.push(&file_name);
        
        local_path
    }
}
