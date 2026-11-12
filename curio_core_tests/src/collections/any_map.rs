#[cfg(test)]
mod tests {
    use curio_core::AnyMap;

    #[derive(Clone)]
    struct MapProbeA {
        pub value: i32,
    }
    #[derive(Clone)]
    struct MapProbeB {
        pub value: i32,
    }

    // =========================================================
    // Insert / get
    // =========================================================

    #[test]
    fn insert_then_get_roundtrip() {
        let mut map: AnyMap<i32> = AnyMap::default();
        map.insert(5, MapProbeA { value: 4 });

        let Some(seen) = map.get::<MapProbeA, i32>(&5) else { panic!("expected Some") };
        assert_eq!(seen.value, 4);
    }

    #[test]
    fn get_mut_edits_in_place() {
        let mut map: AnyMap<i32> = AnyMap::default();
        map.insert(5, MapProbeA { value: 4 });

        let Some(seen) = map.get_mut::<MapProbeA, i32>(&5) else { panic!("expected Some") };
        seen.value = 9;

        let Some(again) = map.get::<MapProbeA, i32>(&5) else { panic!("expected Some") };
        assert_eq!(again.value, 9);
    }

    #[test]
    fn insert_overwrites_previous_value() {
        let mut map: AnyMap<i32> = AnyMap::default();
        map.insert(5, MapProbeA { value: 1 });
        map.insert(5, MapProbeB { value: 2 });

        assert!(
            map.get::<MapProbeA, i32>(&5).is_none(),
            "old value must be replaced, not kept alongside",
        );
        let Some(seen) = map.get::<MapProbeB, i32>(&5) else { panic!("expected Some") };
        assert_eq!(seen.value, 2);
    }

    #[test]
    fn insert_any_stores_a_boxed_value() {
        let mut map: AnyMap<i32> = AnyMap::default();
        map.insert_any(7, Box::new(MapProbeA { value: 3 }));

        let Some(seen) = map.get::<MapProbeA, i32>(&7) else { panic!("expected Some") };
        assert_eq!(seen.value, 3);
    }

    // =========================================================
    // Lookups that miss
    // =========================================================

    #[test]
    fn get_returns_none_for_both_missing_key_and_type_mismatch() {
        let mut map: AnyMap<i32> = AnyMap::default();
        map.insert(5, MapProbeA { value: 4 });

        assert!(map.get::<MapProbeA, i32>(&77).is_none(), "missing key");
        assert!(
            map.get::<MapProbeB, i32>(&5).is_none(),
            "wrong type under a present key",
        );
    }

    #[test]
    fn get_result_separates_ok_from_err() {
        let mut map: AnyMap<i32> = AnyMap::default();
        map.insert(5, MapProbeA { value: 4 });

        assert!(map.get_result::<MapProbeA, i32>(&5).is_ok());
        assert!(map.get_result::<MapProbeA, i32>(&77).is_err());
        assert!(map.get_result::<MapProbeB, i32>(&5).is_err());

        // Note: GetError::EmptyKey vs GetError::MismatchedType cannot be pinned
        // from outside the crate - the collections submodules are pub(crate)
        // and GetError has no root re-export (see audit_findings.md).
    }

    // =========================================================
    // Generic key types
    // =========================================================

    #[test]
    fn string_keyed_map_queries_with_str() {
        let mut map: AnyMap<String> = AnyMap::default();
        map.insert("hp".to_string(), MapProbeA { value: 8 });

        let Some(seen) = map.get::<MapProbeA, str>(&"hp") else { panic!("expected Some") };
        assert_eq!(seen.value, 8, "K: Borrow<Q> lookup path must work");
    }

    // Note: AnyMap has no remove/clear API - entries can be replaced but never
    // deleted. Documented in audit_findings.md.
}
