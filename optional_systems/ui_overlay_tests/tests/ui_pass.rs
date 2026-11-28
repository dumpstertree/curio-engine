//! UiOverlayPass on a real device (lavapipe is enough). A test binary of its own, because the pass reads the
//! gpu from the process-wide Services and the habit tests in src/ install one with no device.

use curio_core::{AssetCache, AssetDatabase, AssetDatabaseListing, AssetPipeline, Assets, Color, CurioNetwork, CurioNetworkParticipant, Gpu, GraphicsMapping, Ledger, Logger, NetworkModes, Services, Severity, Vector2};
use egui_wgpu::wgpu::{
    Backends, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Features, Instance, InstanceDescriptor, Limits, LoadOp, Maintain, MapMode, Operations, Origin3d, PowerPreference, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, StoreOp, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
};
use pollster::FutureExt;
use std::sync::{Mutex, MutexGuard, OnceLock};
use ui_overlay::{SysRecordUi, UiDrawCall, UiKind, UiOverlayPass, UiRect};

const WIDTH: u32 = 128;
const HEIGHT: u32 = 32;

// None when the machine has no adapter at all
fn device() -> Option<(&'static Device, &'static Queue)> {
    let instance = Instance::new(&InstanceDescriptor { backends: Backends::all(), ..Default::default() });
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .block_on()?;
    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: Some("ui pass test"),
                required_features: Features::empty(),
                required_limits: Limits::default(),
                memory_hints: Default::default(),
            },
            None,
        )
        .block_on()
        .ok()?;
    Some((Box::leak(Box::new(device)), Box::leak(Box::new(queue))))
}

// the shared Services (assets, logger) is not thread-safe, so the tests here take turns
fn serial() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

// one device and one Services for the whole binary, since the tests run on parallel threads
fn gpu() -> Option<(&'static Device, &'static Queue)> {
    static GPU: OnceLock<Option<(&'static Device, &'static Queue)>> = OnceLock::new();
    *GPU.get_or_init(|| {
        let (device, queue) = device()?;
        stand_up_services(device, queue);
        Some((device, queue))
    })
}

fn stand_up_services(device: &'static Device, queue: &'static Queue) {
    let services: *mut Services = Box::into_raw(Box::new(Services {
        logger: Box::into_raw(Box::new(Logger::new(Severity::Info))),
        assets: Box::into_raw(Box::new(AssetPipeline::new(AssetCache::new(10), AssetDatabase::new()))),
        gpu: Gpu {
            device: device as *const Device as *const (),
            queue: queue as *const Queue as *const (),
            capture_texture: std::ptr::null(),
            capture_width: WIDTH,
            capture_height: HEIGHT,
        },
    }));
    Services::set(services);
}

fn target(device: &Device) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("ui pass target"),
        size: Extent3d { width: WIDTH, height: HEIGHT, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

fn screen(draw_calls: Vec<UiDrawCall>) -> Vec<Ledger> {
    let network = CurioNetwork::new(vec![CurioNetworkParticipant::new(0, NetworkModes::LocalHost)], 0);
    let mut ledger = Ledger::new(network);
    ledger.write::<SysRecordUi>(|screen| {
        screen.viewport = Vector2::new(WIDTH as f32, HEIGHT as f32);
        screen.draw_calls = draw_calls;
    });
    vec![ledger]
}

// a 2x2 png registered under `uid`: `top` on the upper row, `bottom` on the lower
fn register_texture(uid: i16, top: [u8; 4], bottom: [u8; 4]) {
    let pixels = image::RgbaImage::from_fn(2, 2, |_, y| image::Rgba(if y == 0 { top } else { bottom }));
    let mut png = Vec::new();
    image::DynamicImage::ImageRgba8(pixels)
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageOutputFormat::Png)
        .unwrap();
    Assets::append_listing(format!("ui test texture {}", uid), uid, AssetDatabaseListing::Embedded(png));
}

fn image_call(rect: UiRect, color: Color, uid: i16, order: u32) -> UiDrawCall {
    UiDrawCall::new(rect, color, UiKind::Image, 0.0, String::new(), 12.0, order as i32, order).with_image(Some(uid))
}

fn pixel(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let at = ((y * WIDTH + x) * 4) as usize;
    [bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]
}

// counts the pixels of one frame that are no longer the grey the corner pixel still has
fn changed_pixels(pass: &mut UiOverlayPass, ledgers: &mut Vec<Ledger>, device: &Device, queue: &Queue, target: &Texture) -> usize {
    let bytes = render(pass, ledgers, device, queue, target);
    let grey = bytes[0..4].to_vec();
    bytes
        .chunks(4)
        .filter(|pixel| *pixel != grey.as_slice())
        .count()
}

// renders one frame of the pass over a mid-grey target and reads it back as rgba8
fn render(pass: &mut UiOverlayPass, ledgers: &mut Vec<Ledger>, device: &Device, queue: &Queue, target: &Texture) -> Vec<u8> {
    let view = target.create_view(&TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: Some("ui pass test") });
    {
        let _clear = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("clear to grey"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(egui_wgpu::wgpu::Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
    let mappings = vec![GraphicsMapping::new(Vector2::zero(), Vector2::one())];
    pass.draw_ui_overlays(ledgers, &mut encoder, &view, &mappings);

    // WIDTH * 4 is a multiple of 256, so rows carry no padding
    let readback = device.create_buffer(&BufferDescriptor {
        label: Some("ui pass readback"),
        size: (WIDTH * HEIGHT * 4) as u64,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        TexelCopyTextureInfo {
            texture: target,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        TexelCopyBufferInfo {
            buffer: &readback,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(WIDTH * 4),
                rows_per_image: Some(HEIGHT),
            },
        },
        Extent3d { width: WIDTH, height: HEIGHT, depth_or_array_layers: 1 },
    );
    queue.submit(std::iter::once(encoder.finish()));
    readback.slice(..).map_async(MapMode::Read, |_| {});
    device.poll(Maintain::Wait);
    let bytes = readback.slice(..).get_mapped_range().to_vec();
    readback.unmap();
    bytes
}

#[test]
fn a_screen_of_only_text_labels_paints_on_its_first_frame() {
    let _serial = serial();
    let Some((device, queue)) = gpu() else {
        println!("no gpu adapter: the ui pass test is skipped");
        return;
    };
    let target = target(device);
    let mut ledgers = screen(vec![UiDrawCall::new(UiRect::new(4.0, 4.0, 120.0, 24.0), Color::white(), UiKind::Text, 0.0, "HELLO".to_string(), 16.0, 1, 0)]);
    let mut pass = UiOverlayPass::new();

    // the corner pixel is the grey the glyphs are compared against, so it must stay clear of them
    let first = changed_pixels(&mut pass, &mut ledgers, device, queue, &target);
    assert!(first > 0, "a text-only screen painted nothing on its first frame");
    let second = changed_pixels(&mut pass, &mut ledgers, device, queue, &target);
    assert_eq!(first, second, "the label changed between the first and second frame");
}

const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const WHITE: [u8; 4] = [255, 255, 255, 255];

#[test]
fn an_image_leaf_paints_its_texture_stretched_and_tinted() {
    let _serial = serial();
    let Some((device, queue)) = gpu() else {
        println!("no gpu adapter: the ui pass test is skipped");
        return;
    };
    register_texture(9101, RED, BLUE);
    let target = target(device);
    let mut pass = UiOverlayPass::new();

    let full = UiRect::new(0.0, 0.0, WIDTH as f32, HEIGHT as f32);
    let mut ledgers = screen(vec![image_call(full, Color::white(), 9101, 0)]);
    let bytes = render(&mut pass, &mut ledgers, device, queue, &target);
    assert_eq!(pixel(&bytes, 64, 2), RED, "top of the box is not the texture's top row");
    assert_eq!(pixel(&bytes, 64, 29), BLUE, "bottom of the box is not the texture's bottom row");

    // a yellow tint keeps red and drops blue
    let mut ledgers = screen(vec![image_call(full, "#ffff00ff".parse().unwrap_or(Color::white()), 9101, 0)]);
    let bytes = render(&mut pass, &mut ledgers, device, queue, &target);
    assert_eq!(pixel(&bytes, 64, 2), RED);
    assert_eq!(pixel(&bytes, 64, 29), [0, 0, 0, 255]);
}

#[test]
fn a_missing_texture_paints_the_flat_quad_and_logs_once() {
    let _serial = serial();
    let Some((device, queue)) = gpu() else {
        println!("no gpu adapter: the ui pass test is skipped");
        return;
    };
    let target = target(device);
    let mut pass = UiOverlayPass::new();
    let mut ledgers = screen(vec![image_call(UiRect::new(0.0, 0.0, WIDTH as f32, HEIGHT as f32), Color::white(), 9199, 0)]);
    let first = render(&mut pass, &mut ledgers, device, queue, &target);
    let second = render(&mut pass, &mut ledgers, device, queue, &target);
    assert_eq!((pixel(&first, 64, 16), pixel(&second, 64, 16)), (WHITE, WHITE));
    let warnings = Services::get()
        .logger()
        .drain()
        .into_iter()
        .filter(|(_, line)| line.contains("image texture 9199"))
        .count();
    assert_eq!(warnings, 1, "the missing texture should be logged exactly once");
}

#[test]
fn images_and_boxes_keep_their_paint_order() {
    let _serial = serial();
    let Some((device, queue)) = gpu() else {
        println!("no gpu adapter: the ui pass test is skipped");
        return;
    };
    register_texture(9102, RED, RED);
    register_texture(9103, BLUE, BLUE);
    let target = target(device);
    let mut pass = UiOverlayPass::new();
    // red image, blue image over its middle, red image over that, then a white box: four switches of texture
    let mut ledgers = screen(vec![
        image_call(UiRect::new(0.0, 0.0, 128.0, 32.0), Color::white(), 9102, 0),
        image_call(UiRect::new(32.0, 0.0, 64.0, 32.0), Color::white(), 9103, 1),
        image_call(UiRect::new(56.0, 8.0, 16.0, 16.0), Color::white(), 9102, 2),
        UiDrawCall::new(UiRect::new(100.0, 0.0, 28.0, 32.0), Color::white(), UiKind::Block, 0.0, String::new(), 12.0, 3, 3),
    ]);
    let bytes = render(&mut pass, &mut ledgers, device, queue, &target);
    assert_eq!(pixel(&bytes, 8, 16), RED);
    assert_eq!(pixel(&bytes, 40, 16), BLUE, "the second image should cover the first");
    assert_eq!(pixel(&bytes, 64, 16), RED, "the third image should cover the second");
    assert_eq!(pixel(&bytes, 110, 16), WHITE, "the box should cover the images");
}
