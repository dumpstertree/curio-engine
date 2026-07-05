use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Object used to track when remote data was last updated and by who
#[derive(Serialize, Deserialize, Clone)]
pub struct CacheMetadata {
    pub edited_by_instance: Vec<i32>,
    pub last_edit: SystemTime,
}
