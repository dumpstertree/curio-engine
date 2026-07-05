use std::any::TypeId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{LazyLock, Mutex};

use crate::RecordCommon;

static NEXT_ID: AtomicI32 = AtomicI32::new(0);

static ID_MAP: LazyLock<Mutex<HashMap<TypeId, i32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct SequentialRecordUIDs;

impl SequentialRecordUIDs {
    /// Returns the sequential ID for type `T`, assigning one on first call.
    pub fn of<T: RecordCommon + 'static>() -> i32 {
        // get id
        let type_id = TypeId::of::<T>();

        // check map, if contains return it
        let mut map = ID_MAP.lock().unwrap();
        if let Some(&id) = map.get(&type_id) {
            return id;
        }

        // if not contained get new id and store it
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        map.insert(type_id, id);

        // return new id
        id
    }
}
