use crate::{io::file::File, CacheMetadata, Curio, Severity};
use chrono::DateTime;
use std::{error::Error, time::SystemTime};

const STALE_HOURS: f32 = 1.0;
const STALE_MIN: f32 = 0.0;
const STALE_SEC: f32 = 0.0;

pub enum AssetDatabaseListing {
    /// Embed data in application and reuse it later
    Embedded(Vec<u8>),
    /// Retrieve data from a local path on computer
    Local(String),
    /// Retrieve data from a remote path and then cache it to local path on computer
    RemoteToCache(String, String),
}
// imp -> Pub Fns
impl AssetDatabaseListing {
    /// Fetch an asset returning the data. Will return an empty Vec<u8> if there was a problem
    pub fn fetch_asset(&self, force: bool) -> Vec<u8> {
        match self {
            AssetDatabaseListing::Embedded(embedded) => embedded.clone(),
            AssetDatabaseListing::Local(local_path) => Self::fetch_asset_local(local_path),
            AssetDatabaseListing::RemoteToCache(local_path, remote_path) => Self::fetch_asset_remote(local_path, remote_path, force),
        }
    }
}
// imp -> Private Fns
impl AssetDatabaseListing {
    fn fetch_asset_local(local_path: &str) -> Vec<u8> {
        Curio::log(Severity::Info, &format!("Performed LOCAL fetch at : {}", (&File::join_path(&File::get_built_in_asset_path(), &local_path))));
        // pull asset from local path
        return File::read(&File::join_path(&File::get_built_in_asset_path(), &local_path));
    }
    fn fetch_asset_remote(local_path: &str, remote_path: &String, force: bool) -> Vec<u8> {
        // log
        Curio::log(Severity::Info, &format!("Performed REMOTE fetch at : {}", (&File::join_path(&File::get_built_in_asset_path(), &remote_path))));
        // instance data
        let active_instance: Vec<i32> = vec![];
        // create paths
        let cache_path_asset = File::join_path(&File::get_cache_path(), &local_path);
        let cache_path_meta = File::join_path(&File::get_cache_path(), &format!("{}.meta", &local_path));

        // check if this has been loaded by any of the open instances
        let mut is_stale = true;
        let mut cold_load = true;

        // only run if we arent forcing a recalc
        if !force {
            if let Ok(meta) = serde_json::from_slice::<CacheMetadata>(&File::read(&cache_path_meta)) {
                if let Ok(elapsed) = SystemTime::now().duration_since(meta.last_edit) {
                    is_stale = elapsed.as_secs_f32() > (STALE_HOURS * 3600.0) + (STALE_MIN * 60.0) + STALE_SEC;
                }
                if active_instance
                    .iter()
                    .any(|x| meta.edited_by_instance.contains(&x))
                {
                    cold_load = false;
                }
            }
        }
        // if this is a cold load we need to pull the data and store it
        if cold_load && is_stale {
            // write meta file to disk
            if let Ok(meta) = serde_json::to_vec(&CacheMetadata {
                edited_by_instance: active_instance,
                last_edit: SystemTime::now(),
            }) {
                File::write(&cache_path_meta, &meta);
            }
            //gets the meta data
            let last_modified = File::get_meta_modified(&cache_path_asset);
            // make sure we didnt encounter an error checking for dirty
            if let Ok(is_dirty) = Self::fetch_is_remote_asset_dirty(remote_path, last_modified) {
                // was the value considered dirty
                if is_dirty {
                    //make sure the bytes didnt encounter and error pulling
                    if let Ok(bytes) = Self::fetch_remote_asset(remote_path) {
                        // write file to disk
                        File::write(&cache_path_asset, &bytes);
                        // return
                        return bytes;
                    }
                }
            }
        }

        // file was not dirty or something went wrong - attempt to read from disk
        return File::read(&cache_path_asset);
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
