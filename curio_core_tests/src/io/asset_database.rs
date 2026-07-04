#[cfg(test)]
mod tests {
    use curio_core::io::asset_database::{AssetDatabase, AssetDatabaseListing};

    fn listing(bytes: &[u8]) -> AssetDatabaseListing {
        AssetDatabaseListing::Embedded(bytes.to_vec())
    }

    #[test]
    fn new_from_explicit_builds_lookup_and_listing_maps() {
        let db = AssetDatabase::new_from_explicit(vec![("player".to_string(), 1, listing(&[1, 2, 3])), ("enemy".to_string(), 2, listing(&[4, 5, 6]))]);

        assert_eq!(db.try_lookup_key_for_name("player"), Some(1));
        assert_eq!(db.try_lookup_key_for_name("enemy"), Some(2));
        assert_eq!(db.try_lookup_key_for_name("missing"), None);

        assert_eq!(db.fetch_asset(&1), vec![1, 2, 3]);
        assert_eq!(db.fetch_asset(&2), vec![4, 5, 6]);
    }

    #[test]
    fn append_adds_new_assets() {
        let mut db = AssetDatabase::new_from_explicit(vec![("player".to_string(), 1, listing(&[1]))]);

        db.append(vec![("enemy".to_string(), 2, listing(&[2])), ("tree".to_string(), 3, listing(&[3]))]);

        assert_eq!(db.try_lookup_key_for_name("player"), Some(1));
        assert_eq!(db.try_lookup_key_for_name("enemy"), Some(2));
        assert_eq!(db.try_lookup_key_for_name("tree"), Some(3));

        assert_eq!(db.fetch_asset(&1), vec![1]);
        assert_eq!(db.fetch_asset(&2), vec![2]);
        assert_eq!(db.fetch_asset(&3), vec![3]);
    }

    #[test]
    fn append_overwrites_existing_entries() {
        let mut db = AssetDatabase::new_from_explicit(vec![("player".to_string(), 1, listing(&[1]))]);

        db.append(vec![("player".to_string(), 10, listing(&[9, 9, 9]))]);

        assert_eq!(db.try_lookup_key_for_name("player"), Some(10));
        assert_eq!(db.fetch_asset(&10), vec![9, 9, 9]);

        // Old UID should no longer exist.
        assert_eq!(db.fetch_asset(&1), Vec::<u8>::new());
    }

    #[test]
    fn fetch_asset_returns_empty_when_uid_missing() {
        let db = AssetDatabase::new_from_explicit(vec![]);

        assert!(db.fetch_asset(&999).is_empty());
    }

    #[test]
    fn embedded_listing_returns_original_bytes() {
        let bytes = vec![10, 20, 30, 40];

        let listing = AssetDatabaseListing::Embedded(bytes.clone());

        assert_eq!(listing.fetch_asset(false), bytes);
    }

    #[test]
    fn lookup_returns_none_for_unknown_name() {
        let db = AssetDatabase::new_from_explicit(vec![("player".to_string(), 1, listing(&[1]))]);

        assert_eq!(db.try_lookup_key_for_name("enemy"), None);
    }
}
