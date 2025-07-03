use wgpu::TextureView;

use super::Asset::Asset;

// data
#[derive(Clone)]
pub struct Texture_asset {
    width: i32,
    height: i32,
    // bytes: Vec<u8>,
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}
// construction
impl Texture_asset {
    // pub fn new_from_buffer(label: Option<&str>, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32, buffer: ImageBuffer<Rgba<u8>, &[u8]>) -> Texture_asset {
    pub fn none(device: &wgpu::Device) -> Texture_asset {
        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
            size,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // return
        Texture_asset {
            width: 0,
            height: 0,
            sampler: sampler,
            texture: texture,
            view: view,
        }
    }
    pub fn default(device: &wgpu::Device, queue: &wgpu::Queue) -> Texture_asset {
        let bytes = include_bytes!("../../default_texture.jpg");
        let img = image::load_from_memory(bytes).unwrap();
        let rgba = img.to_rgba8();

        Texture_asset::new_from_buffer(None, device, queue, 1024, 1024, &rgba.to_vec()[..])
    }
    pub fn new_from_buffer(label: Option<&str>, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32, buffer: &[u8]) -> Texture_asset {
        if width % 2 != 0 {
            panic!("texture width not power of 2")
        }
        if height % 2 != 0 {
            panic!("texture height not power of 2")
        }
        let size = wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
            size,
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &buffer,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // return
        Texture_asset {
            width: size.width as i32,
            height: size.height as i32,
            sampler: sampler,
            texture: texture,
            view: view,
        }
    }
}
// public
impl Texture_asset {}
// asset
impl Asset for Texture_asset {}
