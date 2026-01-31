use crate::{collections::version_number::VersionNumber, random::Random};

#[derive(Clone)]
pub struct CurioMetadata {
    pub name: String,
    pub icon: String,
    pub version: VersionNumber,
    pub instance: i32,
}
impl CurioMetadata {
    pub fn new(name: &str, icon: &str, version: VersionNumber) -> CurioMetadata {
        CurioMetadata {
            name: String::from(name),
            icon: String::from(icon),
            version,
            instance: Random::range_int(-9999999, 9999999),
        }
    }
}
impl CurioMetadata {
    pub const fn invalid() -> Self {
        Self {
            name: String::new(),
            icon: String::new(),
            version: VersionNumber::new(0, 0, 0),
            instance: -1,
        }
    }
}
