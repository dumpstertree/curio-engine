use crate::io::file::File;
use chrono::DateTime;
use egui::ahash::{HashMap, HashMapExt};
use std::{error::Error, time::SystemTime};

pub struct AssetDatabase {
    listings: HashMap<String, AssetDatabaseListing>,
}
impl AssetDatabase {
    /// Create a new AssetDatabase from explicitly stated connections
    pub fn new_from_explicit(listings: Vec<(String, AssetDatabaseListing)>) -> AssetDatabase {
        let mut hashmap = HashMap::new();
        for x in listings {
            hashmap.insert(x.0, x.1);
        }
        AssetDatabase { listings: hashmap }
    }

    /// Fetch an asset for the `uid`.
    /// If uid is not mapped returns and empty Vec<u8>
    pub fn fetch_asset(&self, uid: String) -> Vec<u8> {
        if let Some(listing) = self.listings.get(&uid) {
            return listing.fetch_asset();
        } else {
            println!("Database does not contain asset with UID '{}'", uid);
        }
        return vec![];
    }
}

pub enum AssetDatabaseListing {
    /// Retrieve data from a local path on computer
    Local(String),
    /// Retrieve data from a remote path and then cache it to local path on computer
    RemoteToCache(String, String),
}
impl AssetDatabaseListing {
    /// Fetch an asset reeturning the data.
    /// Will return an empty Vec<u8> if there was a problem
    pub fn fetch_asset(&self) -> Vec<u8> {
        match self {
            AssetDatabaseListing::Local(local_path) => Self::fetch_asset_local(local_path),
            AssetDatabaseListing::RemoteToCache(local_path, remote_path) => Self::fetch_asset_remote(local_path, remote_path),
        }
    }
    fn fetch_asset_local(local_path: &str) -> Vec<u8> {
        println!("{}", &File::join_path(&File::get_built_in_asset_path(), &local_path));
        // pull asset from local path
        return File::read(&File::join_path(&File::get_built_in_asset_path(), &local_path));
    }
    fn fetch_asset_remote(local_path: &str, remote_path: &String) -> Vec<u8> {
        let cache_path = File::join_path(&&File::get_cache_path(), &local_path);
        //gets the meta data
        let last_modified = File::get_meta_modified(&cache_path);
        // make sure we didnt encounter an error checking for dirty
        if let Ok(is_dirty) = Self::fetch_is_remote_asset_dirty(remote_path, last_modified) {
            // was the value considered dirty
            if is_dirty {
                //make sure the bytes didnt encounter and error pulling
                if let Ok(bytes) = Self::fetch_remote_asset(remote_path) {
                    // write file to disk
                    File::write(&cache_path, &bytes);
                    // return
                    return bytes;
                }
            }
        }

        // file was not dirty or something went wrong - attempt to read from disk
        return File::read(&cache_path);
    }
    fn fetch_remote_asset(url: &String) -> Result<Vec<u8>, Box<dyn Error>> {
        let client = reqwest::blocking::Client::new();
        let resp = client.get(url).send()?;
        let bytes = resp.bytes()?; // blocking download of entire body
        Ok(bytes.to_vec()) // convert Bytes -> Vec<u8>
    }
    fn fetch_is_remote_asset_dirty(url: &String, last_checked: Option<SystemTime>) -> Result<bool, Box<dyn Error>> {
        // If there was no previous check we need to update
        if last_checked.is_none() {
            return Ok(true);
        }

        // Send a HEAD request
        let client = reqwest::blocking::Client::new();

        // if we failed to reach server just use local
        let resp = client.head(url).send().unwrap();
        if !resp.status().is_success() {
            return Ok(false);
        }

        if let Some(last_modified) = resp.headers().get("last-modified") {
            let last_modified_str = last_modified.to_str()?;

            // Parse HTTP date (RFC 2822 / RFC 1123 format)
            let parsed_date = DateTime::parse_from_rfc2822(last_modified_str)?;

            // Convert to SystemTime
            let remote_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(parsed_date.timestamp() as u64);

            // Compare
            Ok(remote_time > last_checked.unwrap())
        } else {
            // Header missing → treat as newer
            Ok(true)
        }
    }
}
