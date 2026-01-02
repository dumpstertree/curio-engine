use std::{collections::HashMap, sync::Arc, time::Instant};

use egui_wgpu::wgpu::ShaderModule;

use crate::{
    collections::material::ShaderDesc,
    io::{asset_loader::FontAsset, model_asset::ModelAsset, texture_asset::TextureAsset},
};

pub struct AssetCache {
    max_cache_len: usize,
    cache_model: HashMap<String, (Instant, Arc<ModelAsset>)>,
    cache_texture: HashMap<String, (Instant, Arc<TextureAsset>)>,
    cache_shader_desc: HashMap<String, (Instant, Arc<ShaderDesc>)>,
    cache_shader_module: HashMap<String, (Instant, Arc<ShaderModule>)>,
    cache_font_asset: HashMap<String, (Instant, Arc<FontAsset>)>,
}
impl AssetCache {
    pub fn new() -> AssetCache {
        AssetCache {
            max_cache_len: 100,
            cache_model: HashMap::new(),
            cache_texture: HashMap::new(),
            cache_shader_desc: HashMap::new(),
            cache_shader_module: HashMap::new(),
            cache_font_asset: HashMap::new(),
        }
    }
    pub fn try_get_asset_font_asset(&mut self, id: &str) -> Option<Arc<FontAsset>> {
        // check if we have an entry for this id - if not return none
        let Some(entry) = self.cache_font_asset.get_mut(id) else {
            println!("failed to get asset");
            return None;
        };

        // update the time
        entry.0 = Instant::now();

        // we have an entry. take the asset and clone it and return it
        return Some(entry.1.clone());
    }
    /// Tries to get the asset from the cache. If it is not present it will return None
    pub fn try_get_asset_model(&mut self, id: &str) -> Option<Arc<ModelAsset>> {
        // check if we have an entry for this id - if not return none
        let Some(entry) = self.cache_model.get_mut(id) else {
            println!("failed to get asset");
            return None;
        };

        // update the time
        entry.0 = Instant::now();

        // we have an entry. take the asset and clone it and return it
        return Some(entry.1.clone());
    }
    /// Tries to get the asset from the cache. If it is not present it will return None
    pub fn try_get_asset_texture(&mut self, id: &str) -> Option<Arc<TextureAsset>> {
        // check if we have an entry for this id - if not return none
        let Some(entry) = self.cache_texture.get_mut(id) else {
            println!("failed to get asset");
            return None;
        };

        // update the time
        entry.0 = Instant::now();

        // we have an entry. take the asset and clone it and return it
        return Some(entry.1.clone());
    }
    /// Tries to get the asset from the cache. If it is not present it will return None
    pub fn try_get_asset_shader_desc(&mut self, id: &str) -> Option<Arc<ShaderDesc>> {
        // check if we have an entry for this id - if not return none
        let Some(entry) = self.cache_shader_desc.get_mut(id) else {
            println!("failed to get asset");
            return None;
        };

        // update the time
        entry.0 = Instant::now();

        // we have an entry. take the asset and clone it and return it
        return Some(entry.1.clone());
    }
    /// Tries to get the asset from the cache. If it is not present it will return None
    pub fn try_get_asset_shader_module(&mut self, id: &str) -> Option<Arc<ShaderModule>> {
        // check if we have an entry for this id - if not return none
        let Some(entry) = self.cache_shader_module.get_mut(id) else {
            println!("failed to get asset");
            return None;
        };

        // update the time
        entry.0 = Instant::now();

        // we have an entry. take the asset and clone it and return it
        return Some(entry.1.clone());
    }

    /// Tries to store the asset. If it already exists it fails
    pub fn try_store_asset_model(&mut self, id: &str, asset: Arc<ModelAsset>) {
        // make sure the asset isnt already contained
        if self.cache_model.contains_key(id) {
            println!(" already contains asset");
            return;
        }
        // adds the asset storing the time it was added
        self.cache_model
            .insert(id.to_string(), (Instant::now(), asset));

        // trim to max
        self.trim();
    }
    /// Tries to store the asset. If it already exists it fails
    pub fn try_store_asset_texture(&mut self, id: &str, asset: Arc<TextureAsset>) {
        // make sure the asset isnt already contained
        if self.cache_texture.contains_key(id) {
            println!(" already contains asset");
            return;
        }
        // adds the asset storing the time it was added
        self.cache_texture
            .insert(id.to_string(), (Instant::now(), asset));

        // trim to max
        self.trim();
    }
    /// Tries to store the asset. If it already exists it fails
    pub fn try_store_asset_shader_desc(&mut self, id: &str, asset: Arc<ShaderDesc>) {
        // make sure the asset isnt already contained
        if self.cache_shader_desc.contains_key(id) {
            println!(" already contains asset");
            return;
        }
        // adds the asset storing the time it was added
        self.cache_shader_desc
            .insert(id.to_string(), (Instant::now(), asset));

        // trim to max
        self.trim();
    }
    /// Tries to store the asset. If it already exists it fails
    pub fn try_store_asset_shader_module(&mut self, id: &str, asset: Arc<ShaderModule>) {
        // make sure the asset isnt already contained
        if self.cache_shader_module.contains_key(id) {
            println!(" already contains asset");
            return;
        }
        // adds the asset storing the time it was added
        self.cache_shader_module
            .insert(id.to_string(), (Instant::now(), asset));

        // trim to max
        self.trim();
    }
    /// Tries to store the asset. If it already exists it fails
    pub fn try_store_asset_font_asset(&mut self, id: &str, asset: Arc<FontAsset>) {
        // make sure the asset isnt already contained
        if self.cache_font_asset.contains_key(id) {
            println!(" already contains asset");
            return;
        }
        // adds the asset storing the time it was added
        self.cache_font_asset
            .insert(id.to_string(), (Instant::now(), asset));

        // trim to max
        self.trim();
    }

    // Clear all cached assets
    pub fn clear(&mut self) {
        // clear all
        self.cache_model.clear();
    }

    fn trim(&mut self) {
        return;
        // remove old values to meet max
        while self.cache_model.len() > self.max_cache_len {
            let mut oldest_id = None;
            let mut oldest_time = None;
            for (id, (x, y)) in &self.cache_model {
                // values are none add them
                if oldest_id.is_none() || oldest_time.is_none() {
                    oldest_id = Some(id.clone());
                    oldest_time = Some(x);
                    continue;
                } else {
                    //
                    let Some(was_oldest_time) = oldest_time else {
                        continue;
                    };

                    if x.elapsed() > was_oldest_time.elapsed() {
                        oldest_id = Some(id.clone());
                        oldest_time = Some(x);
                    }
                }
            }
            if let Some(oldest_id) = oldest_id {
                self.cache_model.remove(&oldest_id);
                println!("trimmed");
            }
        }
    }
}
