//! GPU → CPU frame readback for the embedded game viewport.
//!
//! **This intentionally reverts the "tight wgpu integration" from an
//! earlier pass.** That version had the game runner thread share eframe's
//! own `Device`/`Queue`/`egui_wgpu::Renderer` to get a zero-copy texture —
//! but that means a second, always-running thread calling
//! `renderer.write()` up to 60×/sec on the *same* lock eframe's own UI
//! paint loop needs every frame to draw anything at all (including
//! sampling that very texture). That's a real cross-thread contention
//! point that the original Tauri build's fully-isolated game device never
//! had, and lines up with the "hangs accessing services, looks like a race"
//! symptom — a bug class that literally could not exist when the game had
//! its own private device touched by no one else.
//!
//! Back to the original, proven-stable shape instead: the game runner owns
//! a fully private headless `Device`/`Queue` (see `game_runner.rs::setup_gpu`)
//! that nothing else in the process ever touches, and frames cross to the
//! UI thread as plain RGBA bytes through this module — a ping-pong mapped
//! buffer readback, landing in a `Mutex<Option<Frame>>` the UI thread polls
//! once per repaint (`center_panel.rs` uploads it into a persistent
//! `egui::TextureHandle` via `.set(...)`). This costs a CPU→GPU reupload
//! each frame instead of a zero-copy sample — the same cost the original
//! Tauri build's canvas-texture-per-frame approach already paid, so it's
//! not a regression versus what was previously working.
//!
//! GLB/PNG/Spine/prefab previews are unaffected by any of this: they render
//! synchronously on the UI thread itself (no independent background
//! thread), so there's no concurrent producer/consumer to race in the
//! first place, and they keep the zero-copy shared-texture registration.

use egui_wgpu::wgpu::{
    self, Buffer, BufferDescriptor, BufferUsages, BufferView, Device, Extent3d, MapMode, Origin3d, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect,
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;

// ── Dimensions ───────────────────────────────────────────────────────────────

pub const CAPTURE_WIDTH: u32 = 1280;
pub const CAPTURE_HEIGHT: u32 = 720;

// ── Latest-frame slot ─────────────────────────────────────────────────────────
// The render thread overwrites this every frame; the UI thread takes the
// latest copy each repaint. No queueing/backpressure needed — we only ever
// care about the newest frame.

pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

static LATEST_FRAME: Lazy<Mutex<Option<Frame>>> = Lazy::new(|| Mutex::new(None));

/// Called by the render thread after a frame is ready.
pub fn push_frame(width: u32, height: u32, rgba: Vec<u8>) {
    *LATEST_FRAME.lock() = Some(Frame { width, height, rgba });
}

/// Called by the UI thread each repaint. Returns `None` if no new frame has
/// landed since the last call.
pub fn take_latest_frame() -> Option<Frame> {
    LATEST_FRAME.lock().take()
}

pub fn clear_latest_frame() {
    *LATEST_FRAME.lock() = None;
}

// ── Utility ───────────────────────────────────────────────────────────────────

pub fn align_to(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

pub fn readback_buffer_size(width: u32, height: u32) -> u64 {
    (align_to(width * 4, 256) * height) as u64
}

// ── Ping-pong readback buffers ────────────────────────────────────────────────

pub struct ReadbackBuffers {
    pub write_buf: Buffer,
    pub read_buf: Buffer,
    pub device: Arc<Device>,
    pub map_pending: bool,
}

impl ReadbackBuffers {
    pub fn new(device: Arc<Device>, width: u32, height: u32) -> Self {
        let size = readback_buffer_size(width, height);
        let make = |label| {
            device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size,
                usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        Self {
            write_buf: make("readback_write"),
            read_buf: make("readback_read"),
            device,
            map_pending: false,
        }
    }

    /// Kick `map_async` on `read_buf` after submit.
    pub fn kick_map(&mut self) {
        if self.map_pending {
            return;
        }
        self.read_buf.slice(..).map_async(MapMode::Read, |_| {});
        self.map_pending = true;
    }

    /// Block until the GPU copy is done, collect pixels, publish the frame.
    /// Call this on the RENDER thread — it owns the buffers exclusively.
    pub fn blocking_collect_and_push(&mut self, width: u32, height: u32) {
        if !self.map_pending {
            return;
        }

        // Blocks until GPU finishes — fires the map_async callback internally.
        self.device.poll(wgpu::Maintain::Wait);

        let bytes = {
            let view: BufferView = self.read_buf.slice(..).get_mapped_range();
            let pixel_stride = (width * 4) as usize;
            let padded_stride = align_to(width * 4, 256) as usize;
            let mut out = vec![0u8; pixel_stride * height as usize];

            for row in 0..height as usize {
                let src = row * padded_stride;
                let dst = row * pixel_stride;
                out[dst..dst + pixel_stride].copy_from_slice(&view[src..src + pixel_stride]);
            }
            out
            // view dropped here before unmap
        };

        self.read_buf.unmap();
        self.map_pending = false;

        std::mem::swap(&mut self.write_buf, &mut self.read_buf);

        push_frame(width, height, bytes);
    }

    pub fn reset(&mut self) {
        self.map_pending = false;
        clear_latest_frame();
    }
}

// ── GPU copy record ───────────────────────────────────────────────────────────

pub fn record_readback(encoder: &mut wgpu::CommandEncoder, capture_texture: &Texture, write_buf: &Buffer) {
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
            buffer: write_buf,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        Extent3d { width, height, depth_or_array_layers: 1 },
    );
}

// ── Memory locking ────────────────────────────────────────────────────────────

pub fn lock_process_memory() {
    #[cfg(unix)]
    {
        extern "C" {
            fn mlockall(flags: i32) -> i32;
        }
        const MCL_CURRENT: i32 = 1;
        const MCL_FUTURE: i32 = 2;

        let result = unsafe { mlockall(MCL_CURRENT | MCL_FUTURE) };
        if result != 0 {
            eprintln!(
                "mlockall failed ({}): process memory may be swapped. \
                 Try: sudo setcap cap_ipc_lock+ep <path_to_binary>",
                std::io::Error::last_os_error()
            );
        } else {
            println!("mlockall: all process memory locked into RAM");
        }
    }
}
