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
