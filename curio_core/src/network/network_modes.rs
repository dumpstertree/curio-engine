use serde::Serialize;
use std::fmt::{Display, Formatter, Result};

/// An object the describes the privlige of a an object on a CurioNetwork
#[derive(Default, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Serialize)]
pub enum NetworkModes {
    LocalHost = 4,
    OnlineHost = 3,
    #[default]
    LocalPeer = 2,
    OnlinePeer = 1,
}
impl NetworkModes {
    // Convience for all values
    pub fn all() -> Vec<NetworkModes> {
        vec![NetworkModes::OnlinePeer, NetworkModes::LocalPeer, NetworkModes::OnlineHost, NetworkModes::LocalHost]
    }
    // Convience for all peer values
    pub fn all_peer() -> Vec<NetworkModes> {
        vec![NetworkModes::OnlinePeer, NetworkModes::LocalPeer]
    }
    // Convience for all host values
    pub fn all_host() -> Vec<NetworkModes> {
        vec![NetworkModes::OnlineHost, NetworkModes::LocalHost]
    }
}
impl Display for NetworkModes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            NetworkModes::LocalHost => f.write_str("local host"),
            NetworkModes::OnlineHost => f.write_str("online host"),
            NetworkModes::LocalPeer => f.write_str("local peer"),
            NetworkModes::OnlinePeer => f.write_str("online peer"),
        }
    }
}
