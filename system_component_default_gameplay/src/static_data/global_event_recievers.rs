use core::collections::event_queue::IGameEvent;
use std::{
    any::Any,
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use crate::traits::impulse::Impulse;

/// Function that creates a boxed untyped value (what register stores)
type ReceiverCreateFn = fn() -> Box<dyn Any>;

struct ReceiverRegistry {
    constructors: HashMap<i32, Vec<ReceiverCreateFn>>,
}

static RECEIVER_REGISTRY: LazyLock<RwLock<ReceiverRegistry>> = LazyLock::new(|| RwLock::new(ReceiverRegistry { constructors: HashMap::new() }));

/// Register a receiver type `R` for event type `T`.
/// We store a constructor that returns `Box<R>` but erased to `Box<dyn Any>`.
pub fn register_global_event_receiver<T, R>()
where
    T: IGameEvent + Clone + 'static,
    R: Impulse<T> + Default + Any + 'static,
{
    let mut reg = RECEIVER_REGISTRY.write().expect("Registry poisoned");

    let id = T::id();

    reg.constructors
        .entry(id)
        .or_insert_with(Vec::new)
        .push(|| {
            // create concrete receiver R, upcast to Box<dyn EventReciever<T>>, then erase to Any
            let boxed_receiver: Box<dyn Impulse<T>> = Box::new(R::default());
            Box::new(boxed_receiver) as Box<dyn Any>
        });
}
pub fn get_global_event_receivers<T>() -> Vec<Box<dyn Impulse<T>>>
where
    T: IGameEvent + Clone + 'static,
{
    let reg = RECEIVER_REGISTRY.read().expect("Registry poisoned");
    let id = T::id();

    if let Some(creators) = reg.constructors.get(&id) {
        creators
            .iter()
            .map(|creator| {
                let boxed_any = creator(); // Box<dyn Any> containing Box<dyn EventReciever<T>>
                let boxed_receiver: Box<dyn Impulse<T>> = *boxed_any
                    .downcast::<Box<dyn Impulse<T>>>()
                    .expect("Type downcast failed for EventReciever<T>");
                boxed_receiver
            })
            .collect()
    } else {
        Vec::new()
    }
}
