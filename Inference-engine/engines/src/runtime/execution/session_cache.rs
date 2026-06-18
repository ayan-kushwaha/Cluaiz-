use dashmap::DashMap;
use once_cell::sync::Lazy;
use cluaize_shared::hardware::memory::kv_cache::stitching::CluaizeSignal;
use uuid::Uuid;

/// 🧠 CoreSessionCache: The Persistent Memory of the Cluaize OS.
/// Stores KV cache signals indexed by session ID to prevent instruction forgetting.
pub static SESSION_CACHE: Lazy<DashMap<Uuid, CluaizeSignal>> = Lazy::new(DashMap::new);

pub struct SessionManager;

impl SessionManager {
    /// 🔗 Stitch Signal: Saves the current Core state for the given session.
    pub fn stitch(session_id: Uuid, signal: CluaizeSignal) {
        SESSION_CACHE.insert(session_id, signal);
        tracing::debug!("🧬 [Session] Core signal stitched for session: {}", session_id);
    }

    /// 🧬 Recall Signal: Retrieves the Core state for the given session.
    pub fn recall(session_id: &Uuid) -> Option<CluaizeSignal> {
        SESSION_CACHE.get(session_id).map(|s| s.clone())
    }

    /// 🧹 Purge: Clears memory for a specific session.
    pub fn purge(session_id: &Uuid) {
        SESSION_CACHE.remove(session_id);
    }
}
