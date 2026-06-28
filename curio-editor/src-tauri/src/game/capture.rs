use egui_wgpu::wgpu::{Buffer, BufferView, Device, Extent3d, ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, MapMode, Origin3d, Queue, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureFormat};

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

// ── Dimensions ───────────────────────────────────────────────────────────────

pub const CAPTURE_WIDTH: u32 = 1280;
pub const CAPTURE_HEIGHT: u32 = 720;

// ── Shared frame buffer ───────────────────────────────────────────────────────
// The game thread writes raw RGBA bytes here after each GPU readback.
// The Tauri command `get_frame` reads and clears it from the main thread.

static FRAME_BUFFER: Lazy<Arc<RwLock<Vec<u8>>>> = Lazy::new(|| Arc::new(RwLock::new(Vec::with_capacity((CAPTURE_WIDTH * CAPTURE_HEIGHT * 4) as usize))));

static FRAME_READY: AtomicBool = AtomicBool::new(false);

// ── Write path (game thread) ──────────────────────────────────────────────────

/// Reads `capture_texture` back to CPU and stores raw RGBA bytes in FRAME_BUFFER.
/// Called from `GameRunner2::render_frame()` after each submit.
pub fn capture_frame(
    device: &Device,
    queue: &Queue,
    capture_texture: Arc<Texture>,
    readback_buffer: &Buffer,
    _format: TextureFormat, // kept for API compatibility, unused now
) {
    let width = capture_texture.width();
    let height = capture_texture.height();
    let bytes_per_row = align_to(width * 4, 256);

    // ── GPU → readback buffer ─────────────────────────────────────────────
    let mut encoder = device.create_command_encoder(&egui_wgpu::wgpu::CommandEncoderDescriptor { label: Some("capture_readback_encoder") });

    encoder.copy_texture_to_buffer(
        TexelCopyTextureInfo {
            texture: &capture_texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        TexelCopyBufferInfo {
            buffer: readback_buffer,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        Extent3d { width, height, depth_or_array_layers: 1 },
    );

    queue.submit(std::iter::once(encoder.finish()));

    // ── Map and copy ──────────────────────────────────────────────────────
    let slice = readback_buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(MapMode::Read, move |result| {
        tx.send(result).ok();
    });

    // Poll until the map is ready — we are on the game thread, not the wgpu
    // polling thread, so we drive it manually here.
    device.poll(egui_wgpu::wgpu::MaintainBase::Wait);
    rx.recv()
        .expect("map_async channel dropped")
        .expect("map failed");

    {
        let view: BufferView = slice.get_mapped_range();

        // Strip the wgpu row-padding: each padded row is `bytes_per_row` bytes
        // wide but we only want `width * 4` bytes of actual pixel data per row.
        let pixel_stride = (width * 4) as usize;
        let padded_stride = bytes_per_row as usize;

        let mut buf = FRAME_BUFFER.write();
        buf.clear();
        buf.reserve(pixel_stride * height as usize);

        for row in 0..height as usize {
            let start = row * padded_stride;
            buf.extend_from_slice(&view[start..start + pixel_stride]);
        }
    }

    readback_buffer.unmap();
    FRAME_READY.store(true, Ordering::Release);
}

// ── Read path (Tauri command thread) ─────────────────────────────────────────

/// Returns Some(raw RGBA bytes) if a new frame is available, None otherwise.
/// Calling this clears the ready flag so the same frame won't be returned twice.
pub fn take_frame() -> Option<Vec<u8>> {
    if FRAME_READY.swap(false, Ordering::AcqRel) {
        Some(FRAME_BUFFER.read().clone())
    } else {
        None
    }
}

// ── Utility ───────────────────────────────────────────────────────────────────

pub fn align_to(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}
