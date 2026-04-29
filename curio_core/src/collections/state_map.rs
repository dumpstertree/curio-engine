use crate::system::system_game_state::RecordCommon;
use std::{borrow::Borrow, collections::HashMap, hash::Hash};

/// Write-side map. Stores owned `Box<dyn RecordCommon>` values that can be
/// mutably accessed and downcast back to their concrete type via `downcast-rs`.
#[derive(Default)]
pub struct StateMap<K: Eq + Hash + Clone> {
    map: HashMap<K, Box<dyn RecordCommon>>,
}

impl<K: Eq + Hash + Clone> Clone for StateMap<K> {
    fn clone(&self) -> Self {
        // RecordCommon values are not Clone by default so we can't clone the map.
        // If you need Clone here, add Clone as a supertrait on RecordCommon.
        panic!("StateMap cannot be cloned unless RecordCommon: Clone");
    }
}

impl<K: Eq + Hash + Clone> StateMap<K> {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    /// Insert a concrete typed value.
    pub fn insert<T: RecordCommon + 'static>(&mut self, key: K, value: T) {
        self.map.insert(key, Box::new(value));
    }

    /// Insert a pre-boxed trait object. Used by the registry and network sync.
    pub fn insert_any(&mut self, key: K, value: Box<dyn RecordCommon>) {
        self.map.insert(key, value);
    }

    /// Returns a shared reference cast to `T`.
    pub fn get<T: RecordCommon + 'static, Q: ?Sized + Eq + Hash>(&self, key: &Q) -> Option<&T>
    where
        K: Borrow<Q>,
    {
        self.map.get(key)?.downcast_ref::<T>()
    }

    /// Returns a mutable reference cast to `T`.
    pub fn get_mut<T: RecordCommon + 'static, Q: ?Sized + Eq + Hash>(&mut self, key: &Q) -> Option<&mut T>
    where
        K: Borrow<Q>,
    {
        self.map.get_mut(key)?.downcast_mut::<T>()
    }

    pub fn contains_key<Q: ?Sized + Eq + Hash>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        self.map.contains_key(key)
    }
}
