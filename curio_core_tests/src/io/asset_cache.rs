#[cfg(test)]
mod tests {
    use curio_core::{AssetCache, AssetCommon};

    use std::sync::Arc;

    #[derive(Debug, PartialEq)]
    struct TestAsset {
        value: i32,
    }

    impl AssetCommon<TestAsset> for TestAsset {
        fn from_bits(_bits: &Vec<u8>) -> TestAsset {
            todo!()
        }
    }

    #[test]
    fn stores_and_retrieves_asset() {
        let mut cache = AssetCache::new(10);

        let asset = Arc::new(TestAsset { value: 42 });

        cache.try_set_asset(&1, asset.clone());

        let retrieved = cache.try_get_asset::<TestAsset>(&1);

        assert!(retrieved.is_some());
        assert!(Arc::ptr_eq(&asset, retrieved.as_ref().unwrap()));
        assert_eq!(retrieved.unwrap().value, 42);
    }

    #[test]
    fn returns_none_for_missing_asset() {
        let mut cache = AssetCache::new(10);

        assert!(cache.try_get_asset::<TestAsset>(&123).is_none());
    }

    #[test]
    fn updates_last_access_when_retrieved() {
        let mut cache = AssetCache::new(10);

        let asset = Arc::new(TestAsset { value: 5 });

        cache.try_set_asset(&1, asset);

        let first = cache.try_get_last_access(&1).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(5));

        cache.try_get_asset::<TestAsset>(&1);

        let second = cache.try_get_last_access(&1).unwrap();

        assert!(second > first);
    }

    #[test]
    fn evicts_oldest_asset_when_cache_exceeds_capacity() {
        let mut cache = AssetCache::new(2);

        cache.try_set_asset(&1, Arc::new(TestAsset { value: 1 }));
        std::thread::sleep(std::time::Duration::from_millis(2));

        cache.try_set_asset(&2, Arc::new(TestAsset { value: 2 }));
        std::thread::sleep(std::time::Duration::from_millis(2));

        cache.try_set_asset(&3, Arc::new(TestAsset { value: 3 }));

        // This SHOULD be evicted.
        assert!(cache.try_get_asset::<TestAsset>(&1).is_none());

        assert!(cache.try_get_asset::<TestAsset>(&2).is_some());
        assert!(cache.try_get_asset::<TestAsset>(&3).is_some());
    }
}
