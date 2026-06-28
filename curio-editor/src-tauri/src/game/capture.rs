use egui_wgpu::wgpu::{Buffer, BufferDescriptor, BufferUsages, BufferView, Device, Extent3d, MapMode, Origin3d, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect};

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

// ── Dimensions ───────────────────────────────────────────────────────────────

pub const CAPTURE_WIDTH: u32 = 1280;
pub const CAPTURE_HEIGHT: u32 = 720;

// ── Double-buffered frame store (CPU side) ────────────────────────────────────

static FRAME_BUFFERS: Lazy<[Arc<RwLock<Vec<u8>>>; 2]> = Lazy::new(|| {
    let size = (CAPTURE_WIDTH * CAPTURE_HEIGHT * 4) as usize;
    [Arc::new(RwLock::new(vec![0u8; size])), Arc::new(RwLock::new(vec![0u8; size]))]
});

static WRITE_IDX: AtomicUsize = AtomicUsize::new(0);
static FRAME_READY: AtomicBool = AtomicBool::new(false);

// ── Utility ───────────────────────────────────────────────────────────────────

pub fn align_to(value: u32, alignment: u32) -> u32 {
    (value + alignment - 1) & !(alignment - 1)
}

pub fn readback_buffer_size(width: u32, height: u32) -> u64 {
    (align_to(width * 4, 256) * height) as u64
}

// ── Ping-pong readback buffers ────────────────────────────────────────────────
// Two GPU readback buffers. While the GPU writes into one, the CPU reads
// the other. They are never in the same state at the same time, so we
// never submit into a mapped buffer.

pub struct ReadbackBuffers {
    // Buffer the GPU is currently copying into
    pub write_buf: Buffer,
    // Buffer the CPU is currently reading / waiting to read
    pub read_buf: Buffer,
    // True when read_buf has an in-flight map_async
    pub map_pending: bool,
    // True when read_buf is mapped and ready to collect
    pub map_ready: Arc<AtomicBool>,
}

impl ReadbackBuffers {
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
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
            map_pending: false,
            map_ready: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Called after queue.submit(). Kicks an async map on read_buf if one
    /// isn't already in flight. Returns immediately.
    pub fn kick_map(&mut self) {
        if self.map_pending {
            return;
        }
        let flag = self.map_ready.clone();
        self.read_buf
            .slice(..)
            .map_async(MapMode::Read, move |result| {
                if result.is_ok() {
                    flag.store(true, Ordering::Release);
                }
            });
        self.map_pending = true;
    }

    /// Called at the start of each frame after poll(Poll).
    /// If read_buf is mapped, copies pixels into the frame store and unmaps.
    /// Then swaps write_buf and read_buf so the GPU writes into the freshly
    /// unmapped buffer next frame.
    pub fn try_collect(&mut self, width: u32, height: u32) {
        if !self.map_pending {
            return;
        }
        if !self.map_ready.load(Ordering::Acquire) {
            return;
        }

        // Collect pixels from read_buf
        {
            let view: BufferView = self.read_buf.slice(..).get_mapped_range();
            let pixel_stride = (width * 4) as usize;
            let padded_stride = align_to(width * 4, 256) as usize;

            let write_idx = WRITE_IDX.load(Ordering::Acquire);
            let mut buf = FRAME_BUFFERS[write_idx].write();

            let expected = pixel_stride * height as usize;
            if buf.len() != expected {
                buf.resize(expected, 0);
            }

            for row in 0..height as usize {
                let src = row * padded_stride;
                let dst = row * pixel_stride;
                buf[dst..dst + pixel_stride].copy_from_slice(&view[src..src + pixel_stride]);
            }
            // view dropped here before unmap
        }

        self.read_buf.unmap();
        self.map_ready.store(false, Ordering::Release);
        self.map_pending = false;

        // Swap — read_buf (now unmapped) becomes the new write target,
        // write_buf (freshly submitted into) becomes the new read target
        std::mem::swap(&mut self.write_buf, &mut self.read_buf);

        WRITE_IDX.fetch_xor(1, Ordering::AcqRel);
        FRAME_READY.store(true, Ordering::Release);
    }

    /// Reset state when stopping or resizing
    pub fn reset(&mut self) {
        self.map_pending = false;
        self.map_ready.store(false, Ordering::Release);
    }
}

// ── Readback record (called before submit) ────────────────────────────────────

/// Records the GPU → CPU copy of `capture_texture` into `write_buf`.
/// Must be called BEFORE queue.submit() and write_buf must NOT be mapped.
pub fn record_readback(encoder: &mut egui_wgpu::wgpu::CommandEncoder, capture_texture: &Texture, write_buf: &Buffer) {
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

// ── Read path (Tauri command thread) ─────────────────────────────────────────

pub fn take_frame() -> Option<Vec<u8>> {
    if !FRAME_READY.swap(false, Ordering::AcqRel) {
        return None;
    }
    let read_idx = WRITE_IDX.load(Ordering::Acquire);
    Some(FRAME_BUFFERS[read_idx].read().clone())
}

// ── Memory locking ────────────────────────────────────────────────────────────

pub fn lock_process_memory() {
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
