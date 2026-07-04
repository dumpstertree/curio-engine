use crate::NetworkModes;

/// A user within a Curio Network
#[derive(Clone)]
pub struct CurioNetworkParticipant {
    pub guid: i32,
    pub mode: NetworkModes,
}
impl CurioNetworkParticipant {
    pub fn new(guid: i32, mode: NetworkModes) -> CurioNetworkParticipant {
        CurioNetworkParticipant { guid, mode }
    }
}
