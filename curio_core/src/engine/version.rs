use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

/// Representation of a version number that can contain major, minor and patch values
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}
impl Version {
    pub const fn new(major: i32, minor: i32, patch: i32) -> Version {
        Version { major, minor, patch }
    }
}
impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
