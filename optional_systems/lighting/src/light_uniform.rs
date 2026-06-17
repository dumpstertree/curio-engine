use bytemuck::{Pod, Zeroable};
use curio_core::{ExtensionsF32, services};
use egui_wgpu::wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ShaderStages};
use serde::{Deserialize, Serialize};
use std::{hash::Hash, num::NonZeroU64};

// CPU-side light types for your ECS
#[derive(Clone, Copy, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightType {
    #[default]
    Point, // uses position + radius/falloff in params
    Directional, // uses direction vector
}

#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DrawCallLight {
    pub light_type: LightType,
    pub position: [f32; 3],  // world-space (ignored for directional)
    pub direction: [f32; 3], // for directional lights (should be normalized)
    pub color: [f32; 3],
    pub intensity: f32,
    pub radius: f32, // for point lights (falloff)
}
impl Eq for DrawCallLight {}
impl Hash for DrawCallLight {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.light_type.hash(state);
        self.position[0].hash(state);
        self.position[1].hash(state);
        self.position[2].hash(state);
        self.direction[0].hash(state);
        self.direction[1].hash(state);
        self.direction[2].hash(state);
        self.color[0].hash(state);
        self.color[1].hash(state);
        self.color[2].hash(state);
        self.intensity.hash(state);
        self.radius.hash(state);
    }
}

// GPU-safe layout (matches WGSL struct below).
// Make each light take 4 vec4 (16 bytes * 4 = 64 bytes) for simple alignment.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuLight {
    pub position: [f32; 4],        // xyz = position, w = light_type (0=dir,1=point)
    pub color_intensity: [f32; 4], // rgb = color, a = intensity
    pub direction_radius: [f32; 4], // xyz = direction, w = radius (for point)
                                   // pub _padding: [f32; 4],         // unused (keeps 64 bytes)
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuLightArray {
    // pack num_lights into x of first vec4; rest unused
    pub count: [u32; MAX_LIGHTS],
    // followed by array of lights
    // NOTE: actual buffer size will be: 16 (count) + MAX_LIGHTS * 64
    // Represented here as zero-length for convenience (we write bytes directly).
}

pub const MAX_LIGHTS: usize = 16;

pub struct LightSystem {
    pub buffer: Buffer,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}
impl LightSystem {
    /// Create a LightSystem: allocates uniform buffer sized for MAX_LIGHTS and creates bind group layout.
    pub fn new() -> Self {
        let device = services().gpu.device();

        // Each GpuLight is 64 bytes, plus a 16-byte header
        let header_size = 16u64;
        // let padding_size = 16u64;
        let light_size = 64u64;
        let size = header_size + light_size * MAX_LIGHTS as u64;

        // Create uniform buffer
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Light Uniform Buffer"),
            size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Bind group layout: binding 0 = uniform buffer
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Light Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: Some(NonZeroU64::new(size).unwrap()),
                },
                count: None,
            }],
        });

        // Bind group
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }],
        });

        Self { buffer, bind_group, bind_group_layout }
    }

    /// Write lights to GPU buffer. Call each frame before drawing 3D.
    pub fn update(&self, sun: &DrawCallLight, lights: &[DrawCallLight]) {
        let queue = services().gpu.queue();
        let n = lights.len().min(MAX_LIGHTS);

        // Header: first 16 bytes. First u32 = count
        let mut bytes: Vec<u8> = Vec::new(); //Vec::with_capacity(16 + 64 * MAX_LIGHTS);
        bytes.extend(&(3 as u32).to_le_bytes());
        bytes.extend(&[0u8; 12]); // padding to 16 bytes

        Self::add_light(sun, &mut bytes);
        // Serialize each light
        for light in lights.iter().take(n) {
            Self::add_light(light, &mut bytes);
        }

        queue.write_buffer(&self.buffer, 0, &bytes);
    }

    fn add_light(l: &DrawCallLight, bytes: &mut Vec<u8>) {
        let mut g = GpuLight::zeroed();

        // position.xyz ; w = light type (0 = dir, 1 = point)
        g.position[0..3].copy_from_slice(&l.position);
        g.position[3] = match l.light_type {
            LightType::Directional => 0.0,
            LightType::Point => 1.0,
        };

        // color.rgb + intensity
        g.color_intensity[0..3].copy_from_slice(&l.color);
        g.color_intensity[3] = l.intensity;

        // direction.xyz + radius in w
        g.direction_radius[0..3].copy_from_slice(&l.direction);
        g.direction_radius[3] = l.radius;

        // _padding already zeroed
        bytes.extend_from_slice(bytemuck::bytes_of(&g));
    }
}
