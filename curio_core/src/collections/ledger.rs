use std::any::type_name;
use std::vec;

use crate::built_in::record::sys_record_screen::SysRecordScreen;
use crate::built_in::record::sys_record_time::SysRecordTime;
use crate::collections::network_capabilities::NetworkCapabilities;
use crate::collections::network_modes::NetworkModes;
use crate::collections::state_map::StateMap;
use crate::collections::state_ownerships::StateOwnerships;
use crate::collections::state_sync_event::StateSyncEvent;
use crate::static_data::global_states::get_global_state_constructor_all;
use crate::system::system_game_state::RecordCommon;
use crate::{log, Severity};

#[derive(Clone)]
pub struct Ledger {
    pub name: String,
    pub instance_id: i32,
    pub all_instance_id: Vec<i32>,
    pub cache: StateMap<i32>,
    pub network_capabilities: Option<NetworkCapabilities>,
}

impl Ledger {
    /// Log but with the prefix for the this gameplay instance
    pub fn log(&self, severity: Severity, contents: &str) {
        log(severity, &format!("[{}]: {}", self.instance_id, contents));
    }
    /// A conveince for get<T> that return time
    pub fn time(&self) -> SysRecordTime {
        self.get::<SysRecordTime>()
    }
    /// A conveince for get<T> that return time
    pub fn screen(&self) -> SysRecordScreen {
        self.get::<SysRecordScreen>()
    }

    /// Apply any SyncEvents produced by other GameStates last frame that need to be applied to overwrite the backing data of this GameState
    pub fn try_apply_network_sync_events(&mut self, sync: &[StateSyncEvent]) {
        // if we dont have networking we can guard
        if self.network_capabilities.is_none() {
            return;
        }

        // for each event we overwrite the value in the cache
        for evnt in sync {
            if let Some(result) = evnt.deserialize() {
                self.cache.insert_any(evnt.id, result);
            } else {
                eprintln!("Deserialize failed for ID {}", evnt.id);
            }
        }
    }

    /// Drainname any sync events that are marked to be sent to another GameState
    pub fn try_drain_network_sync_events(&mut self) -> Vec<StateSyncEvent> {
        // if we have network capabilites
        self.network_capabilities
            // get it mutable
            .as_mut()
            // drains all the sync events from network capabilities
            .map_or_else(Vec::new, |nc| nc.drain_sync_events())
    }

    /// Create a new lightweight instance that works without networking
    pub fn new_single_instance(states: Vec<(i32, Box<dyn RecordCommon>)>) -> Ledger {
        let mut cache = StateMap::new();
        for state in states {
            cache.insert_any(state.0, state.1);
        }

        let gs = Ledger {
            name: String::from(""),
            network_capabilities: None,
            instance_id: -1,
            all_instance_id: vec![-1],
            cache: cache,
        };

        gs.log(Severity::Info, "Created!");

        gs
    }

    /// Create a new instance with network capabilities
    pub fn new(name: &str, network_mode: NetworkModes, instance_id: i32, all_instance_id: Vec<i32>) -> Ledger {
        // create the cache we are going to use
        let mut cache = StateMap::default();

        // add all into cache
        for (id, constructor) in get_global_state_constructor_all() {
            cache.insert_any(id, constructor());
        }

        let gs = Ledger {
            name: String::from(name),
            network_capabilities: Some(NetworkCapabilities::new(network_mode)),
            instance_id: instance_id,
            all_instance_id: all_instance_id,
            cache: cache,
        };

        gs.log(Severity::Info, "Created!");

        gs
    }

    /// Edit the contents of type T
    pub fn edit<TRecord: 'static>(&mut self, edit_fn: impl Fn(&mut TRecord))
    where
        TRecord: RecordCommon + Clone + 'static,
    {
        // pull out values from T
        let state_id = TRecord::id();
        let ownership = TRecord::ownership();

        // get the value from the cache
        let Some(state) = self.cache.get_mut::<TRecord, i32>(&state_id) else {
            panic!("Unknown type {}", type_name::<TRecord>());
        };

        // if we have networking and dont have privilege guard
        if let Some(net) = &self.network_capabilities {
            if !net.has_write_privilege(ownership) {
                eprintln!("No write permission for {}", type_name::<TRecord>());
                return;
            }
        }

        // edit state
        edit_fn(state);

        // if this is only instance wide we can stop now
        if TRecord::ownership() == StateOwnerships::Instance {
            return;
        }

        // check if we have network capabilities
        if let Some(network_capabilites) = &mut self.network_capabilities {
            // get the value we edited from the cache
            if let Some(edited_val) = self.cache.get::<TRecord, i32>(&state_id) {
                // serialize the data into a state sync event
                if let Some(state_sync_event) = StateSyncEvent::serialize(edited_val) {
                    // enqueue the sync event
                    network_capabilites.enqueue_sync_events(state_sync_event);
                }
            }
        };
    }

    /// Get a read only copy of type T
    pub fn get<TRecord: 'static>(&self) -> TRecord
    where
        TRecord: RecordCommon + Clone + 'static,
    {
        self.cache
            // get the cached value
            .get::<TRecord, i32>(&TRecord::id())
            // unwrap else panic
            .unwrap_or_else(|| panic!("Unknown type {}", type_name::<TRecord>()))
            // return clones result
            .clone()
    }
}
