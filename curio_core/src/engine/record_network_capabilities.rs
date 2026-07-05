use crate::{NetworkModes, RecordScope, RecordSynchronizer};

#[derive(Clone)]
pub struct RecordNetworkCapabilities {
    /// level of privilege this instance has
    pub privilege: NetworkModes,
    /// events waiting to be drained and sent to ledgers
    synchronizers_queue: Vec<RecordSynchronizer>,
}
// Public - Fns
impl RecordNetworkCapabilities {
    /// Compare interal privilege with the passed in ownership to see if it should be added to the queue  
    pub fn has_write_privilege(&self, state_ownership: RecordScope) -> bool {
        // if the state is owned by the instance we always write it
        match state_ownership {
            RecordScope::Instance => return true,
            _ => {}
        }
        // if the state is owned by the host we right it if we have host privilege
        match self.privilege {
            NetworkModes::LocalHost => return true,
            NetworkModes::OnlineHost => return true,
            NetworkModes::LocalPeer => return false,
            NetworkModes::OnlinePeer => return false,
        }
    }
    pub fn drain_synchronizers(&mut self) -> Vec<RecordSynchronizer> {
        let result = self.synchronizers_queue.clone();
        self.synchronizers_queue.clear();
        result
    }
    pub fn enqueue_synchronizer(&mut self, event: RecordSynchronizer) {
        self.synchronizers_queue.push(event);
    }
}
// Static - Fns
impl RecordNetworkCapabilities {
    /// Create a new RecordNetworkCapabilities storing the passed in NetworkModes as the point of comparison for has_write_privilege
    pub fn new(privilege: NetworkModes) -> RecordNetworkCapabilities {
        RecordNetworkCapabilities { privilege, synchronizers_queue: Vec::new() }
    }
}
