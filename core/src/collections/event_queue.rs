use crate::collections::game_state::AnyMap;
use egui::util::id_type_map::TypeId;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub struct EventQueue {
    cache: AnyMap<i32>,
}
impl EventQueue {
    pub fn new() -> EventQueue {
        EventQueue {
            cache: AnyMap::<i32>::default(),
        }
    }

    fn type_id_to_i32<T: 'static>() -> i32 {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<T>().hash(&mut hasher);
        (hasher.finish() & 0xFFFF_FFFF) as i32 // Safe truncation
    }

    pub fn enqueue_event<T: 'static>(&mut self, val: T)
    where
        T: Clone,
    {
        let id = EventQueue::type_id_to_i32::<T>();
        if let Some(vec) = self.cache.get_mut::<Vec<T>, i32>(&id) {
            vec.push(val);
        } else {
            self.cache.insert::<Vec<T>>(id, vec![val]);
        }
    }
    pub fn get_queued_events<T: 'static>(&self) -> &[T]
    where
        T: Clone,
    {
        let id = EventQueue::type_id_to_i32::<T>();
        if let Some(x) = self.cache.get::<Vec<T>, i32>(&id) {
            x.as_slice()
        } else {
            &[]
        }
    }
    pub fn clear_queued_events<T: 'static>(&mut self) {
        let id = EventQueue::type_id_to_i32::<T>();
        if let Some(x) = self.cache.get_mut::<Vec<T>, i32>(&id) {
            x.clear();
        }
    }
}
