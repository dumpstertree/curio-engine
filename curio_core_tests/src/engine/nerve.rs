#[cfg(test)]
mod tests {
    use curio_core::{
        CurioNetwork, CurioNetworkParticipant, GlobalImpulses, ImpulseCommon,
        ImpulseScope, ImpulseSynchronizer, Nerve, NetworkModes,
    };
    use crate::engine::services::ensure_services;
    use serde::Deserialize;
    use serde::Serialize;
    use std::{
        fmt::{Display, Formatter, Result},
        thread::sleep,
    };

    // All-owned impulse: must be shared across the network
    #[derive(Default, Clone, Serialize, Deserialize)]
    struct NerveProbe {
        pub value: i32,
    }
    impl Display for NerveProbe {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "NerveProbe({})", self.value)
        }
    }
    impl ImpulseCommon for NerveProbe {
        fn id() -> i32 {
            9101
        }
        fn ownership(&self) -> ImpulseScope {
            ImpulseScope::All
        }
    }

    // Instance-owned impulse: must stay in the local cache
    #[derive(Default, Clone, Serialize, Deserialize)]
    struct NerveProbeLocal {
        pub value: i32,
    }
    impl Display for NerveProbeLocal {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "NerveProbeLocal({})", self.value)
        }
    }
    impl ImpulseCommon for NerveProbeLocal {
        fn id() -> i32 {
            9102
        }
        fn ownership(&self) -> ImpulseScope {
            ImpulseScope::Instance
        }
    }

    fn seat_network(mode: NetworkModes) -> CurioNetwork {
        CurioNetwork::new(vec![CurioNetworkParticipant::new(222, mode)], 0)
    }

    // =========================================================
    // Immediate events
    // =========================================================

    #[test]
    fn enqueue_event_drains_back_same_value() {
        ensure_services();
        GlobalImpulses::register::<NerveProbe>();

        let mut nerve = Nerve::new(seat_network(NetworkModes::LocalHost));

        nerve.enqueue_event(NerveProbe { value: 3 });

        let drained = nerve.drain_queued_events::<NerveProbe>();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].value, 3);

        let again = nerve.drain_queued_events::<NerveProbe>();
        assert_eq!(again.len(), 0, "drain must empty the cache");
    }

    #[test]
    fn multiple_events_all_drain() {
        ensure_services();
        GlobalImpulses::register::<NerveProbe>();

        let mut nerve = Nerve::new(seat_network(NetworkModes::LocalHost));

        nerve.enqueue_event(NerveProbe { value: 1 });
        nerve.enqueue_event(NerveProbe { value: 2 });

        let drained = nerve.drain_queued_events::<NerveProbe>();
        assert_eq!(drained.len(), 2);
    }

    #[test]
    fn instance_scope_never_enqueues_synchronizers() {
        ensure_services();
        GlobalImpulses::register::<NerveProbeLocal>();

        let mut nerve = Nerve::new(seat_network(NetworkModes::LocalHost));

        nerve.enqueue_event(NerveProbeLocal { value: 5 });

        let drained = nerve.drain_queued_events::<NerveProbeLocal>();
        assert_eq!(drained.len(), 1);

        let syncs = nerve.try_drain_synchronizers();
        assert_eq!(syncs.len(), 0, "instance-owned impulses are not synced");
    }

    #[test]
    fn peer_privilege_caches_event_but_drops_sync() {
        ensure_services();
        GlobalImpulses::register::<NerveProbe>();

        let mut nerve = Nerve::new(seat_network(NetworkModes::LocalPeer));

        nerve.enqueue_event(NerveProbe { value: 6 });

        let drained = nerve.drain_queued_events::<NerveProbe>();
        assert_eq!(drained.len(), 1);

        let syncs = nerve.try_drain_synchronizers();
        assert_eq!(syncs.len(), 0, "a peer cannot write all-owned impulse syncs");
    }

    // =========================================================
    // Network sync round trip
    // =========================================================

    #[test]
    fn host_write_enqueues_one_synchronizer() {
        ensure_services();
        GlobalImpulses::register::<NerveProbe>();

        let mut nerve = Nerve::new(seat_network(NetworkModes::LocalHost));

        nerve.enqueue_event(NerveProbe { value: 8 });

        let syncs = nerve.try_drain_synchronizers();
        assert_eq!(syncs.len(), 1);
        assert_eq!(syncs[0].impulse_id, 9101);
        assert!(syncs[0].impulse_scope == ImpulseScope::All);
    }

    #[test]
    fn try_apply_synchronizers_refills_cache() {
        ensure_services();
        GlobalImpulses::register::<NerveProbe>();

        let mut nerve = Nerve::new(seat_network(NetworkModes::LocalHost));

        nerve.enqueue_event(NerveProbe { value: 8 });

        // drain the cache and the sync queue, then replay the sync
        // as if it had arrived from another participant
        nerve.drain_queued_events::<NerveProbe>();
        let syncs = nerve.try_drain_synchronizers();
        assert_eq!(syncs.len(), 1);

        nerve.try_apply_synchronizers(syncs);

        let replayed = nerve.drain_queued_events::<NerveProbe>();
        assert_eq!(replayed.len(), 1);
        assert_eq!(replayed[0].value, 8);
    }

    // =========================================================
    // Delayed events
    // =========================================================

    #[test]
    fn delayed_event_does_not_fire_immediately() {
        ensure_services();
        GlobalImpulses::register::<NerveProbe>();

        let mut nerve = Nerve::new(seat_network(NetworkModes::LocalHost));

        nerve.enqueue_event_delayed(NerveProbe { value: 4 }, 1.0);
        nerve.update_timed_events();

        let drained = nerve.drain_queued_events::<NerveProbe>();
        assert_eq!(drained.len(), 0, "delayed events stay pending before their timer expires");
    }

    #[test]
    fn delayed_event_fires_after_one_second() {
        ensure_services();
        GlobalImpulses::register::<NerveProbe>();

        let mut nerve = Nerve::new(seat_network(NetworkModes::LocalHost));

        nerve.enqueue_event_delayed(NerveProbe { value: 4 }, 1.0);
        sleep(std::time::Duration::from_millis(1100));
        nerve.update_timed_events();

        let drained = nerve.drain_queued_events::<NerveProbe>();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].value, 4);

        let syncs = nerve.try_drain_synchronizers();
        assert_eq!(syncs.len(), 1, "fired delayed events still honour scope");
    }

    #[test]
    fn delayed_event_ignores_requested_delay() {
        ensure_services();
        GlobalImpulses::register::<NerveProbe>();

        // BUG: enqueue_event_delayed discards its _delay parameter and update_timed_events
        // hardcodes a one-second expiry, so a 30 second request fires after 1.1 seconds
        let mut nerve = Nerve::new(seat_network(NetworkModes::LocalHost));

        nerve.enqueue_event_delayed(NerveProbe { value: 9 }, 30.0);
        sleep(std::time::Duration::from_millis(1100));
        nerve.update_timed_events();

        let drained = nerve.drain_queued_events::<NerveProbe>();
        assert_eq!(drained.len(), 1, "delayed events fire on the hardcoded timer, not the requested delay");
    }
}
