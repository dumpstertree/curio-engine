#[cfg(test)]
mod tests {
    use curio_core::{AssetCache, AssetDatabase, AssetDatabaseListing, AssetPipeline, Composition};

    const COMPOSITION: &str = "enabled: true\nbase: ''\nname: card\ncomponents: []\nchildren: []\n";

    fn pipeline() -> AssetPipeline {
        let mut database = AssetDatabase::new();
        database.append_listing("card".to_string(), 7, AssetDatabaseListing::Embedded(COMPOSITION.as_bytes().to_vec()));
        database.append_listing("empty".to_string(), 8, AssetDatabaseListing::Embedded(Vec::new()));
        AssetPipeline::new(AssetCache::new(10), database)
    }

    #[test]
    fn try_load_asset_loads_a_registered_asset_and_caches_it() {
        let mut assets = pipeline();
        let first = assets
            .try_load_asset::<Composition>(&7)
            .expect("a registered asset loads");
        assert_eq!(first.name, "card");
        let again = assets.try_load_asset::<Composition>(&7).unwrap();
        assert!(std::sync::Arc::ptr_eq(&first, &again), "the second load is not the cached asset");
    }

    #[test]
    fn try_load_asset_is_none_for_an_unregistered_or_empty_asset() {
        let mut assets = pipeline();
        assert!(assets.try_load_asset::<Composition>(&99).is_none());
        assert!(assets.try_load_asset::<Composition>(&8).is_none());
    }
}
