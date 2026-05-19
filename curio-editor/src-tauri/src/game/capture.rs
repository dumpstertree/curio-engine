use crate::game::encoding::{base64_encode, encode_frame};
use crate::utils::align_to;
use egui_wgpu::wgpu::{Buffer, CommandEncoderDescriptor, MaintainBase, MapMode, Origin3d, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureFormat};
use tauri::Emitter;

use std::sync::Arc;

pub const CAPTURE_WIDTH: u32 = 1280;
pub const CAPTURE_HEIGHT: u32 = 720;

pub fn capture_frame(app_handle: tauri::AppHandle, device: &egui_wgpu::wgpu::Device, queue: &egui_wgpu::wgpu::Queue, texture: Arc<Texture>, readback: &Buffer, format: TextureFormat) {
    let bytes_per_row = align_to(CAPTURE_WIDTH * 4, 256);

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: Some("capture") });

    encoder.copy_texture_to_buffer(
        TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        TexelCopyBufferInfo {
            buffer: readback,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(CAPTURE_HEIGHT),
            },
        },
        egui_wgpu::wgpu::Extent3d {
            width: CAPTURE_WIDTH,
            height: CAPTURE_HEIGHT,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(Some(encoder.finish()));

    let slice = readback.slice(..);

    slice.map_async(MapMode::Read, |_| {});

    device.poll(MaintainBase::Wait);

    let pixels: Vec<u8> = {
        let view = slice.get_mapped_range();

        view.chunks(bytes_per_row as usize)
            .flat_map(|row| &row[..CAPTURE_WIDTH as usize * 4])
            .copied()
            .collect()
    };

    readback.unmap();

    std::thread::spawn(move || {
        if let Some(encoded) = encode_frame(&pixels, CAPTURE_WIDTH, CAPTURE_HEIGHT, format) {
            let encoded = base64_encode(&encoded);

            app_handle.emit("viewport_frame", encoded).ok();
        }
    });
}
