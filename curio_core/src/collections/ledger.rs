use std::any::type_name;
use std::rc::Rc;
use std::vec;

use crate::built_in::record::sys_record_screen::SysRecordScreen;
use crate::built_in::record::sys_record_time::SysRecordTime;
use crate::collections::network_capabilities::NetworkCapabilities;
use crate::collections::network_modes::NetworkModes;
use crate::collections::state_ownerships::StateOwnerships;
use crate::collections::state_sync_event::StateSyncEvent;
use crate::static_data::global_states::get_global_state_constructor_all;
use crate::system::system_game_state::RecordCommon;
use crate::{log, Severity};

// -------------------------------------------------------------------------
// Internal entry — owns both sides of a single state type
// -------------------------------------------------------------------------

struct Entry {
    /// Authoritative mutable copy. Downcast via `downcast_mut::<T>()`.
    write: Box<dyn RecordCommon>,
    /// Shared snapshot. Downcast via `downcast_rc::<T>()`.
    /// Replaced on every write — existing Rc handles remain valid and
    /// point to the previous value until they are dropped.
    read: Rc<dyn RecordCommon>,
}

impl Entry {
    /// Build from a boxed value. Clones once to seed the read Rc.
    fn new(value: Box<dyn RecordCommon>) -> Self {
        let read = Rc::from(value.clone_box());
        Entry { write: value, read }
    }

    /// Rebuild the read Rc from the current write value.
    /// Called after any mutation so readers immediately see the new value.
    fn sync_read(&mut self) {
        self.read = Rc::from(self.write.clone_box());
    }
}

impl Clone for Entry {
    fn clone(&self) -> Self {
        Entry {
            write: self.write.clone_box(),
            read: Rc::from(self.write.clone_box()),
        }
    }
}

// -------------------------------------------------------------------------
// Ledger
// -------------------------------------------------------------------------

#[derive(Clone)]
pub struct Ledger {
    pub name: String,
    pub instance_id: i32,
    pub all_instance_id: Vec<i32>,
    /// Direct-indexed by sequential state ID. `None` means that slot is
    /// unregistered. Because IDs are 0-based and packed, indexing is O(1)
    /// with no hashing and excellent cache locality.
    entries: Vec<Option<Entry>>,
    pub network_capabilities: Option<NetworkCapabilities>,
}

impl Ledger {
    // -------------------------------------------------------------------------
    // Constructors
    // -------------------------------------------------------------------------

    /// Full instance populated from the global state registry, with networking.
    pub fn new(name: &str, network_mode: NetworkModes, instance_id: i32, all_instance_id: Vec<i32>) -> Self {
        let constructors = get_global_state_constructor_all();

        let max_id = constructors.iter().map(|(id, _)| *id).max().unwrap_or(-1);
        let mut entries: Vec<Option<Entry>> = (0..=max_id.max(0)).map(|_| None).collect();

        for (id, constructor) in constructors {
            entries[id as usize] = Some(Entry::new(constructor()));
        }

        let ledger = Ledger {
            name: name.to_string(),
            instance_id,
            all_instance_id,
            entries,
            network_capabilities: Some(NetworkCapabilities::new(network_mode)),
        };

        ledger.log(Severity::Info, "Created!");
        ledger
    }

    /// Lightweight instance from an explicit list of states, without networking.
    /// Both read and write are seeded from the provided values so `read()` works
    /// immediately without needing a prior `write()` call.
    pub fn new_single_instance(states: Vec<(i32, Box<dyn RecordCommon>)>) -> Self {
        let max_id = states.iter().map(|(id, _)| *id).max().unwrap_or(-1);
        let mut entries: Vec<Option<Entry>> = (0..=max_id.max(0)).map(|_| None).collect();

        for (id, value) in states {
            entries[id as usize] = Some(Entry::new(value));
        }

        let ledger = Ledger {
            name: String::from(""),
            instance_id: -1,
            all_instance_id: vec![-1],
            entries,
            network_capabilities: None,
        };

        ledger.log(Severity::Info, "Created (single instance)!");
        ledger
    }

    // -------------------------------------------------------------------------
    // Logging
    // -------------------------------------------------------------------------

    pub fn log(&self, severity: Severity, contents: &str) {
        log(severity, &format!("[{}]: {}", self.instance_id, contents));
    }

    // -------------------------------------------------------------------------
    // Convenience accessors
    // -------------------------------------------------------------------------

    pub fn time(&self) -> Rc<SysRecordTime> {
        self.read::<SysRecordTime>()
    }

    pub fn screen(&self) -> Rc<SysRecordScreen> {
        self.read::<SysRecordScreen>()
    }

    // -------------------------------------------------------------------------
    // Internal slot access
    // -------------------------------------------------------------------------

    #[inline]
    fn get_entry(&self, id: i32, type_str: &str) -> &Entry {
        self.entries
            .get(id as usize)
            .and_then(|slot| slot.as_ref())
            .unwrap_or_else(|| panic!("[{}] type '{}' is not registered (id {})", self.instance_id, type_str, id))
    }

    #[inline]
    fn get_entry_mut(&mut self, id: i32, instance_id: i32, type_str: &str) -> &mut Entry {
        self.entries
            .get_mut(id as usize)
            .and_then(|slot| slot.as_mut())
            .unwrap_or_else(|| panic!("[{}] type '{}' is not registered (id {})", instance_id, type_str, id))
    }

    // -------------------------------------------------------------------------
    // Core API
    // -------------------------------------------------------------------------

    /// Returns a cloned `Rc<TRecord>` — just a refcount increment, no allocation.
    /// All callers share the same allocation until the next `write()`.
    /// Panics if `TRecord` was not registered.
    #[inline]
    pub fn read<TRecord>(&self) -> Rc<TRecord>
    where
        TRecord: RecordCommon + 'static,
    {
        let id = TRecord::id();
        let entry = self.get_entry(id, type_name::<TRecord>());

        Rc::clone(&entry.read)
            .downcast_rc::<TRecord>()
            .unwrap_or_else(|_| panic!("[{}] read: downcast failed for '{}' — this should never happen", self.instance_id, type_name::<TRecord>()))
    }

    /// Mutate `TRecord` via `edit_fn`, then immediately publish a new read snapshot.
    /// Panics if `TRecord` was not registered.
    #[inline]
    pub fn write<TRecord>(&mut self, edit_fn: impl Fn(&mut TRecord))
    where
        TRecord: RecordCommon + 'static,
    {
        let id = TRecord::id();
        let ownership = TRecord::ownership();
        let instance_id = self.instance_id;

        if let Some(net) = &self.network_capabilities {
            if !net.has_write_privilege(ownership.clone()) {
                eprintln!("[{}] write: no write permission for '{}'", instance_id, type_name::<TRecord>());
                return;
            }
        }

        // Produce the sync event inside this scope so the mutable borrow
        // of self.entries ends before we touch self.network_capabilities.
        let sync_event = {
            let entry = self.get_entry_mut(id, instance_id, type_name::<TRecord>());

            let state = entry
                .write
                .downcast_mut::<TRecord>()
                .unwrap_or_else(|| panic!("[{}] write: downcast failed for '{}' — this should never happen", instance_id, type_name::<TRecord>()));

            edit_fn(state);
            entry.sync_read();

            if ownership != StateOwnerships::Instance {
                entry
                    .write
                    .downcast_ref::<TRecord>()
                    .and_then(|val| StateSyncEvent::serialize(val))
            } else {
                None
            }
        };

        if let Some(event) = sync_event {
            if let Some(net) = &mut self.network_capabilities {
                net.enqueue_sync_events(event);
            }
        }
    }

    // -------------------------------------------------------------------------
    // Networking
    // -------------------------------------------------------------------------

    /// Apply incoming sync events from other instances.
    /// Overwrites both the write-side value and immediately syncs the read Rc.
    pub fn try_apply_network_sync_events(&mut self, sync: &[StateSyncEvent]) {
        if self.network_capabilities.is_none() {
            return;
        }

        for evnt in sync {
            if let Some(value) = evnt.deserialize() {
                let index = evnt.id as usize;
                if let Some(Some(entry)) = self.entries.get_mut(index) {
                    entry.write = value;
                    entry.sync_read();
                } else {
                    eprintln!("[{}] try_apply_network_sync_events: no entry for id {}", self.instance_id, evnt.id);
                }
            } else {
                eprintln!("[{}] try_apply_network_sync_events: deserialize failed for id {}", self.instance_id, evnt.id);
            }
        }
    }

    /// Drain all pending outbound sync events produced this frame.
    pub fn try_drain_network_sync_events(&mut self) -> Vec<StateSyncEvent> {
        self.network_capabilities
            .as_mut()
            .map_or_else(Vec::new, |nc| nc.drain_sync_events())
    }
}
