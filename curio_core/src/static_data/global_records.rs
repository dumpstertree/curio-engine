use crate::static_data::record_registration::{ConstructFn, DeserializerFn, SerializerFn};
use crate::{RecordCommon, RecordRegistration};
use egui::mutex::Mutex;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

static GLOBAL_REGISTRY: LazyLock<Mutex<HashMap<i32, Arc<RecordRegistration>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// A singleton object used to store all the Record types that can be used throughout the engine.
pub struct GlobalRecords {}
impl GlobalRecords {
    /// Register TRecord to the GlobalRecords. This version will not include any serialization data.
    pub fn register<TRecord>()
    where
        TRecord: RecordCommon + Default + 'static,
    {
        // how to create a simple default
        let construct_fn: ConstructFn = || Box::new(TRecord::default());

        // create a registration for this type
        let registration = RecordRegistration { construct_fn, serialize_fn: None, deserialize_fn: None };

        // insert into the global registry
        GLOBAL_REGISTRY
            .lock()
            .insert(TRecord::id(), Arc::new(registration));
    }
    /// Register TRecord to the GlobalRecords. This version will include any serialization data.
    pub fn register_serializable<T>()
    where
        T: RecordCommon + Default + Serialize + DeserializeOwned + 'static,
    {
        // how to create a simple default
        let construct_fn: ConstructFn = || Box::new(T::default());

        // how to serialize
        let serialize_fn: Option<SerializerFn> = Some(|any| {
            any.downcast_ref::<T>()
                .map(|v| bincode::serialize(v).unwrap())
                .expect("Type mismatch in serialization")
        });

        // how to deserialize
        let deserialize_fn: Option<DeserializerFn> = Some(|bytes| Box::new(bincode::deserialize::<T>(bytes).unwrap()));

        // create a registration for this type
        let registration = RecordRegistration { construct_fn, serialize_fn, deserialize_fn };

        // insert into the global registry
        GLOBAL_REGISTRY
            .lock()
            .insert(T::id(), Arc::new(registration));
    }

    /// Try and get an RecordRegistrations for the UID. Will return None if not present
    pub fn try_get_registration(uid: &i32) -> Option<Arc<RecordRegistration>> {
        GLOBAL_REGISTRY.lock().get(uid).cloned()
    }
    /// Returns all RecordRegistrations and their UIDs
    pub fn get_all_registrations() -> Vec<(i32, Arc<RecordRegistration>)> {
        GLOBAL_REGISTRY
            .lock()
            .iter()
            .map(|x| (x.0.clone(), x.1.clone()))
            .collect::<Vec<(i32, Arc<RecordRegistration>)>>()
    }
}
