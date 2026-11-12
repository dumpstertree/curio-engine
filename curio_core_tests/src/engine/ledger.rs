#[cfg(test)]
mod tests {
    use curio_core::{
        CurioNetwork, CurioNetworkParticipant, FieldState, GlobalRecords, Ledger,
        NetworkModes, RecordCommon, RecordOverride, RecordScope,
    };
    use crate::engine::services::ensure_services;
    use serde::Deserialize;
    use serde::Serialize;

    // Test records. Local is instance-owned (always writable), Host is host-owned
    // (only writable with host privilege). Unregistered is deliberately never
    // registered with GlobalRecords.
    #[derive(Clone, Serialize, Deserialize)]
    struct LedgerProbeLocal {
        pub value: i32,
    }
    impl Default for LedgerProbeLocal {
        fn default() -> Self {
            LedgerProbeLocal { value: 7 }
        }
    }
    impl RecordCommon for LedgerProbeLocal {
        fn name(&self) -> String {
            String::from("LedgerProbeLocal")
        }
        fn id() -> i32 {
            9001
        }
    }
    impl RecordOverride for LedgerProbeLocal {
        fn set_state(&mut self, _field: &str, _val: &str) {}
        fn get_state(&self) -> Vec<FieldState> {
            vec![FieldState::new("value", self.value)]
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    struct LedgerProbeHost {
        pub value: i32,
    }
    impl Default for LedgerProbeHost {
        fn default() -> Self {
            LedgerProbeHost { value: 1 }
        }
    }
    impl RecordCommon for LedgerProbeHost {
        fn name(&self) -> String {
            String::from("LedgerProbeHost")
        }
        fn id() -> i32 {
            9002
        }
        fn ownership() -> RecordScope {
            RecordScope::Host
        }
    }
    impl RecordOverride for LedgerProbeHost {
        fn set_state(&mut self, _field: &str, _val: &str) {}
        fn get_state(&self) -> Vec<FieldState> {
            vec![FieldState::new("value", self.value)]
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    struct LedgerProbeUnregistered {
        pub value: i32,
    }
    impl Default for LedgerProbeUnregistered {
        fn default() -> Self {
            LedgerProbeUnregistered { value: 0 }
        }
    }
    impl RecordCommon for LedgerProbeUnregistered {
        fn name(&self) -> String {
            String::from("LedgerProbeUnregistered")
        }
        fn id() -> i32 {
            9990
        }
    }
    impl RecordOverride for LedgerProbeUnregistered {
        fn set_state(&mut self, _field: &str, _val: &str) {}
        fn get_state(&self) -> Vec<FieldState> {
            vec![]
        }
    }

    fn seat_network(mode: NetworkModes) -> CurioNetwork {
        CurioNetwork::new(vec![CurioNetworkParticipant::new(111, mode)], 0)
    }

    // =========================================================
    // Ledger creation from the global registry
    // =========================================================

    #[test]
    fn ledger_new_populates_registered_records() {
        ensure_services();
        GlobalRecords::register_serializable::<LedgerProbeLocal>();

        let ledger = Ledger::new(seat_network(NetworkModes::LocalHost));

        let record = ledger.read::<LedgerProbeLocal>();
        assert_eq!(record.value, 7, "entry must be seeded from the registration default");
    }

    #[test]
    fn ledger_privilege_matches_network_mode() {
        ensure_services();
        let ledger = Ledger::new(seat_network(NetworkModes::LocalPeer));

        assert_eq!(ledger.network_capabilities.privilege, NetworkModes::LocalPeer);
    }

    // =========================================================
    // Read / write
    // =========================================================

    #[test]
    fn write_updates_the_read_copy() {
        ensure_services();
        GlobalRecords::register_serializable::<LedgerProbeLocal>();

        let mut ledger = Ledger::new(seat_network(NetworkModes::LocalHost));

        ledger.write::<LedgerProbeLocal>(|record| record.value = 42);

        let record = ledger.read::<LedgerProbeLocal>();
        assert_eq!(record.value, 42);
    }

    #[test]
    fn instance_owned_writes_never_enqueue_synchronizers() {
        ensure_services();
        GlobalRecords::register_serializable::<LedgerProbeLocal>();

        let mut ledger = Ledger::new(seat_network(NetworkModes::LocalHost));

        ledger.write::<LedgerProbeLocal>(|record| record.value = 42);

        let queue = ledger.try_drain_synchronizers();
        assert_eq!(queue.len(), 0, "instance-owned records stay local");
    }

    // =========================================================
    // Host-owned writes and privilege
    // =========================================================

    #[test]
    fn host_owned_write_with_host_privilege_enqueues_synchronizer() {
        ensure_services();
        GlobalRecords::register_serializable::<LedgerProbeHost>();

        let mut ledger = Ledger::new(seat_network(NetworkModes::LocalHost));

        ledger.write::<LedgerProbeHost>(|record| record.value = 99);

        let queue = ledger.try_drain_synchronizers();
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].record_id, 9002);
        assert_eq!(queue[0].record_scope, RecordScope::Host);
    }

    #[test]
    fn peer_write_on_host_record_is_rejected() {
        ensure_services();
        GlobalRecords::register_serializable::<LedgerProbeHost>();

        let mut ledger = Ledger::new(seat_network(NetworkModes::LocalPeer));

        ledger.write::<LedgerProbeHost>(|record| record.value = 99);

        let record = ledger.read::<LedgerProbeHost>();
        assert_eq!(record.value, 1, "a peer must not be able to write host-owned records");

        let queue = ledger.try_drain_synchronizers();
        assert_eq!(queue.len(), 0, "a rejected write must not enqueue a synchronizer");
    }

    #[test]
    fn synchronizer_roundtrip_restores_value_into_entry() {
        ensure_services();
        GlobalRecords::register_serializable::<LedgerProbeHost>();

        let mut ledger = Ledger::new(seat_network(NetworkModes::LocalHost));

        ledger.write::<LedgerProbeHost>(|record| record.value = 99);
        let queue = ledger.try_drain_synchronizers();
        assert_eq!(queue.len(), 1);

        // pretend the synchronizer came back from another network participant
        ledger.try_apply_synchronizers(&queue);

        let record = ledger.read::<LedgerProbeHost>();
        assert_eq!(record.value, 99);
    }

    #[test]
    #[should_panic]
    fn unregistered_record_read_panics() {
        ensure_services();
        let ledger = Ledger::new(seat_network(NetworkModes::LocalHost));

        // 9990 was never registered - no entry exists for it
        ledger.read::<LedgerProbeUnregistered>();
    }

    // =========================================================
    // Serialization view
    // =========================================================

    #[test]
    fn serializable_reports_ledger_tab() {
        ensure_services();
        GlobalRecords::register_serializable::<LedgerProbeLocal>();

        let ledger = Ledger::new(seat_network(NetworkModes::LocalHost));
        let report = ledger.serializable();

        assert_eq!(report.1.tab_name, "Ledger");

        let found = report.1.objects.iter().any(|obj| obj.object_name == "LedgerProbeLocal");
        assert(found);
    }

    // =========================================================
    // Known bug: new_custom is unusable
    // =========================================================

    #[test]
    #[should_panic]
    fn new_custom_panics_on_empty_network() {
        ensure_services();
        // BUG: new_custom leaves CurioNetwork empty, then logs through
        // self.network.me() - which indexes an empty vec. Every call panics.
        let entries: Vec<(i32, Box<dyn RecordCommon>)> = vec![(9001, Box::new(LedgerProbeLocal { value: 3 }))];
        Ledger::new_custom(entries);
    }
}
