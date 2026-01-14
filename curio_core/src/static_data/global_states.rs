use std::{
    any::Any,
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use crate::system::system_game_state::IState;
use serde::{de::DeserializeOwned, Serialize};

type CreateFn = fn() -> Box<dyn IState>;
type SerializerFn = fn(&dyn Any) -> Vec<u8>;
type DeserializerFn = fn(&[u8]) -> Box<dyn IState>;

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

/// Get all Global State ConstructorFns paired with thier ID
pub fn get_global_state_constructor_all() -> Vec<(i32, CreateFn)> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .creators
        .iter()
        .map(|(&id, &f)| (id, f))
        .collect()
}
/// Get all Global State DeserializeFns paired with thier ID
pub fn get_global_state_deserialize_all() -> Vec<(i32, DeserializerFn)> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .deserializers
        .iter()
        .map(|(&id, &f)| (id, f))
        .collect()
}
/// Get all Global State SerializeFns paired with thier ID
pub fn get_global_state_serialize_all() -> Vec<(i32, SerializerFn)> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .serializers
        .iter()
        .map(|(&id, &f)| (id, f))
        .collect()
}
/// Get Global State ConstructorFn for id. Returns None if fn is not registered for that id
pub fn get_global_state_constructor(id: &i32) -> Option<CreateFn> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .creators
        .get(id)
        .copied()
}
/// Get Global State DeserializerFn for id. Returns None if fn is not registered for that id
pub fn get_global_state_deserializer(id: &i32) -> Option<DeserializerFn> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .deserializers
        .get(id)
        .copied()
}
/// Get Global State SerializerFn for id. Returns None if fn is not registered for that id
pub fn get_global_state_serializer(id: &i32) -> Option<SerializerFn> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .serializers
        .get(id)
        .copied()
}
/// Register a State to be added to the Global State Registry
pub fn register_global_state<T>()
where
    T: Any + IState + Default + 'static,
{
    let mut registry = REGISTRY.write().expect("Registry poisoned");
    registry.creators.insert(T::id(), || Box::new(T::default()));
}
/// Register a State  to be added to the Global State Registry that conforms to Serialize and DeserializeOwned in order to be transmisable between GameStates
pub fn register_global_state_serializable<T>()
where
    T: IState + Serialize + DeserializeOwned + Default + Any + 'static,
{
    let mut registry = REGISTRY.write().expect("Registry poisoned");

    registry.creators.insert(T::id(), || Box::new(T::default()));

    registry.serializers.insert(T::id(), |any| {
        any.downcast_ref::<T>()
            .map(|concrete| bincode::serialize(concrete).unwrap())
            .expect("Type mismatch in serialization")
    });

    registry.deserializers.insert(T::id(), |bytes| {
        let obj: T = bincode::deserialize(bytes).unwrap();
        Box::new(obj)
    });
}
