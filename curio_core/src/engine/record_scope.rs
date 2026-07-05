use serde::{Deserialize, Serialize};

/// The Curios on the network in which an Record will be transmitted
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum RecordScope {
    Instance,
    Host,
}
