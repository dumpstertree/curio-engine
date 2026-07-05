use egui::mutex::Mutex;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, LazyLock, RwLock},
};

use crate::ImpulseCommon;

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

type CreateFn = fn() -> Box<dyn ImpulseCommon>;
type SerializerFn = fn(&dyn Any) -> Vec<u8>;
type DeserializerFn = fn(&[u8]) -> Box<dyn Any>;

// /// Get all Global State ConstructorFns paired with thier ID
// pub fn get_global_event_constructor_all() -> Vec<(i32, CreateFn)> {
//     REGISTRY
//         .read()
//         .expect("Registry poisoned")
//         .creators
//         .iter()
//         .map(|(&id, &f)| (id, f))
//         .collect()
// }
// /// Get all Global State DeserializeFns paired with thier ID
// pub fn get_global_event_deserialize_all() -> Vec<(i32, DeserializerFn)> {
//     REGISTRY
//         .read()
//         .expect("Registry poisoned")
//         .deserializers
//         .iter()
//         .map(|(&id, &f)| (id, f))
//         .collect()
// }
// /// Get all Global State SerializeFns paired with thier ID
// pub fn get_global_event_serialize_all() -> Vec<(i32, SerializerFn)> {
//     REGISTRY
//         .read()
//         .expect("Registry poisoned")
//         .serializers
//         .iter()
//         .map(|(&id, &f)| (id, f))
//         .collect()
// }
// /// Get Global State ConstructorFn for id. Returns None if fn is not registered for that id
// pub fn get_global_event_constructor(id: &i32) -> Option<CreateFn> {
//     REGISTRY
//         .read()
//         .expect("Registry poisoned")
//         .creators
//         .get(id)
//         .copied()
// }
// /// Get Global State DeserializerFn for id. Returns None if fn is not registered for that id
// pub fn get_global_event_deserializer(id: &i32) -> Option<DeserializerFn> {
//     REGISTRY
//         .read()
//         .expect("Registry poisoned")
//         .deserializers
//         .get(id)
//         .copied()
// }
// /// Get Global State SerializerFn for id. Returns None if fn is not registered for that id
// pub fn get_global_event_serializer(id: &i32) -> Option<SerializerFn> {
//     REGISTRY
//         .read()
//         .expect("Registry poisoned")
//         .serializers
//         .get(id)
//         .copied()
// }
// /// Register a State  to be added to the Global State Registry that conforms to Serialize and DeserializeOwned in order to be transmisable between GameStates
// pub fn register_global_events<T>()
// where
//     T: ImpulseCommon + Serialize + DeserializeOwned + Default + Any + 'static,
// {
//     let mut registry = REGISTRY.write().expect("Registry poisoned");

//     let id = T::id();
//     registry.creators.insert(id, || Box::new(T::default()));

//     registry.serializers.insert(id, |any| {
//         any.downcast_ref::<T>()
//             .map(|concrete| bincode::serialize(concrete).unwrap())
//             .expect("Type mismatch in serialization")
//     });

//     registry.deserializers.insert(id, |bytes| {
//         let obj: T = bincode::deserialize(bytes).unwrap();
//         Box::new(obj)
//     });
// }

pub struct GlobalImpulses {}
impl GlobalImpulses {
    pub fn register<T>()
    where
        T: ImpulseCommon + Serialize + DeserializeOwned + Default + Any + 'static,
    {
        // how to create a simple default
        let construct_fn: CreateFn = || Box::new(T::default());

        // how to serialize
        let serialize_fn: SerializerFn = |any| {
            any.downcast_ref::<T>()
                .map(|concrete| bincode::serialize(concrete).unwrap())
                .expect("Type mismatch in serialization")
        };

        // how to deserialize
        let deserialize_fn: DeserializerFn = |bytes| {
            let obj: T = bincode::deserialize(bytes).unwrap();
            Box::new(obj)
        };

        // create a registration for this type
        let registration = ImpulseRegistration { construct_fn, serialize_fn, deserialize_fn };

        // insert into the global registry
        REGISTRY2.lock().insert(T::id(), Arc::new(registration));
    }
    pub fn try_get_registration(uid: &i32) -> Option<Arc<ImpulseRegistration>> {
        REGISTRY2.lock().get(uid).cloned()
    }
    pub fn get_all_registrations() -> Vec<(i32, Arc<ImpulseRegistration>)> {
        REGISTRY2
            .lock()
            .iter()
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect::<Vec<(i32, Arc<ImpulseRegistration>)>>()
    }
}

static REGISTRY2: LazyLock<Mutex<HashMap<i32, Arc<ImpulseRegistration>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct ImpulseRegistration {
    pub construct_fn: CreateFn,
    pub serialize_fn: SerializerFn,
    pub deserialize_fn: DeserializerFn,
}
