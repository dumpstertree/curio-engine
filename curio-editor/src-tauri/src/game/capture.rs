use egui_wgpu::wgpu::{Buffer, BufferView, Device, Extent3d, MapMode, Origin3d, Queue, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureFormat};

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

// ── Dimensions ───────────────────────────────────────────────────────────────

pub const CAPTURE_WIDTH: u32 = 1280;
pub const CAPTURE_HEIGHT: u32 = 720;

// ── Double-buffered frame store ───────────────────────────────────────────────
// The game thread writes into whichever buffer isn't being read.
// The Tauri command thread reads the last completed buffer without blocking
// the game thread.

static FRAME_BUFFERS: Lazy<[Arc<RwLock<Vec<u8>>>; 2]> = Lazy::new(|| {
    let size = (CAPTURE_WIDTH * CAPTURE_HEIGHT * 4) as usize;
    [Arc::new(RwLock::new(vec![0u8; size])), Arc::new(RwLock::new(vec![0u8; size]))]
});

// Which buffer index the game thread writes into next
static WRITE_IDX: AtomicUsize = AtomicUsize::new(0);
static FRAME_READY: AtomicBool = AtomicBool::new(false);

// ── Utility ───────────────────────────────────────────────────────────────────

pub fn align_to(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

// ── Write path (game thread) ──────────────────────────────────────────────────

/// Copies `capture_texture` into `readback_buffer` using the provided encoder.
/// Call this BEFORE submitting — the copy is recorded into the same encoder
/// as the render pass so there is only one queue submit per frame.
pub fn record_readback(encoder: &mut egui_wgpu::wgpu::CommandEncoder, capture_texture: &Texture, readback_buffer: &Buffer) {
    let width = capture_texture.width();
    let height = capture_texture.height();
    let bytes_per_row = align_to(width * 4, 256);

    encoder.copy_texture_to_buffer(
        TexelCopyTextureInfo {
            texture: capture_texture,
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
}

/// Maps `readback_buffer`, strips row padding, and writes raw RGBA pixels into
/// the next double-buffer slot. Call this AFTER queue.submit().
pub fn map_and_store(device: &Device, readback_buffer: &Buffer, width: u32, height: u32) {
    let slice = readback_buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(MapMode::Read, move |result| {
        tx.send(result).ok();
    });

    // Block until the GPU has finished writing into the readback buffer.
    device.poll(egui_wgpu::wgpu::MaintainBase::Wait);
    rx.recv()
        .expect("map_async channel dropped")
        .expect("map failed");

    {
        let view: BufferView = slice.get_mapped_range();
        let pixel_stride = (width * 4) as usize;
        let padded_stride = align_to(width * 4, 256) as usize;

        // Write into the current write slot
        let write_idx = WRITE_IDX.load(Ordering::Acquire);
        let mut buf = FRAME_BUFFERS[write_idx].write();

        // Resize if the capture resolution changed
        let expected = pixel_stride * height as usize;
        if buf.len() != expected {
            buf.resize(expected, 0);
        }

        for row in 0..height as usize {
            let src_start = row * padded_stride;
            let dst_start = row * pixel_stride;
            buf[dst_start..dst_start + pixel_stride].copy_from_slice(&view[src_start..src_start + pixel_stride]);
        }
    }

    readback_buffer.unmap();

    // Swap write index so next frame writes into the other slot
    let old_idx = WRITE_IDX.fetch_xor(1, Ordering::AcqRel);
    let _ = old_idx; // suppress unused warning

    FRAME_READY.store(true, Ordering::Release);
}

// ── Read path (Tauri command thread) ─────────────────────────────────────────

/// Returns Some(raw RGBA bytes) if a new frame is available, None otherwise.
/// Clears the ready flag so the same frame is never returned twice.
pub fn take_frame() -> Option<Vec<u8>> {
    if !FRAME_READY.swap(false, Ordering::AcqRel) {
        return None;
    }
    // Read from the slot that is NOT currently being written into
    let read_idx = WRITE_IDX.load(Ordering::Acquire); // writer just swapped, so this is the "done" slot
    Some(FRAME_BUFFERS[read_idx].read().clone())
}
