use crate::{ImpulseScope, ImpulseSynchronizer, NetworkModes};

#[derive(Clone)]
pub struct ImpulseNetworkCapabilities {
    /// level of privilege this instance has
    pub privilege: NetworkModes,
    /// events waiting to be drained and sent to nerves
    synchronizers_queue: Vec<ImpulseSynchronizer>,
}
// Public - Fns
impl ImpulseNetworkCapabilities {
    pub fn has_write_privilege(&self, state_ownership: ImpulseScope) -> bool {
        // if the state is owned by the instance we never need to send it
        match state_ownership {
            ImpulseScope::Instance => return false,
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
    pub fn drain_synchronizers(&mut self) -> Vec<ImpulseSynchronizer> {
        let result = self.synchronizers_queue.clone();
        self.synchronizers_queue.clear();
        result
    }
    pub fn enqueue_synchronizer(&mut self, event: ImpulseSynchronizer) {
        self.synchronizers_queue.push(event);
    }
}
// Static - Fns
impl ImpulseNetworkCapabilities {
    /// Create a new ImpulseNetworkCapabilities storing the passed in NetworkModes as the point of comparison for has_write_privilege
    pub fn new(privilege: NetworkModes) -> ImpulseNetworkCapabilities {
        ImpulseNetworkCapabilities { privilege, synchronizers_queue: Vec::new() }
    }
}
