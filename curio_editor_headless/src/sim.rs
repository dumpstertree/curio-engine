//! Headless simulation driver for LLM tooling — same contract as the
//! standalone engine (`load_curio` → `window_opened()` → `update()`/`render()`
//! per frame), but with no winit window in front of it:
//!
//! - input events from the scenario go through `Curio::input_button`/
//!   `Curio::input_axis` — exactly the calls the engine's window event
//!   handlers make;
//! - the frame renders into an offscreen texture and the readback is the
//!   editor's proven `capture.rs` pattern (COPY_DST write buffer, `map_async`
//!   on a ping-pong pair, poll, strip the 256-byte row padding);
//! - the ledger is dumped as JSON (`Ledger::serializable()`) plus a
//!   field-level diff against the previous dump, machine-readable by a model;
//! - every frame's `update()` runs under `catch_unwind` — a plugin that
//!   dies mid-run leaves a `HALT:` line and a nonzero exit instead of
//!   bare panic noise.
//!
//! Exit codes: 0 done · 2 bad/missing scenario · 3 no ./test.proj ·
//! 4 no game plugin · 5 plugin died mid-run.

use curio_engine::curio_cabinet::{self, expected_plugin_stem, load_curio, LoadedCurio};
use curio_engine::plugin_loader::lock_process_memory;
use curio_engine::project::{self, Project};

use crate::scenario::{self, InputEvent, Scenario};

use curio_core::{AssetCache, AssetDatabase, AssetPipeline, AxisCode, ButtonCode, ButtonPressed, ComponentState, Curio, CurioCommon, FieldState, File, Gpu, Logger, ObjectState, PluginState, Services, Severity, Vector3};

use time::SysRecordTime;

use egui_wgpu::wgpu::{
    self, Adapter, Backends, Buffer, BufferDescriptor, BufferUsages, BufferView, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Features, Instance, InstanceDescriptor, Limits, Maintain, MapMode, Origin3d, PowerPreference, Queue, RequestAdapterOptions, TexelCopyBufferInfo,
    TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
};

use image::{save_buffer, ExtendedColorType};

use serde_json::{self, Value};

use pollster::FutureExt;

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::{collections::HashMap, path::Path, ptr, sync::Arc};

// ── Readback geometry (capture.rs utility, same padding rule) ───────────────

pub fn align_to(value: u32, alignment: u32) -> u32 {
    return (value + alignment - 1) & !(alignment - 1);
}

pub fn readback_buffer_size(width: u32, height: u32) -> u64 {
    return (align_to(width * 4, 256) * height) as u64;
}

// ── Ledger dump flattening ───────────────────────────────────────────────────
// Flatten the serializable() tree into (object path, field value) pairs so
// consecutive dumps can be diffed cheaply.

pub fn flatten_object(object: &ObjectState, prefix: String) -> Vec<(String, Value)> {
    let mut path = object.object_name.clone();
    if prefix.len() > 0 {
        path = format!("{}/{}", prefix, object.object_name);
    }
    let mut out: Vec<(String, Value)> = vec![];
    for i in 0..object.components.len() {
        let comp: &ComponentState = &object.components[i];
        for j in 0..comp.fields.len() {
            out.push((format!("{}/{}", path, comp.component_name), comp.fields[j].data.clone()));
        }
    }
    for i in 0..object.children.len() {
        let child = flatten_object(&object.children[i], path.clone());
        out.extend(child);
    }
    return out;
}

pub fn flatten_ledger_state(state: &PluginState) -> Vec<(String, Value)> {
    let mut out: Vec<(String, Value)> = vec![];
    for i in 0..state.objects.len() {
        let flat = flatten_object(&state.objects[i], "".to_string());
        out.extend(flat);
    }
    return out;
}

// ── Simulation ───────────────────────────────────────────────────────────────

pub struct Simulation {
    pub scenario_path: String,
    scenario: Scenario,
    event_map: HashMap<u64, Vec<InputEvent>>,
    // both must outlive the Services pointers into them
    logger: Box<Logger>,
    assets: Box<AssetPipeline>,
    services: Option<Box<Services>>,
    // keeps the LoadedCurio (and its open .so) alive
    loaded: Option<LoadedCurio>,
    // all must outlive the Services raw pointers
    device: Option<Arc<Device>>,
    queue: Option<Arc<Queue>>,
    adapter: Option<Arc<Adapter>>,
    capture_texture: Option<Arc<Texture>>,
    write_buf: Option<Buffer>,
    read_buf: Option<Buffer>,
    gpu_ready: bool,
    // the raw pointers the Services (and the plugins) see — set once with
    // the device so no borrow of self is needed when Services is built
    gpu_device_ptr: *const (),
    gpu_queue_ptr: *const (),
    dump_every: u64,
    dump_dir: String,
    capture_dir: String,
    prev_dump: Vec<(String, Value)>,
    halted: bool,
}

impl Simulation {
    pub fn new(path: String) -> Simulation {
        Simulation {
            scenario_path: path,
            scenario: Scenario::default(),
            event_map: HashMap::new(),
            logger: Box::new(Logger::new(Severity::Info)),
            assets: Box::new(AssetPipeline::new(AssetCache::new(100), AssetDatabase::new())),
            services: None,
            loaded: None,
            device: None,
            queue: None,
            adapter: None,
            capture_texture: None,
            write_buf: None,
            read_buf: None,
            gpu_ready: false,
            gpu_device_ptr: ptr::null(),
            gpu_queue_ptr: ptr::null(),
            dump_every: 0,
            dump_dir: "".to_string(),
            capture_dir: "".to_string(),
            prev_dump: vec![],
            halted: false,
        }
    }
}

impl Simulation {
    pub fn run(&mut self) -> i32 {
        // ── scenario ──────────────────────────────────────────────────────
        // File::read returns empty on a missing file — no panic path here
        let bytes = File::read(&self.scenario_path);
        if bytes.len() == 0 {
            println!("HEADLESS: no scenario found at {}", self.scenario_path);
            return 2;
        }
        let Ok(scenario) = serde_yaml::from_slice::<Scenario>(&bytes) else {
            println!("HEADLESS: scenario is not valid YAML for this runner");
            return 2;
        };
        self.scenario = scenario;
        self.event_map = self.scenario.events_by_frame();

        if self.scenario.steps == 0 {
            println!("HEADLESS: steps missing in scenario");
            return 2;
        }

        // ── project ───────────────────────────────────────────────────────
        let Ok(project) = catch_unwind(AssertUnwindSafe(|| Project::load_local())) else {
            println!("HEADLESS: no ./test.proj in the working directory");
            return 3;
        };

        // ── gpu attempt ───────────────────────────────────────────────────
        let width = if self.scenario.capture_width > 0 { self.scenario.capture_width } else { 640 };
        let height = if self.scenario.capture_height > 0 { self.scenario.capture_height } else { 360 };
        self.setup_gpu(width, height);
        if !self.gpu_ready {
            println!("HEADLESS: no adapter — screenshots will be skipped, the sim still runs");
        }

        // ── services (Boxed for the stable address the plugins cast) ───────
        self.services = Some(Box::new(Services {
            assets: self.assets.as_mut() as *mut AssetPipeline,
            logger: self.logger.as_mut() as *mut Logger,
            gpu: Gpu {
                device: if self.gpu_ready { self.gpu_device_ptr } else { ptr::null() },
                queue: if self.gpu_ready { self.gpu_queue_ptr } else { ptr::null() },
                // null on purpose — the game's swapchain→capture copy step
                // self-skips, we do our own readback below
                capture_texture: ptr::null(),
                capture_width: width,
                capture_height: height,
            },
        }));

        let services_ptr: *const Services = self.services.as_deref().unwrap();
        Services::set(services_ptr);

        // ── load the game plugin ──────────────────────────────────────────
        lock_process_memory();

        let path = format!("{}/target/release/", project.project_path);
        let stem = expected_plugin_stem(project.project_path.as_str());
        match catch_unwind(AssertUnwindSafe(|| load_curio(Path::new(&path), services_ptr, stem.as_deref()))) {
            Ok(loaded) => {
                self.loaded = Some(loaded);
            }
            Err(payload) => {
                let msg = payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_else(|| "panic with no message".to_string());
                println!("HEADLESS: no game plugin could be loaded: {msg}");
                return 4;
            }
        }

        // the plugin's open gate — same call the engine/editor make
        let Some(loaded) = &mut self.loaded else {
            return 4;
        };
        loaded.curio.window_opened();

        // ── startup injection ─────────────────────────────────────────────
        self.inject_time();

        // ── frame loop ────────────────────────────────────────────────────
        self.dump_dir = if self.scenario.dump_dir.len() == 0 { "ledger".to_string() } else { self.scenario.dump_dir.clone() };
        self.capture_dir = if self.scenario.capture_dir.len() == 0 { "shots".to_string() } else { self.scenario.capture_dir.clone() };
        self.dump_every = self.scenario.dump_every;

        for frame in 0..self.scenario.steps {
            self.forward_events(frame);

            self.update_frame(frame);
            if self.halted {
                return 5;
            }

            if self.dump_every > 0 && frame % self.dump_every == 0 {
                self.dump_ledger(frame);
            }

            if self.should_capture(frame) {
                if self.gpu_ready {
                    self.capture_frame(frame);
                } else {
                    println!("shot skipped: {} (no adapter)", frame);
                }
            }
        }

        println!("HEADLESS DONE: steps={}", self.scenario.steps);
        return 0;
    }

    // ── gpu setup ─────────────────────────────────────────────────────────
    // Follows the editor's proven headless `game_runner.rs` setup:
    // `compatible_surface: None`, same pinned features, offscreen
    // capture texture + ping-pong readback buffers (`capture.rs`).

    pub fn setup_gpu(&mut self, width: u32, height: u32) {
        let instance = Instance::new(&InstanceDescriptor { backends: Backends::all(), ..Default::default() });

        let Ok(adapter_opt) = catch_unwind(AssertUnwindSafe(|| {
            instance
                .request_adapter(&RequestAdapterOptions {
                    power_preference: PowerPreference::HighPerformance,
                    compatible_surface: None,
                    force_fallback_adapter: false,
                })
                .block_on()
        })) else {
            return;
        };
        let Some(adapter) = adapter_opt else {
            return;
        };

        let Ok(device_result) = catch_unwind(AssertUnwindSafe(|| {
            adapter
                .request_device(
                    &DeviceDescriptor {
                        label: Some("curio_headless_device"),
                        required_features: Features::POLYGON_MODE_LINE | Features::BUFFER_BINDING_ARRAY,
                        required_limits: Limits::default(),
                        memory_hints: Default::default(),
                    },
                    None,
                )
                .block_on()
        })) else {
            return;
        };
        let Ok(device_queue) = device_result else {
            println!("HEADLESS: adapter found but device request failed");
            return;
        };
        let device = Arc::new(device_queue.0);
        let queue = Arc::new(device_queue.1);

        let capture_texture = Arc::new(device.create_texture(&TextureDescriptor {
            label: Some("headless_capture_texture"),
            size: Extent3d { width: width, height: height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC | TextureUsages::COPY_DST,
            view_formats: &[],
        }));

        let size = readback_buffer_size(width, height);
        let write_buf = device.create_buffer(&BufferDescriptor {
            label: Some("headless_write"),
            size: size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let read_buf = device.create_buffer(&BufferDescriptor {
            label: Some("headless_read"),
            size: size,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.gpu_device_ptr = Arc::as_ptr(&device) as *const ();
        self.gpu_queue_ptr = Arc::as_ptr(&queue) as *const ();
        self.device = Some(device);
        self.queue = Some(queue);
        self.adapter = Some(Arc::new(adapter));
        self.capture_texture = Some(capture_texture);
        self.write_buf = Some(write_buf);
        self.read_buf = Some(read_buf);
        self.gpu_ready = true;
    }

    // ── one simulation step ───────────────────────────────────────────────
    pub fn update_frame(&mut self, frame: u64) {
        let Some(loaded) = &mut self.loaded else {
            return;
        };
        let survived = catch_unwind(AssertUnwindSafe(|| loaded.curio.update()));
        if survived.is_err() {
            self.halted = true;
            println!("HALT: the game plugin panicked during frame {} — see the CURIO PANIC banner", frame);
        }
    }

    // ── input forwarding ──────────────────────────────────────────────────
    pub fn forward_events(&mut self, frame: u64) {
        if let Some(events) = self.event_map.get(&frame) {
            let Some(loaded) = &mut self.loaded else {
                return;
            };
            for i in 0..events.len() {
                let event: &InputEvent = &events[i];
                if let Some(code) = &event.button {
                    let mut state = ButtonPressed::Down;
                    if let Some(pressed) = &event.pressed {
                        state = *pressed;
                    }
                    loaded.curio.input_button(*code, state);
                }
                if let Some(code) = &event.axis {
                    loaded
                        .curio
                        .input_axis(*code, Vector3::new(event.x, event.y, 0.0));
                }
            }
        }
    }

    // ── time injection ────────────────────────────────────────────────────
    // `Ledger::write::<SysRecordTime>` is a typed write — when the game came
    // from a dlopen'd .so the plugin's `SysRecordTime` TypeId differs from
    // ours and the downcast panics; guarded here and reported. The generic
    // core-side fix is in todo.md.
    pub fn inject_time(&mut self) {
        let Some(rate) = &self.scenario.time_target_frame_rate else {
            return;
        };
        let Some(loaded) = &mut self.loaded else {
            return;
        };
        let mut applied = 0;
        for i in 0..loaded.curio.ledgers.len() {
            let survived = catch_unwind(AssertUnwindSafe(|| {
                loaded.curio.ledgers[i].write::<SysRecordTime>(|t| {
                    t.target_frame_rate = *rate;
                })
            }));
            if !survived.is_err() {
                applied = applied + 1;
            }
        }
        if applied == 0 {
            println!("HEADLESS: time injection skipped — record type identity differs across the dlopen boundary (see todo.md)");
        }
    }

    // ── ledger dump + diff ────────────────────────────────────────────────
    pub fn dump_ledger(&mut self, frame: u64) {
        let Some(loaded) = &self.loaded else {
            return;
        };
        if loaded.curio.ledgers.len() == 0 {
            return;
        }
        let (id, state) = loaded.curio.ledgers[0].serializable();
        let Ok(json) = serde_json::to_string_pretty(&(&id, &state)) else {
            return;
        };
        let path = format!("{}/ledger-{:06}.json", self.dump_dir, frame);
        if !File::write(&path, json.as_bytes()) {
            println!("HEADLESS: ledger dump failed at {}", path);
            return;
        }
        println!("ledger: {}", path);

        let current = flatten_ledger_state(&state);
        let mut changed: Vec<(String, Value)> = vec![];
        for i in 0..current.len() {
            let entry: &(String, Value) = &current[i];
            let mut same = false;
            for j in 0..self.prev_dump.len() {
                let old: &(String, Value) = &self.prev_dump[j];
                if old.0 == entry.0 && old.1 == entry.1 {
                    same = true;
                }
            }
            if !same {
                changed.push((entry.0.clone(), entry.1.clone()));
            }
        }
        if changed.len() > 0 {
            let Ok(diff_json) = serde_json::to_string_pretty(&changed) else {
                self.prev_dump = current;
                return;
            };
            let diff_path = format!("{}/diff-{:06}.json", self.dump_dir, frame);
            if File::write(&diff_path, diff_json.as_bytes()) {
                println!("diff: {}", diff_path);
            }
        }
        self.prev_dump = current;
    }

    // ── render one frame and save it ──────────────────────────────────────
    pub fn capture_frame(&mut self, frame: u64) -> bool {
        let Some(loaded) = &mut self.loaded else {
            return false;
        };
        let Some(device) = &self.device else {
            return false;
        };
        let Some(queue) = &self.queue else {
            return false;
        };
        let Some(texture) = &self.capture_texture else {
            return false;
        };
        let Some(write_buf) = &mut self.write_buf else {
            return false;
        };
        let Some(read_buf) = &mut self.read_buf else {
            return false;
        };

        let width = texture.width();
        let height = texture.height();

        let output_view = texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: Some("headless_frame") });

        loaded.curio.render(texture, &output_view, &mut encoder);

        let bytes_per_row = align_to(width * 4, 256);
        encoder.copy_texture_to_buffer(
            TexelCopyTextureInfo {
                texture: texture,
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

        queue.submit(std::iter::once(encoder.finish()));

        // the capture.rs readback: map_async then poll, strip the 256-aligned
        // row padding, unmap and ping-pong the buffers
        read_buf.slice(..).map_async(MapMode::Read, |_| {});
        device.poll(Maintain::Wait);

        let bytes = {
            let view: BufferView = read_buf.slice(..).get_mapped_range();
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

        read_buf.unmap();
        std::mem::swap(&mut self.write_buf, &mut self.read_buf);

        let path = format!("{}/shot-{:06}.png", self.capture_dir, frame);
        match save_buffer(&path, &bytes, width, height, ExtendedColorType::Rgba8) {
            Ok(_) => {
                println!("shot: {}", path);
                return true;
            }
            Err(e) => {
                println!("HEADLESS: screenshot save failed at {}: {:?}", path, e);
                return false;
            }
        }
    }

    // ── is this a capture frame? ──────────────────────────────────────────
    pub fn should_capture(&mut self, frame: u64) -> bool {
        for i in 0..self.scenario.capture_frames.len() {
            if self.scenario.capture_frames[i] == frame {
                return true;
            }
        }
        return false;
    }
}
