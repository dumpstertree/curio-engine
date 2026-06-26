use crate::assets::asset::AssetCommonFromBits;
use crate::io::asset_cache::AssetCache;
use crate::io::asset_database::AssetDatabase;
use crate::io::asset_database::AssetDatabaseListing;
use crate::Curio;
use crate::Severity;
use core::panic;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;

static ASSET_DATABASE: LazyLock<Mutex<AssetDatabase>> = LazyLock::new(|| Mutex::new(AssetDatabase::new_from_explicit(vec![])));
static ASSET_CACHE: LazyLock<Mutex<AssetCache>> = LazyLock::new(|| Mutex::new(AssetCache::new(100)));

// Built in Shaders
pub static ASSET_UID_SHADER_UNLIT: i16 = -100;
pub static ASSET_UID_SHADER_LIT: i16 = -101;

// Built in Textures
pub static ASSET_UID_TEXTURE_FONT_ATLAS: i16 = -300;

// Font Asset
pub static ASSET_UID_FONT_ASSET_DEFAULT: i16 = -400;

pub static ASSET_UID_TEXTURE_DEFAULT: i16 = -500;

pub struct AssetLoader {}
// private
impl AssetLoader {
    /// Load an asset. Will first check AssetCache for prexisting instance. If none are found will create one by pulling bits from AssetDatabase and loading into T.
    pub fn load_asset<T>(uid: &i16) -> Arc<T>
    where
        T: AssetCommonFromBits<T>,
    {
        let Ok(mut cache) = ASSET_CACHE.lock() else {
            panic!();
        };

        {
            if let Some(cached_asset) = cache.try_get_asset::<T>(uid) {
                return cached_asset;
            }
        }

        let data = {
            let Ok(asset_database_mutex) = ASSET_DATABASE.lock() else {
                panic!();
            };

            let bytes = asset_database_mutex.fetch_asset(uid);

            if bytes.is_empty() {
                panic!("No data for {}!", uid);
            }

            bytes
        };

        let asset = Arc::new(T::from_bits(&data));

        {
            // Double-check in case another thread loaded it
            if let Some(existing) = cache.try_get_asset::<T>(uid) {
                return existing;
            }

            cache.try_set_asset(uid, asset.clone());
        }

        Curio::log(Severity::Info, &format!("Caching new asset for UID: {}", uid));
        Curio::log(Severity::Info, &format!("Completed lookup: {}", uid));

        asset
    }

    /// Try to find the key based on the name.
    pub fn try_lookup_key_for_name(name: &str) -> Option<i16> {
        let Ok(asset_database) = ASSET_DATABASE.lock() else {
            panic!();
        };

        asset_database.try_lookup_key_for_name(name)
    }
    pub fn preload_remote_assets(force: bool) {
        let Ok(asset_database) = ASSET_DATABASE.lock() else {
            panic!();
        };
        // preload
        asset_database.preload_remote_assets(force);
    }

    // set database
    pub fn set_database(database: AssetDatabase) {
        let Ok(mut global_database) = ASSET_DATABASE.lock() else {
            panic!();
        };

        global_database.listings = database.listings;
        global_database.lookup = database.lookup;

        global_database.append(vec![
            // shaders
            ("shader_lit".to_string(), ASSET_UID_SHADER_LIT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/shader/lit.shader").to_vec())),
            ("shader_unlit".to_string(), ASSET_UID_SHADER_UNLIT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/shader/unlit.shader").to_vec())),
            // textures
            ("default_texture_font_atlas".to_string(), ASSET_UID_TEXTURE_FONT_ATLAS, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/texture/font_black.png").to_vec())),
            // font
            ("default_font_asset".to_string(), ASSET_UID_FONT_ASSET_DEFAULT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/font/default.font").to_vec())),
            // texutre
            ("default_texture".to_string(), ASSET_UID_TEXTURE_DEFAULT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/texture/default.png").to_vec())),
        ]);
    }
    pub fn set_cache(cache: AssetCache) {
        let Ok(mut global_cache) = ASSET_CACHE.lock() else {
            panic!();
        };

        global_cache.max_cache_len = cache.max_cache_len;
    }
}

pub struct BuiltInAssets {}

impl BuiltInAssets {}
