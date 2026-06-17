use crate::{assets::asset::AssetCommonFromBits, engine::engine_services::services};

use super::asset::AssetCommon;

// data
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TextureAsset {
    // width: i32,
    // height: i32,
    // bytes: Vec<u8>,
    pub texture: egui_wgpu::wgpu::Texture,
    pub view: egui_wgpu::wgpu::TextureView,
    pub sampler: egui_wgpu::wgpu::Sampler,
}
// construction
impl TextureAsset {
    pub const DEPTH_FORMAT: egui_wgpu::wgpu::TextureFormat = egui_wgpu::wgpu::TextureFormat::Depth32Float; // 1.

    // pub fn create_depth_texture(gpu: SystemGPU, label: &str) -> Self {
    //     // panic!("GPU CONVERSION FAILURE");

    //     // let device = SystemGPU::get_device();
    //     // let config = SystemGPU::get_config();
    //     let device = gpu.device;
    //     let config = gpu.config;
    //     let size = egui_wgpu::wgpu::Extent3d {
    //         // 2.
    //         width: config.width.max(1),
    //         height: config.height.max(1),
    //         depth_or_array_layers: 1,
    //     };

    //     // println!("create depth with size {}, {}", config.width.max(1), config.height.max(1));
    //     let desc = egui_wgpu::wgpu::TextureDescriptor {
    //         label: Some(label),
    //         mip_level_count: 1,
    //         sample_count: 1,
    //         dimension: egui_wgpu::wgpu::TextureDimension::D2,
    //         format: Self::DEPTH_FORMAT,
    //         usage: egui_wgpu::wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
    //             |  egui_wgpu::wgpu::TextureUsages::TEXTURE_BINDING,
    //         view_formats: &[],
    //         size,
    //     };
    //     let texture = device.create_texture(&desc);

    //     let view = texture.create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());
    //     let sampler = device.create_sampler(&egui_wgpu::wgpu::SamplerDescriptor {
    //         // 4.
    //         address_mode_u: egui_wgpu::wgpu::AddressMode::ClampToEdge,
    //         address_mode_v: egui_wgpu::wgpu::AddressMode::ClampToEdge,
    //         address_mode_w: egui_wgpu::wgpu::AddressMode::ClampToEdge,
    //         mag_filter: egui_wgpu::wgpu::FilterMode::Linear,
    //         min_filter: egui_wgpu::wgpu::FilterMode::Linear,
    //         mipmap_filter: egui_wgpu::wgpu::FilterMode::Nearest,
    //         compare: Some(egui_wgpu::wgpu::CompareFunction::LessEqual), // 5.
    //         lod_min_clamp: 0.0,
    //         lod_max_clamp: 100.0,
    //         ..Default::default()
    //     });

    //     TextureAsset {
    //         // width: size.width as i32,
    //         // height: size.height as i32,
    //         sampler: sampler,
    //         texture: texture,
    //         view: view,
    //     }
    // }
    // pub fn new_from_buffer(label: Option<&str>, device: &egui_wgpu::wgpu::Device, queue: &egui_wgpu::wgpu::Queue, width: u32, height: u32, buffer: ImageBuffer<Rgba<u8>, &[u8]>) -> Texture_asset {
    pub fn none() -> TextureAsset {
        let s = services();
        let device = s.gpu.device();

        let size = egui_wgpu::wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 };
        let texture = device.create_texture(&egui_wgpu::wgpu::TextureDescriptor {
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            dimension: egui_wgpu::wgpu::TextureDimension::D2,
            format: egui_wgpu::wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: egui_wgpu::wgpu::TextureUsages::TEXTURE_BINDING | egui_wgpu::wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
            size,
        });
        let view = texture.create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&egui_wgpu::wgpu::SamplerDescriptor {
            address_mode_u: egui_wgpu::wgpu::AddressMode::ClampToEdge,
            address_mode_v: egui_wgpu::wgpu::AddressMode::ClampToEdge,
            address_mode_w: egui_wgpu::wgpu::AddressMode::ClampToEdge,
            mag_filter: egui_wgpu::wgpu::FilterMode::Linear,
            min_filter: egui_wgpu::wgpu::FilterMode::Nearest,
            mipmap_filter: egui_wgpu::wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // return
        TextureAsset { sampler: sampler, texture: texture, view: view }
    }
    // pub fn default() -> TextureAsset {
    //     let bytes = include_bytes!("../../../assets/built_in/texture/default.png");
    //     let img = image::load_from_memory(bytes).unwrap();
    //     let rgba = img.to_rgba8();

    //     TextureAsset::new_from_buffer(None, 512, 512, &rgba.to_vec()[..])
    // }
    pub fn new_from_buffer(label: Option<&str>, width: u32, height: u32, buffer: &[u8]) -> TextureAsset {
        let s = services();
        let queue = s.gpu.queue();
        let device = s.gpu.device();

        if width % 2 != 0 {
            panic!("texture width not power of 2")
        }
        if height % 2 != 0 {
            panic!("texture height not power of 2")
        }
        let size = egui_wgpu::wgpu::Extent3d { width: width, height: height, depth_or_array_layers: 1 };

        let texture = device.create_texture(&egui_wgpu::wgpu::TextureDescriptor {
            label,
            mip_level_count: 1,
            sample_count: 1,
            dimension: egui_wgpu::wgpu::TextureDimension::D2,
            format: egui_wgpu::wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: egui_wgpu::wgpu::TextureUsages::TEXTURE_BINDING | egui_wgpu::wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
            size,
        });

        // let rgba = texture();

        println!("width: {}", width);
        println!("height: {}", height);
        // println!("bytes: {}", rgba.len);
        println!("expected: {}", width * height * 4);
        queue.write_texture(
            egui_wgpu::wgpu::TexelCopyTextureInfo {
                aspect: egui_wgpu::wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: egui_wgpu::wgpu::Origin3d::ZERO,
            },
            &buffer,
            egui_wgpu::wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&egui_wgpu::wgpu::SamplerDescriptor {
            address_mode_u: egui_wgpu::wgpu::AddressMode::ClampToEdge,
            address_mode_v: egui_wgpu::wgpu::AddressMode::ClampToEdge,
            address_mode_w: egui_wgpu::wgpu::AddressMode::ClampToEdge,
            mag_filter: egui_wgpu::wgpu::FilterMode::Linear,
            min_filter: egui_wgpu::wgpu::FilterMode::Nearest,
            mipmap_filter: egui_wgpu::wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // return
        TextureAsset {
            // width: size.width as i32,
            // height: size.height as i32,
            sampler: sampler,
            texture: texture,
            view: view,
        }
    }
}
// public
impl TextureAsset {}
// asset
impl AssetCommon for TextureAsset {}
impl AssetCommonFromBits<TextureAsset> for TextureAsset {
    fn from_bits(bits: &Vec<u8>) -> TextureAsset {
        let image: image::DynamicImage = image::load_from_memory(&bits).unwrap();
        let texture = TextureAsset::new_from_buffer(None, image.width(), image.height(), image.as_bytes());
        texture
    }
}
