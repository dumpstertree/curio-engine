use std::hash::Hash;
use std::{any::Any, borrow::Borrow, collections::HashMap};
pub struct GameState {
    cache: AnyMap<i32>,
}
impl GameState {
    pub fn new() -> GameState {
        GameState {
            cache: AnyMap::<i32>::default(),
        }
    }
    pub fn get_value<T: 'static>(&self, key: i32) -> Result<&T, GetError> {
        self.cache.get_result::<T, i32>(&key)
    }
    pub fn has_value(&self, key: i32) -> bool {
        self.cache.0.contains_key(&key)
    }
    pub fn add<T: Any>(&mut self, key: i32, val: T) {
        self.cache.insert::<T>(key, val);
    }
}

#[derive(Default)]
pub struct AnyMap<K>(HashMap<K, Box<dyn Any>>);

#[derive(Debug)]
pub enum GetError {
    EmptyKey,
    MismatchedType,
}

impl<K: Hash + Eq> AnyMap<K> {
    fn insert<T: Any>(&mut self, key: K, value: T) {
        self.0.insert(key, Box::new(value));
    }

    pub fn get<T: Any, Q: ?Sized + Hash + Eq>(&self, key: &Q) -> Option<&T>
    where
        K: Borrow<Q>,
    {
        self.0.get(key)?.downcast_ref()
    }

    pub fn get_result<T: Any, Q: ?Sized + Hash + Eq>(&self, key: &Q) -> Result<&T, GetError>
    where
        K: Borrow<Q>,
    {
        self.0.get(key).ok_or(GetError::EmptyKey)?.downcast_ref().ok_or(GetError::MismatchedType)
    }
}
