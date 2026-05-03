use dashmap::DashMap;
use once_cell::sync::Lazy;
use archer_shared::hardware::memory::kv_cache::stitching::SovereignSignal;
use uuid::Uuid;

/// 🧠 NeuralSessionCache: The Persistent Memory of the Sovereign OS.
/// Stores KV cache signals indexed by session ID to prevent instruction forgetting.
pub static SESSION_CACHE: Lazy<DashMap<Uuid, SovereignSignal>> = Lazy::new(DashMap::new);

pub struct SessionManager;

impl SessionManager {
    /// 🔗 Stitch Signal: Saves the current neural state for the given session.
    pub fn stitch(session_id: Uuid, signal: SovereignSignal) {
        SESSION_CACHE.insert(session_id, signal);
        tracing::debug!("🧬 [Session] Neural signal stitched for session: {}", session_id);
    }

    /// 🧬 Recall Signal: Retrieves the neural state for the given session.
    pub fn recall(session_id: &Uuid) -> Option<SovereignSignal> {
        SESSION_CACHE.get(session_id).map(|s| s.clone())
    }

    /// 🧹 Purge: Clears memory for a specific session.
    pub fn purge(session_id: &Uuid) {
        SESSION_CACHE.remove(session_id);
    }
}
