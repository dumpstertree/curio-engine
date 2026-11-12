#[cfg(test)]
mod tests {
    use curio_core::{
        FieldState,
        GlobalRecords,
        NetworkModes,
        RecordCommon,
        RecordNetworkCapabilities,
        RecordOverride,
        RecordScope,
        RecordSynchronizer,
    };
    use crate::engine::services::ensure_services;
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Clone, Serialize, Deserialize)]
    struct CapProbe {
        pub value: i32,
    }
    impl Default for CapProbe {
        fn default() -> Self {
            CapProbe { value: 0 }
        }
    }
    impl RecordCommon for CapProbe {
        fn name(&self) -> String {
            String::from("CapProbe")
        }
        fn id() -> i32 {
            9005
        }
    }
    impl RecordOverride for CapProbe {
        fn set_state(&mut self, _field: &str, _val: &str) {}
        fn get_state(&self) -> Vec<FieldState> {
            vec![FieldState::new("value", self.value)]
        }
    }

    // =========================================================
    // Privilege matrix
    // =========================================================

    #[test]
    fn instance_scope_always_writable() {
        ensure_services();
        for mode in NetworkModes::all() {
            let caps = RecordNetworkCapabilities::new(mode);
            assert!(caps.has_write_privilege(RecordScope::Instance), "instance-owned records are always writable");
        }
    }

    #[test]
    fn host_scope_writable_only_for_hosts() {
        ensure_services();
        for mode in NetworkModes::all() {
            let caps = RecordNetworkCapabilities::new(mode);
            let is_host = (mode == NetworkModes::LocalHost) || (mode == NetworkModes::OnlineHost);
            assert_eq!(caps.has_write_privilege(RecordScope::Host), is_host);
        }
    }

    // =========================================================
    // Synchronizer queue
    // =========================================================

    #[test]
    fn enqueue_then_drain_returns_everything() {
        ensure_services();
        GlobalRecords::register_serializable::<CapProbe>();

        let mut caps = RecordNetworkCapabilities::new(NetworkModes::LocalHost);

        let Some(first) = RecordSynchronizer::serialize::<CapProbe>(&CapProbe { value: 1 }) else { panic!("expected Some") };
        let Some(second) = RecordSynchronizer::serialize::<CapProbe>(&CapProbe { value: 2 }) else { panic!("expected Some") };
        caps.enqueue_synchronizer(first);
        caps.enqueue_synchronizer(second);

        let drained = caps.drain_synchronizers();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].record_id, 9005);
    }

    #[test]
    fn drain_empties_the_queue() {
        ensure_services();
        GlobalRecords::register_serializable::<CapProbe>();

        let mut caps = RecordNetworkCapabilities::new(NetworkModes::LocalHost);

        let Some(first) = RecordSynchronizer::serialize::<CapProbe>(&CapProbe { value: 1 }) else { panic!("expected Some") };
        caps.enqueue_synchronizer(first);

        caps.drain_synchronizers();
        let again = caps.drain_synchronizers();
        assert_eq!(again.len(), 0, "drain must clear the queue");
    }
}
