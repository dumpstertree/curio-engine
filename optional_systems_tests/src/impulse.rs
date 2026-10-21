#[cfg(test)]
mod tests {
    use curio_core::{ImpulseCommon, ImpulseScope};
    use gameplay::traits::ui_events::IUIEvent;
    use impulse::{curio_impulse, curio_ui_event};

    // three enums exercising id uniqueness and the scope overrides

    #[curio_impulse]
    pub enum ImpulseA {}

    #[curio_impulse]
    pub enum ImpulseB {}

    #[curio_impulse(scope = "connected_peers")]
    pub enum ImpulseC {}

    #[curio_impulse(scope = "connected_host")]
    pub enum NetImpulse {
        Ping,
    }

    #[curio_ui_event]
    pub enum TestUIEvents {}

    #[test]
    fn impulse_ids_are_unique_per_type() {
        let a = ImpulseA::id();
        let b = ImpulseB::id();
        let c = ImpulseC::id();
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
        // ids are stable across calls
        assert_eq!(ImpulseA::id(), a);
    }

    #[test]
    fn impulse_ids_no_longer_collide_on_zero() {
        // the whole point of the derive: no hand-written impls hardcoding 0
        assert_ne!(ImpulseA::id(), 0);
        assert_ne!(ImpulseB::id(), 0);
    }

    #[test]
    fn ownership_scopes_are_respected() {
        // empty enums cannot produce values, so exercise ownership through a variant
        assert!(matches!(NetImpulse::Ping.ownership(), ImpulseScope::ConnectedHost));
        // and generated Clone works (the old hand-rolled boilerplate panicked here)
        let cloned = NetImpulse::Ping.clone();
        assert!(matches!(cloned, NetImpulse::Ping));
    }

    #[test]
    fn impulse_types_satisfy_engine_bounds() {
        fn assert_impulse<T: ImpulseCommon + 'static>() {}
        assert_impulse::<ImpulseA>();
        assert_impulse::<ImpulseB>();
        assert_impulse::<ImpulseC>();
    }

    #[test]
    fn ui_event_type_satisfies_engine_bounds() {
        fn assert_ui_event<T: IUIEvent + 'static>() {}
        assert_ui_event::<TestUIEvents>();
    }

    #[test]
    fn ui_event_derives_present() {
        // Clone/Copy/PartialEq/Eq/Hash/Display are generated; assert them via bounds + hashing
        fn assert_all<T: Clone + Copy + PartialEq + Eq + std::hash::Hash + std::fmt::Display>() {}
        assert_all::<TestUIEvents>();
    }
}
