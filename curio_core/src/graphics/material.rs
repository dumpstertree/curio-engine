use core::panic;
use egui_wgpu::wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, ShaderModule};
use serde::{Deserialize, Serialize};
use std::{hash::Hash, sync::Arc};

use crate::{
    assets::asset::AssetCommonFromBits,
    engine_services::services,
    io::asset_loader::{ASSET_UID_SHADER_MODULE_LIT, ASSET_UID_SHADER_MODULE_UNLIT},
    random::Random,
    system_adapters::adapter_system_gpu::get_shader_module,
    AssetCommon, Color, TextureAsset,
};

//=========================================
// Data Types
//=========================================
#[derive(PartialEq)]
pub struct Material {
    name: String,
    pub instance_id: i32,
    // pub shader: Arc<ShaderModule>,
    shader_desc: Arc<ShaderDesc>,
    textures: Vec<Option<Arc<TextureAsset>>>,
    colors: Vec<Color>,
    color_buffers: Vec<Option<Buffer>>,
    none: Arc<TextureAsset>,
    cache: Option<Arc<(BindGroup, BindGroupLayout)>>,
}
impl Eq for Material {}
impl Hash for Material {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.instance_id.hash(state);
    }
}

// A uniform buffer struct for one color
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorUniform {
    color: [f32; 4],
}
impl ColorUniform {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> ColorUniform {
        ColorUniform { color: [r, g, b, a] }
    }
}

//=========================================
// Construction
//=========================================
impl Material {
    pub fn new(name: &str, shader_desc: Arc<ShaderDesc>, finalize: bool) -> Material {
        // let device = &SystemGPU::get_device();
        // let shader = AssetLoader::load_shader_module(device, &shader_desc.shader_module_path);

        let mut m = Material {
            name: name.to_string(),
            instance_id: Random::range_int(-99999999, 9999999),
            shader_desc,
            textures: Vec::new(),
            colors: Vec::new(),
            color_buffers: Vec::new(),
            none: Arc::new(TextureAsset::none()),
            cache: None, // shader,
        };

        m.initialize_vec_lengths();
        m.upload_color_buffer(0);

        if finalize {
            m.finalize();
        }
        m
    }

    pub fn shader(&self) -> Arc<ShaderModule> {
        // let device = &SystemGPU::get_device();
        // AssetLoader::load_shader_module(device, Builtin &self.shader_desc.shader_module_path)

        let s = services();

        if self.shader_desc.shader_module_path == "shader_module_lit" {
            return get_shader_module(&ASSET_UID_SHADER_MODULE_LIT);
        }
        if self.shader_desc.shader_module_path == "shader_module_unlit" {
            return get_shader_module(&ASSET_UID_SHADER_MODULE_UNLIT);
        }
        // SystemGPU::get_shader_module(&self.shader_desc.shader_module_path).unwrap()

        // self.shader_desc.shader_module_path
        panic!("IDK ABOUT THIS");
    }
    pub fn instantiate(&self, name: &str) -> Material {
        Material {
            name: name.to_string(),
            instance_id: Random::range_int(-9999999, 9999999),
            // shader: self.shader.clone(),
            shader_desc: self.shader_desc.clone(),
            textures: self.textures.clone(),
            colors: self.colors.clone(),
            color_buffers: self.color_buffers.clone(),
            none: self.none.clone(),
            cache: None,
        }
    }

    fn initialize_vec_lengths(&mut self) {
        for _ in &self.shader_desc.textures {
            self.textures.push(None);
        }
        for _ in &self.shader_desc.colors {
            self.colors.push(Color::white());
            self.color_buffers.push(None);
        }
    }

    //=========================================
    // Setters
    //=========================================
    pub fn set_color_with_index(&mut self, color: Color, index: usize) {
        self.colors[index] = color;
        self.upload_color_buffer(index);
    }

    pub fn set_color_with_label(&mut self, color: Color, label: &str) {
        for i in 0..self.shader_desc.colors.len() {
            if self.shader_desc.colors[i].label == label {
                self.colors[i] = color;
                self.upload_color_buffer(i);
                return;
            }
        }
    }

    pub fn set_texture_with_index(&mut self, texture: Option<Arc<TextureAsset>>, index: usize) {
        self.textures[index] = texture;
    }

    pub fn set_texture_with_label(&mut self, texture: Option<Arc<TextureAsset>>, label: &str) {
        for i in 0..self.shader_desc.textures.len() {
            if self.shader_desc.textures[i].label == label {
                self.textures[i] = texture;
                return;
            }
        }
    }

    //=========================================
    // Helpers
    //=========================================
    fn upload_color_buffer(&mut self, index: usize) {
        let device = services().gpu.device();
        let c = self.colors[index];
        let buffer = device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
            label: Some("Color Buffer"),
            contents: bytemuck::cast_slice(&[ColorUniform::new(c.as_r_01(), c.as_g_01(), c.as_b_01(), c.as_a_01())]),
            usage: egui_wgpu::wgpu::BufferUsages::UNIFORM | egui_wgpu::wgpu::BufferUsages::COPY_DST,
        });
        self.color_buffers[index] = Some(buffer);
    }

    //=========================================
    // Combined Binding Group
    //=========================================
    pub fn finalize(&mut self) {
        let device = services().gpu.device();
        let mut entries: Vec<egui_wgpu::wgpu::BindGroupEntry> = Vec::new();
        let mut layouts: Vec<egui_wgpu::wgpu::BindGroupLayoutEntry> = Vec::new();
        let mut binding_index = 0u32;

        // ---- Colors (Uniform Buffers) ----
        for buffer in &self.color_buffers {
            if let Some(buf) = buffer {
                entries.push(egui_wgpu::wgpu::BindGroupEntry {
                    binding: binding_index,
                    resource: buf.as_entire_binding(),
                });

                layouts.push(egui_wgpu::wgpu::BindGroupLayoutEntry {
                    binding: binding_index,
                    visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
                    ty: egui_wgpu::wgpu::BindingType::Buffer {
                        ty: egui_wgpu::wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                });
                binding_index += 1;
            }
        }

        // ---- Textures & Samplers ----
        for texture_opt in &self.textures {
            let texture = texture_opt.as_ref().unwrap_or(&self.none);

            entries.push(egui_wgpu::wgpu::BindGroupEntry {
                binding: binding_index,
                resource: egui_wgpu::wgpu::BindingResource::TextureView(&texture.view),
            });
            layouts.push(egui_wgpu::wgpu::BindGroupLayoutEntry {
                binding: binding_index,
                visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
                ty: egui_wgpu::wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: egui_wgpu::wgpu::TextureViewDimension::D2,
                    sample_type: egui_wgpu::wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            });
            binding_index += 1;

            entries.push(egui_wgpu::wgpu::BindGroupEntry {
                binding: binding_index,
                resource: egui_wgpu::wgpu::BindingResource::Sampler(&texture.sampler),
            });
            layouts.push(egui_wgpu::wgpu::BindGroupLayoutEntry {
                binding: binding_index,
                visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
                ty: egui_wgpu::wgpu::BindingType::Sampler(egui_wgpu::wgpu::SamplerBindingType::Filtering),
                count: None,
            });
            binding_index += 1;
        }

        // ---- Create Layout + Group ----
        let layout = device.create_bind_group_layout(&egui_wgpu::wgpu::BindGroupLayoutDescriptor {
            entries: &layouts[..],
            label: Some("Material Combined Layout"),
        });

        let group = device.create_bind_group(&egui_wgpu::wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &entries[..],
            label: Some("Material Combined Group"),
        });

        let arc_val = Arc::new((group, layout));

        self.cache = Some(arc_val.clone());
    }
    pub fn get_combined_binding_group<'a>(&self) -> Arc<(BindGroup, BindGroupLayout)> {
        // return cached
        if let Some(cached) = &self.cache {
            return cached.clone();
        }

        panic!("Not finalized {}", self.name);
    }
}

//=========================================
// Shader Descriptors
//=========================================
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ShaderDesc {
    shader_module_path: String,
    textures: Vec<ShaderTextureDesc>,
    colors: Vec<ShaderColorDesc>,
}
impl AssetCommon for ShaderDesc {}
impl AssetCommonFromBits<ShaderDesc> for ShaderDesc {
    fn from_bits(bits: &Vec<u8>) -> ShaderDesc {
        let json: serde_json::Value = serde_json::from_slice(bits).expect("file should be proper JSON");
        let my_struct: ShaderDesc = serde_json::from_str(&json.to_string()).unwrap();
        my_struct
    }
}
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ShaderTextureDesc {
    label: String,
}
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ShaderColorDesc {
    label: String,
}

// // Placeholder shader variable descriptors
// pub struct ShaderVec1Desc {}
// pub struct ShaderVec2Desc {}
// pub struct ShaderVec3Desc {}
// pub struct ShaderVec4Desc {}
