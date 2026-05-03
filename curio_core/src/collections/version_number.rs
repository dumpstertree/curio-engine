use std::fmt::{Display, Formatter, Result};

#[derive(Clone)]
pub struct VersionNumber {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}
impl VersionNumber {
    pub const fn new(major: i32, minor: i32, patch: i32) -> VersionNumber {
        VersionNumber { major, minor, patch }
    }
}
impl Display for VersionNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
