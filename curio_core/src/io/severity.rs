/// The severity of a log. Info < Warning < Error
#[derive(PartialEq, PartialOrd, Eq, Clone)]
pub enum Severity {
    Info = 0,
    Warning = 1,
    Error = 2,
}
