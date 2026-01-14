use std::{any::Any, borrow::Borrow, collections::HashMap, hash::Hash};

#[derive(Default)]
pub struct AnyMap<K>(HashMap<K, Box<dyn Any>>);

impl<K: Hash + Eq> AnyMap<K> {
    pub fn insert<T: Any>(&mut self, key: K, value: T) {
        self.0.insert(key, Box::new(value));
    }
    pub fn insert_any(&mut self, key: K, value: Box<dyn Any>) {
        self.0.insert(key, value);
    }

    pub fn get<T: Any, Q: ?Sized + Hash + Eq>(&self, key: &Q) -> Option<&T>
    where
        K: Borrow<Q>,
    {
        self.0.get(key)?.downcast_ref()
    }
    pub fn get_mut<T: Any, Q: ?Sized + Hash + Eq>(&mut self, key: &Q) -> Option<&mut T>
    where
        K: Borrow<Q>,
    {
        self.0.get_mut(key)?.downcast_mut()
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

#[derive(Debug)]
pub enum GetError {
    EmptyKey,
    MismatchedType,
}
