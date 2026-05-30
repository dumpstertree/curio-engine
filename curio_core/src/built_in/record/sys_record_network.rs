use std::{hash::Hash, sync::OnceLock};

use crate::{
    system::{record_common::RecordOverride, record_id::RecordId},
    FieldState, RecordCommon,
};
static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

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
    fn name(&self) -> String {
        String::from("Network")
    }
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordNetwork>())
    }
}
impl RecordOverride for SysRecordNetwork {
    fn apply(&mut self, field: &str, val: &str) {}
    fn get_state(&self) -> Vec<crate::FieldState> {
        vec![
            FieldState::new("peer_ids", &self.peer_instance_ids()), //
            FieldState::new("host_ids", &self.hosts_instance_ids()),
        ]
    }
}
