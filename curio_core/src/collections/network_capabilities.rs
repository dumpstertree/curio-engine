use crate::collections::network_modes::NetworkModes;
use crate::collections::state_ownerships::StateOwnerships;
use crate::collections::state_sync_event::StateSyncEvent;

#[derive(Clone)]
pub struct NetworkCapabilities {
    /// level of privilege this instance has
    pub privilege: NetworkModes,

    /// events waiting to be drained and sent to other game states
    pub ouput_sync_events: Vec<StateSyncEvent>,
}
// Public - Fns
impl NetworkCapabilities {
    pub fn has_write_privilege(&self, state_ownership: StateOwnerships) -> bool {
        // if the state is owned by the instance we always write it
        match state_ownership {
            StateOwnerships::Instance => return true,
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
    pub fn drain_sync_events(&mut self) -> Vec<StateSyncEvent> {
        let result = self.ouput_sync_events.clone();
        self.ouput_sync_events.clear();
        result
    }
    pub fn enqueue_sync_events(&mut self, event: StateSyncEvent) {
        self.ouput_sync_events.push(event);
    }
}
// Static - Fns
impl NetworkCapabilities {}
impl NetworkCapabilities {
    pub fn new(privilige: NetworkModes) -> NetworkCapabilities {
        NetworkCapabilities { privilege: privilige, ouput_sync_events: Vec::new() }
    }
}
