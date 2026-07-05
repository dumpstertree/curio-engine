use crate::AssetDatabaseListing;
use std::collections::HashMap;

pub struct AssetDatabase {
    lookup: HashMap<String, i16>,
    listings: HashMap<i16, AssetDatabaseListing>,
}
impl AssetDatabase {
    /// Create a new AssetDatabase
    pub fn new() -> AssetDatabase {
        AssetDatabase { lookup: HashMap::new(), listings: HashMap::new() }
    }
    /// Add a single listing to be referenced during a fetch
    pub fn append_listing(&mut self, name: String, uid: i16, listing: AssetDatabaseListing) {
        self.lookup.insert(name, uid);
        self.listings.insert(uid, listing);
    }
    /// Add a group of listings to be referenced during a fetch
    pub fn append_listing_group(&mut self, listings: Vec<(String, i16, AssetDatabaseListing)>) {
        for x in &listings {
            self.lookup.insert(x.0.clone(), x.1);
        }
        for x in listings {
            self.listings.insert(x.1, x.2);
        }
    }

    /// Use the name to get a key. Will return None if name is not present
    pub fn try_lookup_key_for_name(&self, name: &str) -> Option<i16> {
        self.lookup.get(name).cloned()
    }
    /// Fetch an asset for the `uid`. If uid is not mapped returns and empty Vec<u8>
    pub fn fetch_asset(&self, uid: &i16) -> Vec<u8> {
        if let Some(listing) = self.listings.get(&uid) {
            return listing.fetch_asset(false);
        } else {
            println!("Database does not contain asset with UID '{}'", uid);
        }
        return vec![];
    }
    /// Cache all remote assets locally for faster access later
    pub fn preload_remote_assets(&self, force: bool) {
        for listing in &self.listings {
            match listing.1 {
                AssetDatabaseListing::RemoteToCache(_, _) => {
                    _ = listing.1.fetch_asset(force);
                }
                _ => {}
            }
        }
    }
}
