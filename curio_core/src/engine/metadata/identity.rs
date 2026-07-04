use crate::{Random, Version};

/// Identity represents the specifics of a Curio instance
#[derive(Clone)]
pub struct Identity {
    pub name: String,
    pub icon: String,
    pub version: Version,
    pub instance: i32,
}
impl Identity {
    pub fn new(name: &str, icon: &str, version: Version) -> Identity {
        Identity {
            name: String::from(name),
            icon: String::from(icon),
            version,
            instance: Random::guid(6),
        }
    }
}
impl Identity {
    pub const fn invalid() -> Self {
        Self {
            name: String::new(),
            icon: String::new(),
            version: Version::new(0, 0, 0),
            instance: -1,
        }
    }
}
