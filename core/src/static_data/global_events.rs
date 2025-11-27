use crate::collections::event_queue::IGameEvent;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    any::Any,
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

struct StateRegistry {
    creators: HashMap<i32, CreateFn>,
    serializers: HashMap<i32, SerializerFn>,
    deserializers: HashMap<i32, DeserializerFn>,
}

static REGISTRY: LazyLock<RwLock<StateRegistry>> = LazyLock::new(|| {
    RwLock::new(StateRegistry {
        creators: HashMap::new(),
        serializers: HashMap::new(),
        deserializers: HashMap::new(),
    })
});

type CreateFn = fn() -> Box<dyn IGameEvent>;
type SerializerFn = fn(&dyn Any) -> Vec<u8>;
type DeserializerFn = fn(&[u8]) -> Box<dyn Any>;

/// Get all Global State ConstructorFns paired with thier ID
pub fn get_global_event_constructor_all() -> Vec<(i32, CreateFn)> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .creators
        .iter()
        .map(|(&id, &f)| (id, f))
        .collect()
}
/// Get all Global State DeserializeFns paired with thier ID
pub fn get_global_event_deserialize_all() -> Vec<(i32, DeserializerFn)> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .deserializers
        .iter()
        .map(|(&id, &f)| (id, f))
        .collect()
}
/// Get all Global State SerializeFns paired with thier ID
pub fn get_global_event_serialize_all() -> Vec<(i32, SerializerFn)> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .serializers
        .iter()
        .map(|(&id, &f)| (id, f))
        .collect()
}
/// Get Global State ConstructorFn for id. Returns None if fn is not registered for that id
pub fn get_global_event_constructor(id: &i32) -> Option<CreateFn> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .creators
        .get(id)
        .copied()
}
/// Get Global State DeserializerFn for id. Returns None if fn is not registered for that id
pub fn get_global_event_deserializer(id: &i32) -> Option<DeserializerFn> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .deserializers
        .get(id)
        .copied()
}
/// Get Global State SerializerFn for id. Returns None if fn is not registered for that id
pub fn get_global_event_serializer(id: &i32) -> Option<SerializerFn> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .serializers
        .get(id)
        .copied()
}
/// Register a State  to be added to the Global State Registry that conforms to Serialize and DeserializeOwned in order to be transmisable between GameStates
pub fn register_global_events<T>()
where
    T: IGameEvent + Serialize + DeserializeOwned + Default + Any + 'static,
{
    let mut registry = REGISTRY.write().expect("Registry poisoned");

    let id = T::id();
    registry.creators.insert(id, || Box::new(T::default()));

    registry.serializers.insert(id, |any| {
        any.downcast_ref::<T>()
            .map(|concrete| bincode::serialize(concrete).unwrap())
            .expect("Type mismatch in serialization")
    });

    registry.deserializers.insert(id, |bytes| {
        let obj: T = bincode::deserialize(bytes).unwrap();
        Box::new(obj)
    });
}
