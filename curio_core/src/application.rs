use crate::{log, Severity};

pub struct Application {}
impl Application {
    pub fn log(severity: Severity, contents: &str) {
        log(0, severity, &format!("[SYS]: {}", contents));
    }
}
