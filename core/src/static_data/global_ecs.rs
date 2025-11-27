use std::{
    any::Any,
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use crate::gameplay::ecs::traits::{ecs_event_reciever::EventReciever, ecs_system::ECSSystemEventless};

/// Function that creates a boxed untyped value (what register stores)
type ReceiverCreateFn = fn() -> Box<dyn ECSSystemEventless>;

struct ReceiverRegistry {
    constructors: Vec<ReceiverCreateFn>,
}

static RECEIVER_REGISTRY: LazyLock<RwLock<ReceiverRegistry>> = LazyLock::new(|| RwLock::new(ReceiverRegistry { constructors: Vec::new() }));

/// Register a receiver type `R` for event type `T`.
/// We store a constructor that returns `Box<R>` but erased to `Box<dyn Any>`.
pub fn register_global_ecs<T>()
where
    T: ECSSystemEventless + Default,
{
    let mut reg = RECEIVER_REGISTRY.write().expect("Registry poisoned");

    reg.constructors.push(|| Box::new(T::default()));
}
pub fn get_global_ecs_instances() -> Vec<Box<dyn ECSSystemEventless>>
where {
    let reg = RECEIVER_REGISTRY.read().expect("Registry poisoned");

    reg.constructors.iter().map(|creator| creator()).collect()
}
