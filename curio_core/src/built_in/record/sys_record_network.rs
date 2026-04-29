use std::hash::Hash;

use crate::system::system_game_state::RecordCommon;

#[derive(Default, Hash, PartialEq, Eq, Clone)]
pub struct SysRecordNetwork {
    instance_ids_peers: Vec<i32>,
    instance_ids_hosts: Vec<i32>,
}
impl SysRecordNetwork {
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

impl RecordCommon for SysRecordNetwork {
    fn id() -> i32 {
        109
    }
}
