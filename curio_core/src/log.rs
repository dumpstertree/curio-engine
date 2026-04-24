pub fn log(severity: Severity, contents: &str) {
    if severity < LOG_LEVEL {
        return;
    }
    match severity {
        Severity::Info => log::info!("{}", contents),
        Severity::Warning => log::warn!("{}", contents),
        Severity::Error => log::error!("{}", contents),
    }
}

static LOG_LEVEL: Severity = Severity::Info;

#[derive(PartialEq, PartialOrd, Eq)]
pub enum Severity {
    Info = 0,
    Warning = 1,
    Error = 2,
}
