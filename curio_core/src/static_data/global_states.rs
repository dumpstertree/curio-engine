use serde::{de::DeserializeOwned, Serialize};
use std::any::Any;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, RwLock},
};

use crate::{Curio, RecordCommon};

// `CreateFn` returns a boxed concrete type.
// The closure captures `T` so Box<dyn RecordCommon> retains the correct TypeId
// for downcast-rs to recover later.
type CreateFn = fn() -> Box<dyn RecordCommon>;

// `CreateFnArc` returns an arced concrete type.
// Same principle — Arc<T> is upcast to Arc<dyn RecordCommon> inside the closure
// while T is still known, preserving the vtable downcast-rs needs.
type CreateFnArc = fn() -> Arc<dyn RecordCommon>;

type SerializerFn = fn(&dyn Any) -> Vec<u8>;
type DeserializerFn = fn(&[u8]) -> Box<dyn RecordCommon>;

struct StateRegistry {
    creators: HashMap<i32, CreateFn>,
    creators_arc: HashMap<i32, CreateFnArc>,
    serializers: HashMap<i32, SerializerFn>,
    deserializers: HashMap<i32, DeserializerFn>,
}

static REGISTRY: LazyLock<RwLock<StateRegistry>> = LazyLock::new(|| {
    RwLock::new(StateRegistry {
        creators: HashMap::new(),
        creators_arc: HashMap::new(),
        serializers: HashMap::new(),
        deserializers: HashMap::new(),
    })
});

// -------------------------------------------------------------------------
// Getters
// -------------------------------------------------------------------------

pub fn get_global_state_constructor_all() -> Vec<(i32, CreateFn)> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .creators
        .iter()
        .map(|(&id, &f)| (id, f))
        .collect()
}

pub fn get_global_state_constructor_arc_all() -> Vec<(i32, CreateFnArc)> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .creators_arc
        .iter()
        .map(|(&id, &f)| (id, f))
        .collect()
}

pub fn get_global_state_deserialize_all() -> Vec<(i32, DeserializerFn)> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .deserializers
        .iter()
        .map(|(&id, &f)| (id, f))
        .collect()
}

pub fn get_global_state_serialize_all() -> Vec<(i32, SerializerFn)> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .serializers
        .iter()
        .map(|(&id, &f)| (id, f))
        .collect()
}

pub fn get_global_state_constructor(id: &i32) -> Option<CreateFn> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .creators
        .get(id)
        .copied()
}

pub fn get_global_state_deserializer(id: &i32) -> Option<DeserializerFn> {
    let x = REGISTRY
        .read()
        .expect("Registry poisoned")
        .deserializers
        .get(id)
        .copied();

    Curio::log(crate::Severity::Warning, &format!("Failed: {}", id));
    x
}

pub fn get_global_state_serializer(id: &i32) -> Option<SerializerFn> {
    REGISTRY
        .read()
        .expect("Registry poisoned")
        .serializers
        .get(id)
        .copied()
}

// -------------------------------------------------------------------------
// Registration
// -------------------------------------------------------------------------

/// Register a non-serializable state type.
/// Only populates the write-side constructor.
pub fn register_global_state<T>()
where
    T: RecordCommon + Default + 'static,
{
    let mut registry = REGISTRY.write().expect("Registry poisoned");
    // Concrete T is captured here — Box<dyn RecordCommon> retains its TypeId
    registry.creators.insert(T::id(), || Box::new(T::default()));
}

/// Register a serializable state type.
/// Populates both write-side and read-side constructors plus serialization fns.
pub fn register_global_state_serializable<T>()
where
    T: RecordCommon + Serialize + DeserializeOwned + Default + Any + 'static,
{
    let mut registry = REGISTRY.write().expect("Registry poisoned");

    // Write side — Box<T> upcast to Box<dyn RecordCommon> while T is known.
    registry.creators.insert(T::id(), || Box::new(T::default()));

    // Read side — Arc<T> upcast to Arc<dyn RecordCommon> while T is known.
    // downcast-rs will recover Arc<T> from Arc<dyn RecordCommon> via downcast_arc().
    registry
        .creators_arc
        .insert(T::id(), || Arc::new(T::default()) as Arc<dyn RecordCommon>);

    registry.serializers.insert(T::id(), |any| {
        any.downcast_ref::<T>()
            .map(|v| bincode::serialize(v).unwrap())
            .expect("Type mismatch in serialization")
    });

    registry
        .deserializers
        .insert(T::id(), |bytes| Box::new(bincode::deserialize::<T>(bytes).unwrap()));
}
