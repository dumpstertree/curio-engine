use std::any::type_name;
use std::rc::Rc;

use crate::built_in::record::sys_record_screen::SysRecordScreen;
use crate::built_in::record::sys_record_time::SysRecordTime;
use crate::static_data::global_states::get_global_state_constructor_all;
use crate::{log, ComponentState, Curio, CurioNetwork, LedgerEntry, ObjectState, PluginState, RecordCommon, RecordNetworkCapabilities, RecordScope, RecordSynchronizer, Severity};

#[derive(Clone)]
pub struct Ledger {
    entries: Vec<Option<LedgerEntry>>,
    pub network_capabilities: RecordNetworkCapabilities,
    pub network: CurioNetwork,
}

// impl Fns -> Convience
impl Ledger {
    pub fn time(&self) -> Rc<SysRecordTime> {
        self.read::<SysRecordTime>()
    }
    pub fn screen(&self) -> Rc<SysRecordScreen> {
        self.read::<SysRecordScreen>()
    }
}
// Ledger -> impl Private Fns
impl Ledger {
    #[inline]
    fn get_entry(&self, id: i32, type_str: &str) -> &LedgerEntry {
        self.entries
            .get(id as usize)
            .and_then(|slot| slot.as_ref())
            .unwrap_or_else(|| panic!("[{}] type '{}' is not registered (id {})", self.network.me().guid, type_str, id))
    }

    #[inline]
    fn get_entry_mut(&mut self, id: i32, instance_id: i32, type_str: &str) -> &mut LedgerEntry {
        self.entries
            .get_mut(id as usize)
            .and_then(|slot| slot.as_mut())
            .unwrap_or_else(|| panic!("[{}] type '{}' is not registered (id {})", instance_id, type_str, id))
    }
}

impl Ledger {
    /// Serialize to a readable copy
    pub fn serializable(&self) -> (String, PluginState) {
        let mut objs = vec![];
        for e in &self.entries {
            if let Some(e) = e {
                let obj = ObjectState {
                    object_name: e.write.name(),
                    children: vec![],
                    components: vec![ComponentState {
                        component_name: e.write.name(),
                        fields: e.write.get_state(),
                    }],
                };
                objs.push(obj);
            }
        }
        return (format!("{}-{}", self.network.me().mode, self.network.me().guid.to_string()), PluginState { tab_name: "Ledger".to_string(), objects: objs });
    }

    /// Instance populated from the GlobalRecord registry
    pub fn new(network: CurioNetwork) -> Self {
        // get all constructors
        let constructors = get_global_state_constructor_all();

        // get the max count of the constructors
        let max_id = constructors.iter().map(|(id, _)| *id).max().unwrap_or(-1);

        // populate a vec of all the entries we need
        let mut entries: Vec<Option<LedgerEntry>> = (0..=max_id.max(0)).map(|_| None).collect();

        // populate indicies of entries vec
        for (id, constructor) in constructors {
            entries[id as usize] = Some(LedgerEntry::new(constructor()));
        }

        // create and return the ledger
        Ledger {
            entries,
            network_capabilities: RecordNetworkCapabilities::new(network.me().mode),
            network,
        }
    }
    /// Instance populated from the custom registry. NetworkModes are always set to LocalHost. Expect this to eventually be deprecated.
    pub fn new_custom(states: Vec<(i32, Box<dyn RecordCommon>)>) -> Self {
        let max_id = states.iter().map(|(id, _)| *id).max().unwrap_or(-1);
        let mut entries: Vec<Option<LedgerEntry>> = (0..=max_id.max(0)).map(|_| None).collect();

        for (id, value) in states {
            entries[id as usize] = Some(LedgerEntry::new(value));
        }

        let ledger = Ledger {
            entries,
            network_capabilities: RecordNetworkCapabilities::new(crate::NetworkModes::LocalHost),
            network: CurioNetwork::new(Vec::new(), 0),
        };

        ledger.log(Severity::Info, "Created (single instance)!");
        ledger
    }

    /// Log adding tracing through this Ledger for debugging
    pub fn log(&self, severity: Severity, contents: &str) {
        log(self.network.me().guid, severity, &format!("[{}~{}]: {}", self.network_capabilities.clone().privilege, self.network.me().guid, contents));
    }

    /// Returns the current value of TRecord wrapped in Rc to avoid excess cloning. This will fail if TRecord is not registered with GlobalRecords
    #[inline]
    pub fn read<TRecord>(&self) -> Rc<TRecord>
    where
        TRecord: RecordCommon + 'static,
    {
        // get the global id for TRecord
        let record_id = TRecord::id();

        // get the entry for the id
        let entry = self.get_entry(record_id, type_name::<TRecord>());

        // clone and return
        Rc::clone(&entry.read)
            .downcast_rc::<TRecord>()
            .unwrap_or_else(|_| panic!("[{}] read: downcast failed for '{}' — this should never happen", self.network.me().guid, type_name::<TRecord>()))
    }

    /// Edit the TRecord.This will fail if TRecord is not registered with GlobalRecords
    #[inline]
    pub fn write<TRecord>(&mut self, edit_fn: impl Fn(&mut TRecord))
    where
        TRecord: RecordCommon + 'static,
    {
        // we grab the global id and scope for this TRecord
        let record_id = TRecord::id();
        let record_scope = TRecord::ownership();

        // get my id from the network
        let instance_id = self.network.me().guid;

        //we first check if we have the ability to write based on the ledger restrictions and the record restrictions
        if !self
            .network_capabilities
            .has_write_privilege(record_scope.clone())
        {
            Curio::log(Severity::Warning, &format!("[{}] write: no write permission for '{}'", instance_id, type_name::<TRecord>()));
            return;
        }

        // create a synchronizer while running the write command. if NetworkCapabilities dont allow it returns None
        let synchronizer = {
            // get an entry for the id
            let entry = self.get_entry_mut(record_id, instance_id, type_name::<TRecord>());

            // downcast the entry write value as the TRecord
            let record = entry
                .write
                .downcast_mut::<TRecord>()
                .unwrap_or_else(|| panic!("[{}] write: downcast failed for '{}' — this should never happen", instance_id, type_name::<TRecord>()));

            // apply the edit fn to the cast TRecord
            edit_fn(record);

            // update the entries read value to match the new write value
            entry.sync_read();

            if record_scope != RecordScope::Instance {
                // if the scope is not Instance we use our new entry.write to create a RecordSynchronizer
                entry
                    .write
                    .downcast_ref::<TRecord>()
                    .and_then(|val| RecordSynchronizer::serialize(val))
            } else {
                // otherwise we just return none
                None
            }
        };

        // if we got a synchronizer we are going to try to add it to the queue
        if let Some(synchronizer) = synchronizer {
            self.network_capabilities.enqueue_synchronizer(synchronizer);
        }
    }

    /// Try to overwrite any Records with the provided RecordSynchronizers
    pub fn try_apply_synchronizers(&mut self, sync: &[RecordSynchronizer]) {
        for evnt in sync {
            if let Some(value) = evnt.deserialize() {
                let index = evnt.record_id as usize;
                if let Some(Some(entry)) = self.entries.get_mut(index) {
                    entry.write = value;
                    entry.sync_read();
                } else {
                    Curio::log(Severity::Error, &format!("[{}] try_apply_network_sync_events: no entry for id {}", self.network.me().guid, evnt.record_id));
                }
            } else {
                Curio::log(Severity::Error, &format!("[{}] try_apply_network_sync_events: deserialize failed for id {}", self.network.me().guid, evnt.record_id));
            }
        }
    }

    /// Try to drain any RecordSynchronizers that have been created from write fn calls
    pub fn try_drain_synchronizers(&mut self) -> Vec<RecordSynchronizer> {
        self.network_capabilities.drain_synchronizers()
    }
}
