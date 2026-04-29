use std::any::TypeId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{LazyLock, Mutex};

static NEXT_ID: AtomicI32 = AtomicI32::new(0);

static ID_MAP: LazyLock<Mutex<HashMap<TypeId, i32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct RecordId;

impl RecordId {
    /// Returns the sequential ID for type `T`, assigning one on first call.
    /// The mutex is only hit once per type — after that the macro-generated
    /// OnceLock in each type's `id()` impl makes this unreachable.
    pub fn of<T: 'static>() -> i32 {
        let type_id = TypeId::of::<T>();
        let mut map = ID_MAP.lock().unwrap();
        if let Some(&id) = map.get(&type_id) {
            return id;
        }
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        map.insert(type_id, id);
        id
    }
}
