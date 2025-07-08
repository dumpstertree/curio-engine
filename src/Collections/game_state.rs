use std::hash::Hash;
use std::{any::Any, borrow::Borrow, collections::HashMap};

use crate::system::system_game_state::IState;

pub struct GameState {
    cache: AnyMap<i32>,
}
impl GameState {
    pub fn new() -> GameState {
        GameState {
            cache: AnyMap::<i32>::default(),
        }
    }

    pub fn set_value2<T: 'static>(&mut self, val: T)
    where
        T: IState<T>,
        T: Clone,
    {
        let id = T::id();
        self.cache.insert::<T>(id, val);
    }
    pub fn get_value2<T: 'static>(&self) -> T
    where
        T: IState<T>,
        T: Clone,
    {
        let id = T::id();
        let Some(val) = self.cache.get::<T, i32>(&id) else { return T::default() };

        return val.clone();
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
        self.0
            .get(key)
            .ok_or(GetError::EmptyKey)?
            .downcast_ref()
            .ok_or(GetError::MismatchedType)
    }
}
