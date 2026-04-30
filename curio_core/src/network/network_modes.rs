use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum NetworkModes {
    LocalHost = 4,
    OnlineHost = 3,
    LocalPeer = 2,
    OnlinePeer = 1,
}
impl NetworkModes {
    pub fn all() -> Vec<NetworkModes> {
        vec![NetworkModes::OnlinePeer, NetworkModes::LocalPeer, NetworkModes::OnlineHost, NetworkModes::LocalHost]
    }
    pub fn all_peer() -> Vec<NetworkModes> {
        vec![NetworkModes::OnlinePeer, NetworkModes::LocalPeer]
    }
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
