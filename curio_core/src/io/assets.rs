use crate::{AssetCommon, AssetDatabaseListing, Services};
use std::sync::Arc;

//A convience wrapper for Services::get().assets()
pub struct Assets {}
impl Assets {
    /// Load an asset from the set AssetPipeline.
    pub fn load_asset<T>(uid: &i16) -> Arc<T>
    where
        T: AssetCommon<T>,
    {
        Services::get().assets().load_asset::<T>(uid)
    }
    /// Get a UID for name from set AssetPipeline
    pub fn try_lookup_key_for_name(name: &str) -> Option<i16> {
        Services::get().assets().try_lookup_key_for_name(name)
    }
    /// Add a listing to the set AssetPipeline
    pub fn append_listing_group(listings: Vec<(String, i16, AssetDatabaseListing)>) {
        Services::get()
            .assets()
            .asset_database
            .append_listing_group(listings);
    }
    /// Add mulitple listings to the set AssetPipeline
    pub fn append_listing(name: String, uid: i16, listing: AssetDatabaseListing) {
        Services::get()
            .assets()
            .asset_database
            .append_listing(name, uid, listing);
    }
}
