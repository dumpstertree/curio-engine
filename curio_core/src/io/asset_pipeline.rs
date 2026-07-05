use crate::io::asset_cache::AssetCache;
use crate::io::asset_database::AssetDatabase;
use crate::AssetCommon;
use crate::AssetDatabaseListing;
use core::panic;
use std::sync::Arc;

// Built in Shaders
pub static ASSET_UID_SHADER_UNLIT: i16 = -100;
pub static ASSET_UID_SHADER_LIT: i16 = -101;

// Built in Textures
pub static ASSET_UID_TEXTURE_FONT_ATLAS: i16 = -300;

// Font Asset
pub static ASSET_UID_FONT_ASSET_DEFAULT: i16 = -400;

pub static ASSET_UID_TEXTURE_DEFAULT: i16 = -500;

pub struct AssetPipeline {
    pub(crate) asset_cache: AssetCache,
    pub(crate) asset_database: AssetDatabase,
}
// private
impl AssetPipeline {
    /// Create a new AssetPipeline. This will Automatically embed all built in assets
    pub fn new(cache: AssetCache, database: AssetDatabase) -> AssetPipeline {
        // create our new db
        let mut database = database;

        // add all built in assets
        database.append_listing_group(vec![
            ("shader_lit".to_string(), ASSET_UID_SHADER_LIT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/shader/lit.shader").to_vec())),
            ("shader_unlit".to_string(), ASSET_UID_SHADER_UNLIT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/shader/unlit.shader").to_vec())),
            ("default_texture".to_string(), ASSET_UID_TEXTURE_DEFAULT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/texture/default.png").to_vec())),
            ("default_texture_font_atlas".to_string(), ASSET_UID_TEXTURE_FONT_ATLAS, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/texture/font_black.png").to_vec())),
            ("default_font_asset".to_string(), ASSET_UID_FONT_ASSET_DEFAULT, AssetDatabaseListing::Embedded(include_bytes!("../../../assets/built_in/font/default.font").to_vec())),
        ]);
        // return the instance
        AssetPipeline { asset_cache: cache, asset_database: database }
    }
    /// Load an asset. Will first check AssetCache for prexisting instance. If none are found will create one by pulling bits from AssetDatabase and loading into T.
    pub fn load_asset<T>(&mut self, uid: &i16) -> Arc<T>
    where
        T: AssetCommon<T>,
    {
        // try to get the cached asset
        if let Some(cached_asset) = self.asset_cache.try_get_asset::<T>(uid) {
            return cached_asset;
        }

        // was not in cache so pull the data from the listing. If we are given no bytes back we panic.
        let data = {
            let bytes = self.asset_database.fetch_asset(uid);
            if bytes.is_empty() {
                panic!("No data for {}!", uid);
            }
            bytes
        };

        // cast the asset and wrap it in an Arc
        let asset = Arc::new(T::from_bits(&data));

        // save the asset to the cache
        self.asset_cache.try_set_asset(uid, asset.clone());

        // return the asset
        asset
    }

    /// Try to find the key based on the name.
    pub fn try_lookup_key_for_name(&self, name: &str) -> Option<i16> {
        self.asset_database.try_lookup_key_for_name(name)
    }

    /// Load any remote assets in the Database and save them to the cache.
    pub fn preload_remote_assets(&mut self, force: bool) {
        self.asset_database.preload_remote_assets(force);
    }
}
