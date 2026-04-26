use kernel::CureKernel;
use storage::EmbeddedManager;

/// Shared application state containing the kernel and storage manager.
pub struct AppState {
    pub kernel: CureKernel,
    pub embedded: EmbeddedManager,
}
