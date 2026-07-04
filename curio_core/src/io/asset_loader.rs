use crate::io::asset_cache::AssetCache;
use crate::io::asset_database::AssetDatabase;
use crate::io::asset_database::AssetDatabaseListing;
use crate::services;
use crate::AssetCommon;
use crate::Curio;
use crate::EngineServices;
use crate::Severity;
use core::panic;
use std::sync::Arc;
use std::sync::Mutex;

// static mut ASSET_DATABASE: Option<Mutex<AssetDatabase>> = None;
// static mut ASSET_CACHE: Option<Mutex<AssetCache>> = None;

// Built in Shaders
pub static ASSET_UID_SHADER_UNLIT: i16 = -100;
pub static ASSET_UID_SHADER_LIT: i16 = -101;

// Built in Textures
pub static ASSET_UID_TEXTURE_FONT_ATLAS: i16 = -300;

// Font Asset
pub static ASSET_UID_FONT_ASSET_DEFAULT: i16 = -400;

pub static ASSET_UID_TEXTURE_DEFAULT: i16 = -500;

pub struct AssetLoader {
    asset_cache: AssetCache,
    asset_database: AssetDatabase,
}
// private
impl AssetLoader {
    pub fn new(cache: AssetCache, database: AssetDatabase) -> AssetLoader {
        let mut database = database;
        database.append(vec![
            ("shader_lit".to_string(), ASSET_UID_SHADER_LIT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/shader/lit.shader").to_vec())),
            ("shader_unlit".to_string(), ASSET_UID_SHADER_UNLIT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/shader/unlit.shader").to_vec())),
            ("default_texture".to_string(), ASSET_UID_TEXTURE_DEFAULT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/texture/default.png").to_vec())),
        ]);
        AssetLoader { asset_cache: cache, asset_database: database }
    }
    /// Load an asset. Will first check AssetCache for prexisting instance. If none are found will create one by pulling bits from AssetDatabase and loading into T.
    pub fn load_asset<T>(&mut self, uid: &i16) -> Arc<T>
    where
        T: AssetCommon<T>,
    {
        println!("loading {}", uid);
        unsafe {
            {
                // let Some(asset_cache_mutex) = &ASSET_CACHE else {
                //     panic!("ASSET_CACHE not initialized");
                // };
                // let mut asset_cache = asset_cache_mutex
                //     .lock()
                //     .expect("Failed to lock asset cache");
                println!("here 00");

                if let Some(cached_asset) = self.asset_cache.try_get_asset::<T>(uid) {
                    return cached_asset;
                }
            }

            println!("getting for id '{}'", uid);

            let data = {
                // let Some(asset_database_mutex) = &ASSET_DATABASE else {
                //     panic!("ASSET_DATABASE not initialized");
                // };
                // let asset_database = asset_database_mutex
                //     .lock()
                //     .expect("Failed to lock asset database");
                println!("here 0");
                let bytes = self.asset_database.fetch_asset(uid);

                if bytes.is_empty() {
                    panic!("No data for {}!", uid);
                }

                bytes
            };

            let asset = Arc::new(T::from_bits(&data));

            {
                // let Some(asset_cache_mutex) = &ASSET_CACHE else {
                //     panic!("ASSET_CACHE not initialized");
                // };
                // let mut asset_cache = asset_cache_mutex
                //     .lock()
                //     .expect("Failed to lock asset cache");
                println!("here 2");
                // Double-check in case another thread loaded it
                if let Some(existing) = self.asset_cache.try_get_asset::<T>(uid) {
                    return existing;
                }

                self.asset_cache.try_set_asset(uid, asset.clone());
            }

            Curio::log(Severity::Info, &format!("Caching new asset for UID: {}", uid));
            Curio::log(Severity::Info, &format!("Completed lookup: {}", uid));

            asset
        }
    }

    /// Try to find the key based on the name.
    pub fn try_lookup_key_for_name(&self, name: &str) -> Option<i16> {
        //     unsafe {
        //         // let Some(asset_database) = &ASSET_DATABASE else {
        //         //     panic!();
        //         // };
        //         // let Ok(asset_database) = as
        //     let Some(key) = AssetLoader::try_lookup_key_for_name(&name) else {
        //         panic!();
        //     };set_database.lock() else {
        //         //     panic!();
        //         // };

        //         self.asset_database.try_lookup_key_for_name(name)
        //     }
        // }
        None
    }
    pub fn preload_remote_assets(&mut self, force: bool) {
        unsafe {
            // let Some(asset_database) = &ASSET_DATABASE else {
            //     panic!();
            // };
            // let Ok(mut asset_database) = asset_database.lock() else {
            //     panic!();
            // };

            // preload
            self.asset_database.preload_remote_assets(force);
        }
    }

    // set database
    pub fn set_database(database: AssetDatabase) {
        let mut database = database;
        database.append(vec![
            // shaders
            ("shader_lit".to_string(), ASSET_UID_SHADER_LIT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/shader/lit.shader").to_vec())),
            ("shader_unlit".to_string(), ASSET_UID_SHADER_UNLIT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/shader/unlit.shader").to_vec())),
            // shader modules
            // ("shader_module_lit".to_string(), ASSET_UID_SHADER_MODULE_LIT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/shader_module/lit.wgsl").to_vec())),
            // ("shader_module_unlit".to_string(), ASSET_UID_SHADER_MODULE_UNLIT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/shader_module/unlit.wgsl").to_vec())),
            // textures
            // ("default_texture_font_atlas".to_string(), ASSET_UID_TEXTURE_FONT_ATLAS, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/texture/font_black.png").to_vec())),
            // font
            // ("default_font_asset".to_string(), ASSET_UID_FONT_ASSET_DEFAULT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/font/default.font").to_vec())),
            // texutre
            ("default_texture".to_string(), ASSET_UID_TEXTURE_DEFAULT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/texture/default.png").to_vec())),
        ]);

        // unsafe {
        //     ASSET_DATABASE = Some(Mutex::new(database));
        // }
    }
    pub fn set_cache(&self, cache: AssetCache) {
        // unsafe {
        //     ASSET_CACHE = Some(Mutex::new(cache));
        // }
    }

    // // load
    // pub fn load_shader_module(device: &Device, path: &i16) -> Arc<ShaderModule> {
    //     unsafe {
    //         return SystemGPU::get_shader_module(path);
    //     }
    // }
}

pub struct BuiltInAssets {}

impl BuiltInAssets {}

pub struct Assets {}
impl Assets {
    pub fn load_asset<T>(uid: &i16) -> Arc<T>
    where
        T: AssetCommon<T>,
    {
        services().assets().load_asset::<T>(uid)
    }
    pub fn try_lookup_key_for_name(name: &str) -> Option<i16> {
        None
    }
    pub fn append(listings: Vec<(String, i16, AssetDatabaseListing)>) {
        services().assets().asset_database.append(listings);
    }
}
