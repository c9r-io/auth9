//! Compiled script cache types.

use lru::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Compiled script cache entry
#[derive(Debug, Clone)]
pub(super) struct CompiledScript {
    /// Transpiled JavaScript code
    pub code: String,
}

/// Script cache (action_id -> compiled script)
pub(super) type ScriptCache = Arc<RwLock<LruCache<String, CompiledScript>>>;
