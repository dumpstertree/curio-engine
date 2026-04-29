use crate::system::system_game_state::RecordCommon;
use std::sync::Arc;
use std::{collections::HashMap, hash::Hash};

/// Read-side map. Stores `Arc<dyn RecordCommon>` values so that `get` returns
/// a cheaply cloned `Arc<T>` — just a refcount increment, no `T` clone.
#[derive(Default, Clone)]
pub struct StateMapRead<K: Eq + Hash + Clone> {
    map: HashMap<K, Arc<dyn RecordCommon>>,
}

impl<K: Eq + Hash + Clone> StateMapRead<K> {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    /// Insert a pre-built `Arc<dyn RecordCommon>`.
    /// Used by the registry in `Ledger::new()` where the concrete type has
    /// already been erased by the constructor fn — but that's fine because
    /// `downcast-rs` tracks the concrete TypeId inside the vtable.
    pub fn insert_raw(&mut self, key: K, value: Arc<dyn RecordCommon>) {
        self.map.insert(key, value);
    }

    /// Insert a concrete `Arc<T>`, upcasting to `Arc<dyn RecordCommon>`.
    /// Used by `Ledger::write()` where `T` is still known.
    pub fn insert_arc<T: RecordCommon + 'static>(&mut self, key: K, value: Arc<T>) {
        self.map.insert(key, value as Arc<dyn RecordCommon>);
    }

    /// Returns a cloned `Arc<T>` — just a refcount increment.
    /// All callers share the same underlying `T` allocation until the next write.
    pub fn get<T: RecordCommon + 'static>(&self, key: &K) -> Option<Arc<T>> {
        // downcast_arc is provided by downcast-rs when the trait uses
        // `DowncastSync` + `impl_downcast!(sync RecordCommon)`
        Arc::clone(self.map.get(key)?).downcast_arc::<T>().ok()
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }
}
