use crate::{collections::any_map::AnyMap, AssetCommon};
use std::{collections::HashMap, sync::Arc, time::Instant, usize};

pub struct AssetCache2 {}
impl AssetCache2 {}
pub struct AssetCache {
    cache: AnyMap<i16>,
    access: HashMap<i16, Instant>,
    max_cache_len: usize,
}
impl AssetCache {
    pub fn try_get_asset<T: AssetCommon>(&mut self, id: &i16) -> Option<Arc<T>> {
        if let Some(edited_val) = self.cache.get::<Arc<T>, i16>(id) {
            // update access to now
            self.access.insert(*id, Instant::now());
            // return clone of arc value
            return Some(edited_val.clone());
        }
        // none found
        None
    }
    pub fn try_set_asset<T: AssetCommon>(&mut self, id: &i16, asset: Arc<T>) {
        // get now to reususe
        let now: Instant = Instant::now();

        // prune old
        while self.access.len() > self.max_cache_len {
            // Find the key with the oldest Instant
            if let Some((&oldest_key, _)) = self.access.iter().min_by_key(|(_, &instant)| instant) {
                // Remove the oldest entry
                self.access.remove(&oldest_key);
            } else {
                break;
            }
        }
        // update access to now
        self.access.insert(*id, now);
        // insert asset
        self.cache.insert(*id, asset);
    }
    pub fn new(max_cache_len: usize) -> AssetCache {
        AssetCache {
            cache: AnyMap::default(),
            access: HashMap::new(),
            max_cache_len,
        }
    }
}
