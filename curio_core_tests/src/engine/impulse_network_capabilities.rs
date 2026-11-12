#[cfg(test)]
mod tests {
    use curio_core::{GlobalImpulses, ImpulseCommon, ImpulseScope, ImpulseSynchronizer, ImpulseNetworkCapabilities, NetworkModes};
    use crate::engine::services::ensure_services;
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Default, Clone, Serialize, Deserialize)]
    struct CapImpulse {
        pub value: i32,
    }
    impl ImpulseCommon for CapImpulse {
        fn id() -> i32 {
            9103
        }
        fn ownership(&self) -> ImpulseScope {
            ImpulseScope::All
        }
    }

    // =========================================================
    // Privilege matrix
    // =========================================================

    #[test]
    fn instance_scope_never_writable() {
        ensure_services();
        // note the asymmetry with RecordScope::Instance, which is always
        // writable: instance-owned impulses never need to be sent anywhere
        for mode in NetworkModes::all() {
            let caps = ImpulseNetworkCapabilities::new(mode);
            assert!(
                !caps.has_write_privilege(ImpulseScope::Instance),
                "instance-owned impulses are never synced",
            );
        }
    }

    #[test]
    fn network_scopes_writable_only_for_hosts() {
        ensure_services();
        for mode in NetworkModes::all() {
            let caps = ImpulseNetworkCapabilities::new(mode);
            let is_host = (mode == NetworkModes::LocalHost) || (mode == NetworkModes::OnlineHost);
            assert_eq!(caps.has_write_privilege(ImpulseScope::All), is_host);
            assert_eq!(caps.has_write_privilege(ImpulseScope::ConnectedHost), is_host);
            assert_eq!(caps.has_write_privilege(ImpulseScope::ConnectedPeers), is_host);
        }
    }

    // =========================================================
    // Synchronizer queue
    // =========================================================

    #[test]
    fn enqueue_then_drain_returns_everything() {
        ensure_services();
        GlobalImpulses::register::<CapImpulse>();

        let mut caps = ImpulseNetworkCapabilities::new(NetworkModes::LocalHost);

        let Some(first) = ImpulseSynchronizer::serialize::<CapImpulse>(&CapImpulse { value: 1 });
        caps.enqueue_synchronizer(first);

        let drained = caps.drain_synchronizers();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].impulse_id, 9103);
    }

    #[test]
    fn drain_empties_the_queue() {
        ensure_services();
        GlobalImpulses::register::<CapImpulse>();

        let mut caps = ImpulseNetworkCapabilities::new(NetworkModes::LocalHost);

        let Some(first) = ImpulseSynchronizer::serialize::<CapImpulse>(&CapImpulse { value: 1 });
        caps.enqueue_synchronizer(first);

        caps.drain_synchronizers();
        let again = caps.drain_synchronizers();
        assert_eq!(again.len(), 0, "drain must clear the queue");
    }
}
