use core::system::system_game_state::IState;
use std::hash::Hash;

use macro_state::global_state;

#[derive(Hash, Eq)]
#[global_state]
pub struct StateNetwork {
    instance_ids_peers: Vec<i32>,
    instance_ids_hosts: Vec<i32>,
}
impl StateNetwork {
    pub fn peer_instance_ids(&self) -> &[i32] {
        &self.instance_ids_peers.as_slice()
    }
    pub fn hosts_instance_ids(&self) -> &[i32] {
        &self.instance_ids_peers.as_slice()
    }
    pub fn set_peer_instance_ids(&mut self, ids: Vec<i32>) {
        self.instance_ids_peers = ids;
    }
    pub fn set_hosts_instance_ids(&mut self, ids: Vec<i32>) {
        self.instance_ids_hosts = ids;
    }
}

impl IState for StateNetwork {
    fn id() -> i32 {
        345434
    }
}
