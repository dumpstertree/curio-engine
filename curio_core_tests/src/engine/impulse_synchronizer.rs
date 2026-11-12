#[cfg(test)]
mod tests {
    use curio_core::{GlobalImpulses, ImpulseCommon, ImpulseScope, ImpulseSynchronizer};
    use crate::engine::services::ensure_services;
    use serde::Deserialize;
    use serde::Serialize;
    use std::any::Any;

    #[derive(Default, Clone, Serialize, Deserialize)]
    struct SyncProbe {
        pub value: i32,
    }
    impl ImpulseCommon for SyncProbe {
        fn id() -> i32 {
            9201
        }
        fn ownership(&self) -> ImpulseScope {
            ImpulseScope::ConnectedHost
        }
    }

    // deliberately never registered with GlobalImpulses
    #[derive(Default, Clone, Serialize, Deserialize)]
    struct SyncProbeUnregistered {
        pub value: i32,
    }
    impl ImpulseCommon for SyncProbeUnregistered {
        fn id() -> i32 {
            9991
        }
        fn ownership(&self) -> ImpulseScope {
            ImpulseScope::All
        }
    }

    // =========================================================
    // Serialize
    // =========================================================

    #[test]
    fn serialize_stamps_id_and_scope() {
        ensure_services();
        GlobalImpulses::register::<SyncProbe>();

        let sync = ImpulseSynchronizer::serialize::<SyncProbe>(&SyncProbe { value: 1 });
        assert!(sync.is_some(), "registered impulses must serialize");

        let Some(stamped) = sync else { panic!("expected Some") };
        assert_eq!(stamped.impulse_id, 9201);
        assert!(stamped.impulse_scope == ImpulseScope::ConnectedHost);
    }

    #[test]
    fn unregistered_impulse_fails_to_serialize() {
        ensure_services();
        let sync = ImpulseSynchronizer::serialize::<SyncProbeUnregistered>(&SyncProbeUnregistered { value: 1 });
        assert!(sync.is_none(), "unregistered impulses must not serialize");
    }

    // =========================================================
    // Round trip
    // =========================================================

    #[test]
    fn serialize_deserialize_roundtrip() {
        ensure_services();
        GlobalImpulses::register::<SyncProbe>();

        let sync = ImpulseSynchronizer::serialize::<SyncProbe>(&SyncProbe { value: 77 });
        let Some(payload) = sync else { panic!("expected Some") };

        let boxed = payload.deserialize();
        let Some(boxed) = boxed else { panic!("expected Some") };

        let Ok(recovered) = boxed.downcast::<SyncProbe>() else { panic!("expected Ok") };
        assert_eq!(recovered.value, 77);
    }

    // Note: corrupt-payload paths are not covered - the payload field is private
    // and the registration's bincode calls unwrap(), so a bad payload is an
    // unrecoverable panic that cannot be produced from outside the crate.
}
