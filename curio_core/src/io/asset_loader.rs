use crate::assets::asset::AssetCommonFromBits;
use crate::io::asset_cache::AssetCache;
use crate::io::asset_database::AssetDatabase;
use crate::io::asset_database::AssetDatabaseListing;
use crate::Application;
use crate::Severity;
use core::panic;
use std::sync::Arc;
use std::sync::Mutex;

static mut ASSET_DATABASE: Option<Mutex<AssetDatabase>> = None;
static mut ASSET_CACHE: Option<Mutex<AssetCache>> = None;

// Built in Shaders
pub static ASSET_UID_SHADER_UNLIT: i16 = -100;
pub static ASSET_UID_SHADER_LIT: i16 = -101;

// Built in Shaders
pub static ASSET_UID_SHADER_MODULE_UNLIT: i16 = -200;
pub static ASSET_UID_SHADER_MODULE_LIT: i16 = -201;

// Built in Textures
pub static ASSET_UID_TEXTURE_FONT_ATLAS: i16 = -300;

// Font Asset
pub static ASSET_UID_FONT_ASSET_DEFAULT: i16 = -400;
pub struct AssetLoader {}
// private
impl AssetLoader {
    /// Load an asset. Will first check AssetCache for prexisting instance. If none are found will create one by pulling bits from AssetDatabase and loading into T.
    pub fn load_asset<T>(uid: &i16) -> Arc<T>
    where
        T: AssetCommonFromBits<T>,
    {
        unsafe {
            let Some(asset_cache_mutex) = &ASSET_CACHE else {
                panic!("ASSET_CACHE not initialized");
            };
            let Some(asset_database_mutex) = &ASSET_DATABASE else {
                panic!("ASSET_DATABASE not initialized");
            };
            {
                let mut asset_cache = asset_cache_mutex
                    .lock()
                    .expect("Failed to lock asset cache");

                if let Some(cached_asset) = asset_cache.try_get_asset::<T>(uid) {
                    return cached_asset;
                }
            }

            let data = {
                let asset_database = asset_database_mutex
                    .lock()
                    .expect("Failed to lock asset database");

                let bytes = asset_database.fetch_asset(uid);

                if bytes.is_empty() {
                    panic!("No data for {}!", uid);
                }

                bytes
            };

            let asset = Arc::new(T::from_bits(&data));

            {
                let mut asset_cache = asset_cache_mutex
                    .lock()
                    .expect("Failed to lock asset cache");

                // Double-check in case another thread loaded it
                if let Some(existing) = asset_cache.try_get_asset::<T>(uid) {
                    return existing;
                }

                asset_cache.try_set_asset(uid, asset.clone());
            }

            Application::log(Severity::Info, &format!("Caching new asset for UID: {}", uid));

            asset
        }
    }

    /// Try to find the key based on the name.
    pub fn try_lookup_key_for_name(name: &str) -> Option<i16> {
        unsafe {
            let Some(asset_database) = &ASSET_DATABASE else {
                panic!();
            };
            let Ok(asset_database) = asset_database.lock() else {
                panic!();
            };

            asset_database.try_lookup_key_for_name(name)
        }
    }
    pub fn preload_remote_assets(force: bool) {
        unsafe {
            let Some(asset_database) = &ASSET_DATABASE else {
                panic!();
            };
            let Ok(mut asset_database) = asset_database.lock() else {
                panic!();
            };

            // preload
            asset_database.preload_remote_assets(force);
        }
    }

    // set database
    pub fn set_database(database: AssetDatabase) {
        let mut database = database;
        database.append(vec![
            // shaders
            ("shader_lit".to_string(), ASSET_UID_SHADER_LIT, AssetDatabaseListing::Local("built_in/shader/lit.shader".to_string())),
            ("shader_unlit".to_string(), ASSET_UID_SHADER_UNLIT, AssetDatabaseListing::Local("built_in/shader/unlit.shader".to_string())),
            // shader modules
            ("shader_module_lit".to_string(), ASSET_UID_SHADER_MODULE_LIT, AssetDatabaseListing::Local("built_in/shader_module/lit.wgsl".to_string())),
            ("shader_module_unlit".to_string(), ASSET_UID_SHADER_MODULE_UNLIT, AssetDatabaseListing::Local("built_in/shader_module/unlit.wgsl".to_string())),
            // textures
            ("default_texture_font_atlas".to_string(), ASSET_UID_TEXTURE_FONT_ATLAS, AssetDatabaseListing::Local("built_in/texture/font_black.png".to_string())),
            // font
            ("default_font_asset".to_string(), ASSET_UID_FONT_ASSET_DEFAULT, AssetDatabaseListing::Local("built_in/font/default.font".to_string())),
        ]);
        unsafe {
            ASSET_DATABASE = Some(Mutex::new(database));
        }
    }
    pub fn set_cache(cache: AssetCache) {
        unsafe {
            ASSET_CACHE = Some(Mutex::new(cache));
        }
    }

    // // load
    // pub fn load_shader_module(device: &Device, path: &i16) -> Arc<ShaderModule> {
    //     unsafe {
    //         return SystemGPU::get_shader_module(path);
    //     }
    // }
}
