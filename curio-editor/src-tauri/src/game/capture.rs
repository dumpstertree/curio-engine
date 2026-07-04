use egui_wgpu::wgpu::{Buffer, BufferDescriptor, BufferUsages, BufferView, Device, Extent3d, MapMode, Origin3d, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect};

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, Receiver, SyncSender},
    Arc, Mutex,
};

// ── Dimensions ───────────────────────────────────────────────────────────────

pub const CAPTURE_WIDTH: u32 = 1280;
pub const CAPTURE_HEIGHT: u32 = 720;

// ── Frame channel ─────────────────────────────────────────────────────────────
// The render thread sends completed frames here.
// The stream thread reads and forwards them to JS via Tauri Channel.
// SyncSender with bound=1 means the render thread never blocks waiting for
// the stream thread — if a frame is already waiting it just drops it and
// moves on, keeping the render loop running at full speed.

static FRAME_TX: Mutex<Option<SyncSender<Vec<u8>>>> = Mutex::new(None);

pub fn install_frame_sender(tx: SyncSender<Vec<u8>>) {
    eprintln!("[capture] install_frame_sender called");
    *FRAME_TX.lock().unwrap() = Some(tx);
}

pub fn uninstall_frame_sender() {
    eprintln!("[capture] uninstall_frame_sender called");
    *FRAME_TX.lock().unwrap() = None;
}

/// Called by the render thread after a frame is ready.
/// Sends to the stream thread — drops the frame silently if the channel is
/// full (stream thread hasn't consumed the previous one yet).
pub fn push_frame(bytes: Vec<u8>) {
    if let Ok(guard) = FRAME_TX.lock() {
        if let Some(tx) = guard.as_ref() {
            match tx.try_send(bytes) {
                Ok(()) => eprintln!("[capture] push_frame: sent ok"),
                Err(e) => eprintln!("[capture] push_frame: try_send failed: {:?}", e),
            }
        } else {
            eprintln!("[capture] push_frame: no sender installed");
        }
    } else {
        eprintln!("[capture] push_frame: FRAME_TX lock failed");
    }
}

/// Called by the stream thread to wait for the next frame.
pub fn recv_frame(rx: &Receiver<Vec<u8>>) -> Option<Vec<u8>> {
    rx.recv().ok()
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

    /// Kick map_async on read_buf after submit.
    pub fn kick_map(&mut self) {
        if self.map_pending {
            return;
        }
        self.read_buf.slice(..).map_async(MapMode::Read, |_| {});
        self.map_pending = true;
    }

    /// Block until the GPU copy is done, collect pixels, push to stream thread.
    /// Call this on the RENDER thread — it owns the buffers exclusively.
    pub fn blocking_collect_and_push(&mut self, width: u32, height: u32) {
        if !self.map_pending {
            eprintln!("[capture] blocking_collect_and_push: no map pending, skipping");
            return;
        }
        eprintln!("[capture] blocking_collect_and_push: waiting on poll(Wait)...");

        // Block until GPU finishes — fires the map_async callback internally
        self.device.poll(egui_wgpu::wgpu::MaintainBase::Wait);

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

        // Send to stream thread — non-blocking, drops frame if full
        push_frame(bytes);
    }

    pub fn reset(&mut self) {
        self.map_pending = false;
    }
}

// ── GPU copy record ───────────────────────────────────────────────────────────

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
