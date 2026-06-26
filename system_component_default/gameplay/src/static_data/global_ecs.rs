use crate::traits::habit::Habit;
use std::sync::{LazyLock, RwLock};

/// Function that creates a boxed untyped value (what register stores)
type ReceiverCreateFn = fn() -> Box<dyn Habit>;

struct ReceiverRegistry {
    constructors: Vec<ReceiverCreateFn>,
}

static RECEIVER_REGISTRY: LazyLock<RwLock<ReceiverRegistry>> = LazyLock::new(|| RwLock::new(ReceiverRegistry { constructors: Vec::new() }));

/// Register a receiver type `R` for event type `T`.
/// We store a constructor that returns `Box<R>` but erased to `Box<dyn Any>`.
pub fn register_global_ecs<T>()
where
    T: Habit + Default + 'static,
{
    // Curio::log(Severity::Info, &format!("Registered Global Record: {}", type_name::<T>()));
    let mut reg = RECEIVER_REGISTRY.write().expect("Registry poisoned");

    reg.constructors.push(|| Box::new(T::default()));
}
pub fn get_global_ecs_instances() -> Vec<Box<dyn Habit>>
where {
    let reg = RECEIVER_REGISTRY.read().expect("Registry poisoned");

    reg.constructors.iter().map(|creator| creator()).collect()
}
