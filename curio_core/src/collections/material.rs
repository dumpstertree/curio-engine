// use egui_wgpu::wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, ShaderModule};
// use serde::{Deserialize, Serialize};

// use crate::{
//     collections::color::Color,
//     io::{asset_loader::AssetLoader, texture_asset::TextureAsset},
//     system_adapters::adapter_system_gpu::SystemGPU,
// };

// //data
// pub struct Material {
//     pub shader: ShaderModule,
//     shader_desc: ShaderDesc,
//     textures: Vec<Option<TextureAsset>>,
//     colors: Vec<Color>,
//     colors_uniform: Vec<Option<Buffer>>,
//     none: TextureAsset,
// }
// #[repr(C)]
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct ColorUniform {
//     color: [f32; 4],
// }
// impl ColorUniform {
//     pub fn new(r: f32, g: f32, b: f32, a: f32) -> ColorUniform {
//         ColorUniform { color: [r, g, b, a] }
//     }
// }

// // construction
// impl Material {
//     pub fn instantiate(&self) -> Material {
//         Material {
//             shader: self.shader.clone(),
//             shader_desc: self.shader_desc.clone(),
//             textures: self.textures.clone(),
//             colors: self.colors.clone(),
//             colors_uniform: self.colors_uniform.clone(),
//             none: self.none.clone(),
//         }
//     }
//     fn initialize_vec_lengths(&mut self) {
//         for _ in &self.shader_desc.textures {
//             self.textures.push(None);
//         }
//         for _ in &self.shader_desc.colors {
//             self.colors.push(Color::black());
//             self.colors_uniform.push(None);
//         }
//     }
//     pub fn new(shader_desc: ShaderDesc) -> Material {
//         let device = &SystemGPU::get_device();
//         let shader = AssetLoader::load_shader_module(device, &shader_desc.shader_module_path);

//         let mut m = Material {
//             shader_desc: shader_desc,
//             textures: Vec::new(),
//             colors: Vec::new(),
//             colors_uniform: Vec::new(),
//             none: TextureAsset::none(),
//             shader: shader,
//         };
//         m.initialize_vec_lengths();
//         m
//     }
//     pub fn set_color_with_index(&mut self, color: Color, index: usize) {
//         self.colors[index] = color;
//     }
//     pub fn set_color_with_label(&mut self, color: Color, label: &str) {
//         for i in 0..self.shader_desc.colors.len() {
//             let is_same = self.shader_desc.colors[i].label == label;
//             if !is_same {
//                 continue;
//             };

//             let device = SystemGPU::get_device();
//             let color_buffer = device.create_buffer_init(&egui_wgpu::wgpu::util::BufferInitDescriptor {
//                 label: Some("Color Buffer"),
//                 contents: bytemuck::cast_slice(&[ColorUniform::new(color.as_r_01(), color.as_g_01(), color.as_b_01(), color.as_a_01())]),
//                 usage: egui_wgpu::wgpu::BufferUsages::UNIFORM | egui_wgpu::wgpu::BufferUsages::COPY_DST,
//             });
//             self.colors[i] = color;
//             self.colors_uniform[i] = Some(color_buffer);
//             return;
//         }
//     }
//     pub fn set_texture_with_index(&mut self, texture: Option<TextureAsset>, index: usize) {
//         self.textures[index] = texture;
//     }
//     pub fn set_texture_with_label(&mut self, texture: Option<TextureAsset>, label: &str) {
//         for i in 0..self.shader_desc.textures.len() {
//             let is_same = self.shader_desc.textures[i].label == label;
//             if !is_same {
//                 continue;
//             };

//             self.textures[i] = texture;
//             return;
//         }
//     }

//     pub fn get_color_binding_group<'a>(&self, device: &Device) -> (BindGroup, BindGroupLayout) {
//         // create entries
//         let mut i = 0;
//         let mut entries: Vec<egui_wgpu::wgpu::BindGroupEntry> = Vec::new();

//         for t in &self.colors_uniform {
//             let Some(buffer) = t else {
//                 continue;
//             };
//             entries.push(egui_wgpu::wgpu::BindGroupEntry { binding: (i), resource: buffer.as_entire_binding() });
//             i = i + 1;
//         }

//         // create layout
//         let mut i = 0;
//         let mut layouts: Vec<egui_wgpu::wgpu::BindGroupLayoutEntry> = Vec::new();
//         for _ in &self.colors {
//             layouts.push(egui_wgpu::wgpu::BindGroupLayoutEntry {
//                 binding: i,
//                 visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
//                 ty: egui_wgpu::wgpu::BindingType::Buffer {
//                     ty: egui_wgpu::wgpu::BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: None,
//                 },
//                 count: None,
//             });
//             i = i + 1;
//         }

//         let texture_bind_group_layout = device.create_bind_group_layout(&egui_wgpu::wgpu::BindGroupLayoutDescriptor { entries: &layouts[..], label: None });

//         let diffuse_bind_group = device.create_bind_group(&egui_wgpu::wgpu::BindGroupDescriptor {
//             layout: &texture_bind_group_layout,
//             entries: &entries[..],
//             label: None,
//         });

//         (diffuse_bind_group, texture_bind_group_layout)
//     }
//     pub fn get_texture_binding_group<'a>(&self, device: &Device) -> (BindGroup, BindGroupLayout) {
//         // create entries
//         let mut i = 0;
//         let mut entries: Vec<egui_wgpu::wgpu::BindGroupEntry> = Vec::new();
//         for t in &self.textures {
//             let texture: &TextureAsset;
//             match t {
//                 Some(x) => {
//                     texture = x;
//                 }
//                 None => {
//                     texture = &self.none;
//                 }
//             }

//             entries.push(egui_wgpu::wgpu::BindGroupEntry {
//                 binding: (i * 2),
//                 resource: egui_wgpu::wgpu::BindingResource::TextureView(&texture.view),
//             });
//             entries.push(egui_wgpu::wgpu::BindGroupEntry {
//                 binding: (i * 2) + 1,
//                 resource: egui_wgpu::wgpu::BindingResource::Sampler(&texture.sampler),
//             });
//             i = i + 1;
//         }

//         // create layout
//         let mut i = 0;
//         let mut layouts: Vec<egui_wgpu::wgpu::BindGroupLayoutEntry> = Vec::new();
//         for _ in &self.textures {
//             layouts.push(egui_wgpu::wgpu::BindGroupLayoutEntry {
//                 binding: (i * 2),
//                 visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
//                 ty: egui_wgpu::wgpu::BindingType::Texture {
//                     multisampled: false,
//                     view_dimension: egui_wgpu::wgpu::TextureViewDimension::D2,
//                     sample_type: egui_wgpu::wgpu::TextureSampleType::Float { filterable: true },
//                 },
//                 count: None,
//             });
//             layouts.push(egui_wgpu::wgpu::BindGroupLayoutEntry {
//                 binding: (i * 2) + 1,
//                 visibility: egui_wgpu::wgpu::ShaderStages::FRAGMENT,
//                 ty: egui_wgpu::wgpu::BindingType::Sampler(egui_wgpu::wgpu::SamplerBindingType::Filtering),
//                 count: None,
//             });
//             i = i + 1;
//         }

//         let texture_bind_group_layout = device.create_bind_group_layout(&egui_wgpu::wgpu::BindGroupLayoutDescriptor { entries: &layouts[..], label: None });

//         let diffuse_bind_group = device.create_bind_group(&egui_wgpu::wgpu::BindGroupDescriptor {
//             layout: &texture_bind_group_layout,
//             entries: &entries[..],
//             label: None,
//         });

//         (diffuse_bind_group, texture_bind_group_layout)
//     }
// }
// // public
// impl Material {}
// // private
// impl Material {}
// // asset

// #[derive(Clone, Serialize, Deserialize)]
// pub struct ShaderDesc {
//     shader_module_path: String,
//     textures: Vec<ShaderTextureDesc>,
//     colors: Vec<ShaderColorDesc>,
// }
// #[derive(Clone, Serialize, Deserialize)]
// pub struct ShaderTextureDesc {
//     label: String,
// }

use std::{hash::Hash, sync::Arc};

// #[derive(Clone, Serialize, Deserialize)]
// pub struct ShaderColorDesc {
//     label: String,
// }
// pub struct ShaderVec1Desc {}
// pub struct ShaderVec2Desc {}
// pub struct ShaderVec3Desc {}
// pub struct ShaderVec4Desc {}
use egui_wgpu::wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, ShaderModule};
use serde::{Deserialize, Serialize};

use crate::{
    collections::color::Color,
    io::{asset_loader::AssetLoader, texture_asset::TextureAsset},
    random::Random,
    system_adapters::adapter_system_gpu::SystemGPU,
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
        let device = &SystemGPU::get_device();
        // AssetLoader::load_shader_module(device, Builtin &self.shader_desc.shader_module_path)
        AssetLoader::load_shader_module(device, &AssetLoader::try_lookup_key_for_name(&self.shader_desc.shader_module_path).unwrap())
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
        let device = SystemGPU::get_device();
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
        let device = SystemGPU::get_device();
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
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ShaderTextureDesc {
    label: String,
}
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ShaderColorDesc {
    label: String,
}

// Placeholder shader variable descriptors
pub struct ShaderVec1Desc {}
pub struct ShaderVec2Desc {}
pub struct ShaderVec3Desc {}
pub struct ShaderVec4Desc {}
