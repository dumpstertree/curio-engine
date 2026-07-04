use crate::CurioNetworkParticipant;

/// A Network of Curios that can interact with one another.
#[derive(Clone)]
pub struct CurioNetwork {
    all: Vec<CurioNetworkParticipant>,
    me_index: usize,
}
impl CurioNetwork {
    pub fn new(all: Vec<CurioNetworkParticipant>, me: usize) -> CurioNetwork {
        CurioNetwork { all: all, me_index: me }
    }

    /// Get all CurioNetworkParticipants in this CurioNetwork
    pub fn all(&self) -> &[CurioNetworkParticipant] {
        &self.all
    }

    /// Get my CurioNetworkParticipant data in this CurioNetwork
    pub fn me(&self) -> &CurioNetworkParticipant {
        &self.all[self.me_index]
    }
}
