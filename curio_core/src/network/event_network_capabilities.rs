use crate::{EventScope, EventSyncEvent, NetworkModes};

#[derive(Clone)]
pub struct EventNetworkCapabilities {
    /// level of privilege this instance has
    pub privilege: NetworkModes,

    /// events waiting to be drained and sent to other game states
    pub ouput_sync_events: Vec<EventSyncEvent>,
}
// Public - Fns
impl EventNetworkCapabilities {
    pub fn has_write_privilege(&self, state_ownership: EventScope) -> bool {
        // if the state is owned by the instance we never need to send it
        match state_ownership {
            EventScope::Instance => return false,
            _ => {}
        }
        // // if the state is owned by the host we right it if we have host privilege
        // match self.privilege {
        //     NetworkModes::LocalHost => return true,
        //     NetworkModes::OnlineHost => return true,
        //     NetworkModes::LocalPeer => return false,
        //     NetworkModes::OnlinePeer => return false,
        // }
        true
    }
    pub fn drain_sync_events(&mut self) -> Vec<EventSyncEvent> {
        let result = self.ouput_sync_events.clone();
        self.ouput_sync_events.clear();
        result
    }
    pub fn enqueue_sync_events(&mut self, event: EventSyncEvent) {
        self.ouput_sync_events.push(event);
    }
}
// Static - Fns
impl EventNetworkCapabilities {}
impl EventNetworkCapabilities {
    pub fn new(privilige: NetworkModes) -> EventNetworkCapabilities {
        EventNetworkCapabilities { privilege: privilige, ouput_sync_events: Vec::new() }
    }
}
