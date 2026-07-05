use crate::{
    static_data::impulse_registration::{CustructorFn, DeserializerFn, SerializerFn},
    ImpulseCommon, ImpulseRegistration,
};
use egui::mutex::Mutex;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, LazyLock},
};

static GLOBAL_REGISTRY: LazyLock<Mutex<HashMap<i32, Arc<ImpulseRegistration>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// A singleton object used to store all the Impulse types that can be used throughout the engine.
pub struct GlobalImpulses {}
impl GlobalImpulses {
    /// Register TImpulse to the GlobalRecords. This version will include any serialization data.
    pub fn register<TImpulse>()
    where
        TImpulse: ImpulseCommon + Serialize + DeserializeOwned + Default + Any + 'static,
    {
        // how to create a simple default
        let construct_fn: CustructorFn = || Box::new(TImpulse::default());

        // how to serialize
        let serialize_fn: SerializerFn = |any| {
            any.downcast_ref::<TImpulse>()
                .map(|concrete| bincode::serialize(concrete).unwrap())
                .expect("Type mismatch in serialization")
        };

        // how to deserialize
        let deserialize_fn: DeserializerFn = |bytes| {
            let obj: TImpulse = bincode::deserialize(bytes).unwrap();
            Box::new(obj)
        };

        // create a registration for this type
        let registration = ImpulseRegistration { construct_fn, serialize_fn, deserialize_fn };

        // insert into the global registry
        GLOBAL_REGISTRY
            .lock()
            .insert(TImpulse::id(), Arc::new(registration));
    }

    /// Try and get an ImpulseRegistration for the UID. Will return None if not present
    pub fn try_get_registration(uid: &i32) -> Option<Arc<ImpulseRegistration>> {
        GLOBAL_REGISTRY.lock().get(uid).cloned()
    }
    /// Returns all ImpulseRegistrations and thier UIDs
    pub fn get_all_registrations() -> Vec<(i32, Arc<ImpulseRegistration>)> {
        GLOBAL_REGISTRY
            .lock()
            .iter()
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect::<Vec<(i32, Arc<ImpulseRegistration>)>>()
    }
}
