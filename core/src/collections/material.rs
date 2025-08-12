use egui_wgpu::wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, ShaderModule};
use serde::{Deserialize, Serialize};

use crate::{
    collections::color::Color,
    io::{asset_loader::AssetLoader, texture_asset::TextureAsset},
    system_adapters::adapter_system_gpu::SystemGPU,
};

//data
#[derive(Clone)]
pub struct Material {
    pub shader: ShaderModule,
    shader_desc: ShaderDesc,
    textures: Vec<Option<TextureAsset>>,
    colors: Vec<Color>,
    colors_uniform: Vec<Option<Buffer>>,
    none: TextureAsset,
}
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

// construction
impl Material {
    fn initialize_vec_lengths(&mut self) {
        for _ in &self.shader_desc.textures {
            self.textures.push(None);
        }
        for _ in &self.shader_desc.colors {
            self.colors.push(Color::black());
            self.colors_uniform.push(None);
        }
    }
    pub fn new(shader_desc: ShaderDesc) -> Material {
        let device = &SystemGPU::get_device();
        let shader = AssetLoader::load_shader_module(device, &shader_desc.shader_module_path);

        let mut m = Material {
            shader_desc: shader_desc,
            textures: Vec::new(),
            colors: Vec::new(),
            colors_uniform: Vec::new(),
            none: TextureAsset::none(),
            shader: shader,
        };
        m.initialize_vec_lengths();
        m
    }
    pub fn set_color_with_index(&mut self, color: Color, index: usize) {
        self.colors[index] = color;
    }
    pub fn set_color_with_label(&mut self, color: Color, label: &str) {
        for i in 0..self.shader_desc.colors.len() {
            let is_same = self.shader_desc.colors[i].label == label;
            if !is_same {
                continue;
            };

            let device = SystemGPU::get_device();
            let color_buffer = device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
                label: Some("Color Buffer"),
                contents: bytemuck::cast_slice(&[ColorUniform::new(color.as_r_01(), color.as_g_01(), color.as_b_01(), color.as_a_01())]),
                usage: egui_wgpu::wgpu::BufferUsages::UNIFORM | egui_wgpu::wgpu::BufferUsages::COPY_DST,
            });
            self.colors[i] = color;
            self.colors_uniform[i] = Some(color_buffer);
            return;
        }
    }
    pub fn set_texture_with_index(&mut self, texture: Option<TextureAsset>, index: usize) {
        self.textures[index] = texture;
    }
    pub fn set_texture_with_label(&mut self, texture: Option<TextureAsset>, label: &str) {
        for i in 0..self.shader_desc.textures.len() {
            let is_same = self.shader_desc.textures[i].label == label;
            if !is_same {
                continue;
            };

            self.textures[i] = texture;
            return;
        }
    }

    pub fn get_color_binding_group<'a>(&self, device: &Device) -> (BindGroup, BindGroupLayout) {
        // create entries
        let mut i = 0;
        let mut entries: Vec<egui_wgpu::wgpu::BindGroupEntry> = Vec::new();

        for t in &self.colors_uniform {
            let Some(buffer) = t else {
                continue;
            };
            entries.push(egui_wgpu::wgpu::BindGroupEntry {
                binding: (i),
                resource: buffer.as_entire_binding(),
            });
            i = i + 1;
        }

        // create layout
        let mut i = 0;
        let mut layouts: Vec<egui_wgpu::wgpu::BindGroupLayoutEntry> = Vec::new();
        for _ in &self.colors {
            layouts.push(egui_wgpu::wgpu::BindGroupLayoutEntry {
                binding: i,
                visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
                ty: egui_wgpu::wgpu::BindingType::Buffer {
                    ty: egui_wgpu::wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
            i = i + 1;
        }

        let texture_bind_group_layout = device.create_bind_group_layout(&egui_wgpu::wgpu::BindGroupLayoutDescriptor {
            entries: &layouts[..],
            label: None,
        });

        let diffuse_bind_group = device.create_bind_group(&egui_wgpu::wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &entries[..],
            label: None,
        });

        (diffuse_bind_group, texture_bind_group_layout)
    }
    pub fn get_texture_binding_group<'a>(&self, device: &Device) -> (BindGroup, BindGroupLayout) {
        // create entries
        let mut i = 0;
        let mut entries: Vec<egui_wgpu::wgpu::BindGroupEntry> = Vec::new();
        for t in &self.textures {
            let texture: &TextureAsset;
            match t {
                Some(x) => {
                    texture = x;
                }
                None => {
                    texture = &self.none;
                }
            }

            entries.push(egui_wgpu::wgpu::BindGroupEntry {
                binding: (i * 2),
                resource: egui_wgpu::wgpu::BindingResource::TextureView(&texture.view),
            });
            entries.push(egui_wgpu::wgpu::BindGroupEntry {
                binding: (i * 2) + 1,
                resource: egui_wgpu::wgpu::BindingResource::Sampler(&texture.sampler),
            });
            i = i + 1;
        }

        // create layout
        let mut i = 0;
        let mut layouts: Vec<egui_wgpu::wgpu::BindGroupLayoutEntry> = Vec::new();
        for _ in &self.textures {
            layouts.push(egui_wgpu::wgpu::BindGroupLayoutEntry {
                binding: (i * 2),
                visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
                ty: egui_wgpu::wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: egui_wgpu::wgpu::TextureViewDimension::D2,
                    sample_type: egui_wgpu::wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            });
            layouts.push(egui_wgpu::wgpu::BindGroupLayoutEntry {
                binding: (i * 2) + 1,
                visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
                ty: egui_wgpu::wgpu::BindingType::Sampler(egui_wgpu::wgpu::SamplerBindingType::Filtering),
                count: None,
            });
            i = i + 1;
        }

        let texture_bind_group_layout = device.create_bind_group_layout(&egui_wgpu::wgpu::BindGroupLayoutDescriptor {
            entries: &layouts[..],
            label: None,
        });

        let diffuse_bind_group = device.create_bind_group(&egui_wgpu::wgpu::BindGroupDescriptor {
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
    shader_module_path: String,
    textures: Vec<ShaderTextureDesc>,
    colors: Vec<ShaderColorDesc>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct ShaderTextureDesc {
    label: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ShaderColorDesc {
    label: String,
}
pub struct ShaderVec1Desc {}
pub struct ShaderVec2Desc {}
pub struct ShaderVec3Desc {}
pub struct ShaderVec4Desc {}
