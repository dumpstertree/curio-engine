use crate::system::system_game_state::IState;
use std::{collections::HashMap, fmt::Debug, hash::Hash};

// ------------------------------------------------------
// Struct: StateMap
// ------------------------------------------------------

#[derive(Default)]
pub struct StateMap<K>
where
    K: Eq + Hash + Clone + Debug,
{
    map: HashMap<K, Box<dyn IState>>,
}

// ------------------------------------------------------
// Implementation
// ------------------------------------------------------
impl<K> Clone for StateMap<K>
where
    K: Eq + Hash + Clone + Debug,
{
    fn clone(&self) -> Self {
        let mut cloned = HashMap::new();
        for (k, v) in &self.map {
            cloned.insert(k.clone(), v.clone_box());
        }
        Self { map: cloned }
    }
}

impl<K> StateMap<K>
where
    K: Eq + Hash + Clone + Debug,
{
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    /// Insert a new IState into the map.
    pub fn insert<T: IState + Default + Clone + 'static>(&mut self, key: K, value: T) {
        self.map.insert(key, Box::new(value));
    }
    pub fn insert_any(&mut self, key: K, value: Box<dyn IState>) {
        self.map.insert(key, value);
    }

    /// Get a reference to a stored type.
    pub fn get<T: IState + 'static, Q: ?Sized + Eq + Hash>(&self, key: &Q) -> Option<&T>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.map.get(key)?.as_ref().as_any()?.downcast_ref()
    }

    /// Get a mutable reference to a stored type.
    pub fn get_mut<T: IState + 'static, Q: ?Sized + Eq + Hash>(&mut self, key: &Q) -> Option<&mut T>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.map.get_mut(key)?.as_mut_any()?.downcast_mut::<T>()
    }

    /// Check if the map contains a given key.
    pub fn contains_key<Q: ?Sized + Eq + Hash>(&self, key: &Q) -> bool
    where
        K: std::borrow::Borrow<Q>,
    {
        self.map.contains_key(key)
    }

    /// Remove a key and return the boxed IState.
    pub fn remove<Q: ?Sized + Eq + Hash>(&mut self, key: &Q) -> Option<Box<dyn IState>>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.map.remove(key)
    }

    /// Clears all entries.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Iterate over keys and their dyn values.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &Box<dyn IState>)> {
        self.map.iter()
    }

    /// Mutable iterator.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut Box<dyn IState>)> {
        self.map.iter_mut()
    }
}

// ------------------------------------------------------
// Helper trait for downcasting
// ------------------------------------------------------
pub trait AsAny {
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
    fn as_mut_any(&mut self) -> Option<&mut dyn std::any::Any> {
        None
    }
}

impl<T: IState + 'static> AsAny for T {
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
    fn as_mut_any(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}
