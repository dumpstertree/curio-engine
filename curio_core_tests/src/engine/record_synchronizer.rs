#[cfg(test)]
mod tests {
    use curio_core::{FieldState, GlobalRecords, RecordCommon, RecordOverride, RecordScope, RecordSynchronizer};
    use crate::engine::services::ensure_services;
    use serde::Deserialize;
    use serde::Serialize;

    // Host-owned record so the stamped scope is interesting
    #[derive(Clone, Serialize, Deserialize)]
    struct RecordSyncProbe {
        pub value: i32,
    }
    impl Default for RecordSyncProbe {
        fn default() -> Self {
            RecordSyncProbe { value: 0 }
        }
    }
    impl RecordCommon for RecordSyncProbe {
        fn name(&self) -> String {
            String::from("RecordSyncProbe")
        }
        fn id() -> i32 {
            9003
        }
        fn ownership() -> RecordScope {
            RecordScope::Host
        }
    }
    impl RecordOverride for RecordSyncProbe {
        fn set_state(&mut self, _field: &str, _val: &str) {}
        fn get_state(&self) -> Vec<FieldState> {
            vec![FieldState::new("value", self.value)]
        }
    }

    // registered without serialization data (GlobalRecords::register)
    #[derive(Clone, Serialize, Deserialize)]
    struct RecordSyncNoSerde {
        pub value: i32,
    }
    impl Default for RecordSyncNoSerde {
        fn default() -> Self {
            RecordSyncNoSerde { value: 0 }
        }
    }
    impl RecordCommon for RecordSyncNoSerde {
        fn name(&self) -> String {
            String::from("RecordSyncNoSerde")
        }
        fn id() -> i32 {
            9004
        }
    }
    impl RecordOverride for RecordSyncNoSerde {
        fn set_state(&mut self, _field: &str, _val: &str) {}
        fn get_state(&self) -> Vec<FieldState> {
            vec![]
        }
    }

    // deliberately never registered with GlobalRecords
    #[derive(Clone, Serialize, Deserialize)]
    struct RecordSyncUnregistered {
        pub value: i32,
    }
    impl Default for RecordSyncUnregistered {
        fn default() -> Self {
            RecordSyncUnregistered { value: 0 }
        }
    }
    impl RecordCommon for RecordSyncUnregistered {
        fn name(&self) -> String {
            String::from("RecordSyncUnregistered")
        }
        fn id() -> i32 {
            9992
        }
    }
    impl RecordOverride for RecordSyncUnregistered {
        fn set_state(&mut self, _field: &str, _val: &str) {}
        fn get_state(&self) -> Vec<FieldState> {
            vec![]
        }
    }

    // =========================================================
    // Serialize
    // =========================================================

    #[test]
    fn serialize_stamps_id_and_scope() {
        ensure_services();
        GlobalRecords::register_serializable::<RecordSyncProbe>();

        let sync = RecordSynchronizer::serialize::<RecordSyncProbe>(&RecordSyncProbe { value: 42 });
        assert!(sync.is_some(), "registered serializable records must serialize");

        let Some(stamped) = sync;
        assert_eq!(stamped.record_id, 9003);
        assert_eq!(stamped.record_scope, RecordScope::Host);
    }

    #[test]
    fn unregistered_record_fails_to_serialize() {
        ensure_services();
        let sync = RecordSynchronizer::serialize::<RecordSyncUnregistered>(&RecordSyncUnregistered { value: 1 });
        assert!(sync.is_none(), "unregistered records must not serialize");
    }

    #[test]
    fn record_registered_without_serde_fails_to_serialize() {
        ensure_services();
        // register() stores no serialize/deserialize fns - the serialize
        // attempt must bail with None rather than a panic
        GlobalRecords::register::<RecordSyncNoSerde>();

        let sync = RecordSynchronizer::serialize::<RecordSyncNoSerde>(&RecordSyncNoSerde { value: 1 });
        assert!(sync.is_none());
    }

    // =========================================================
    // Round trip
    // =========================================================

    #[test]
    fn serialize_deserialize_roundtrip() {
        ensure_services();
        GlobalRecords::register_serializable::<RecordSyncProbe>();

        let sync = RecordSynchronizer::serialize::<RecordSyncProbe>(&RecordSyncProbe { value: 42 });
        let Some(payload) = sync;

        let boxed = payload.deserialize();
        let Some(boxed) = boxed;

        let Ok(recovered) = boxed.downcast::<RecordSyncProbe>();
        assert_eq!(recovered.value, 42);
    }

    // Note: the deserialize-None paths (unregistered id / registration without
    // deserialize_fn) are unreachable from outside the crate because the
    // payload field is private and RecordSynchronizer can only be produced by
    // a successful serialize.
}
