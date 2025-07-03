use serde::{Deserialize, Serialize};
use wgpu::{BindGroup, BindGroupLayout, Device, ShaderModule};

use crate::IO::{texture_asset::Texture_asset, AssetLoader::AssetLoader};

//data
#[derive(Clone)]
pub struct Material {
    pub shader: ShaderModule,
    shader_desc: ShaderDesc,
    textures: Vec<Option<Texture_asset>>,
    none: Texture_asset,
}

// construction
impl Material {
    fn initialize_vec_lengths(&mut self) {
        for _ in &self.shader_desc.textures {
            self.textures.push(None);
        }
    }
    pub fn new(shader_desc: ShaderDesc, device: &Device) -> Material {
        let shader = AssetLoader::load_shader_module(device, &shader_desc.shader_module_path);

        let mut m = Material {
            shader_desc: shader_desc,
            textures: Vec::new(),
            none: Texture_asset::none(device),
            shader: shader,
        };
        m.initialize_vec_lengths();
        m
    }
    pub fn set_texture_with_index(&mut self, texture: Option<Texture_asset>, index: usize) {
        self.textures[index] = texture;
    }
    pub fn set_texture_with_label(&mut self, texture: Option<Texture_asset>, label: &str) {
        for i in 0..self.shader_desc.textures.len() {
            let is_same = self.shader_desc.textures[i].label == label;
            if !is_same {
                continue;
            };

            self.textures[i] = texture;
            return;
        }
    }
    pub fn get_binding_group<'a>(&self, device: &Device) -> (BindGroup, BindGroupLayout) {
        // create entries
        let mut i = 0;
        let mut entries: Vec<wgpu::BindGroupEntry> = Vec::new();
        for t in &self.textures {
            let texture: &Texture_asset;
            match t {
                Some(x) => {
                    texture = x;
                }
                None => {
                    texture = &self.none;
                }
            }

            entries.push(wgpu::BindGroupEntry {
                binding: (i * 2),
                resource: wgpu::BindingResource::TextureView(&texture.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: (i * 2) + 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            });
            i = i + 1;
        }
        // create layout
        let mut i = 0;
        let mut layouts: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();
        for t in &self.textures {
            layouts.push(wgpu::BindGroupLayoutEntry {
                binding: (i * 2),
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            });
            layouts.push(wgpu::BindGroupLayoutEntry {
                binding: (i * 2) + 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
            i = i + 1;
        }

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { entries: &layouts[..], label: None });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &entries[..],
            label: None,
        });

        (diffuse_bind_group, texture_bind_group_layout)
    }
}
// public
impl Material {}
// private
impl Material {}
// asset

#[derive(Clone, Serialize, Deserialize)]
pub struct ShaderDesc {
    textures: Vec<ShaderTextureDesc>,
    shader_module_path: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct ShaderTextureDesc {
    label: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ShaderColorDesc {
    label: String,
    index: i32,
}
pub struct ShaderVec1Desc {}
pub struct ShaderVec2Desc {}
pub struct ShaderVec3Desc {}
pub struct ShaderVec4Desc {}
