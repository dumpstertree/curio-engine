//! UiOverlayPass on a real device (lavapipe is enough). A test binary of its own, because the pass reads the
//! gpu from the process-wide Services and the habit tests in src/ install one with no device.

use curio_core::{AssetCache, AssetDatabase, AssetPipeline, Color, CurioNetwork, CurioNetworkParticipant, Gpu, GraphicsMapping, Ledger, Logger, NetworkModes, Services, Severity, Vector2};
use egui_wgpu::wgpu::{
    Backends, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Features, Instance, InstanceDescriptor, Limits, LoadOp, Maintain, MapMode, Operations, Origin3d, PowerPreference, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, StoreOp, TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
};
use pollster::FutureExt;
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

// renders one frame of the pass over a mid-grey target and counts the pixels that are no longer grey
fn changed_pixels(pass: &mut UiOverlayPass, ledgers: &mut Vec<Ledger>, device: &Device, queue: &Queue, target: &Texture) -> usize {
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
    let changed = {
        let bytes = readback.slice(..).get_mapped_range();
        let grey = bytes[0..4].to_vec();
        bytes
            .chunks(4)
            .filter(|pixel| *pixel != grey.as_slice())
            .count()
    };
    readback.unmap();
    changed
}

#[test]
fn a_screen_of_only_text_labels_paints_on_its_first_frame() {
    let Some((device, queue)) = device() else {
        println!("no gpu adapter: the ui pass test is skipped");
        return;
    };
    stand_up_services(device, queue);
    let target = device.create_texture(&TextureDescriptor {
        label: Some("ui pass target"),
        size: Extent3d { width: WIDTH, height: HEIGHT, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let network = CurioNetwork::new(vec![CurioNetworkParticipant::new(0, NetworkModes::LocalHost)], 0);
    let mut ledger = Ledger::new(network);
    ledger.write::<SysRecordUi>(|screen| {
        screen.viewport = Vector2::new(WIDTH as f32, HEIGHT as f32);
        screen.draw_calls = vec![UiDrawCall::new(UiRect::new(4.0, 4.0, 120.0, 24.0), Color::white(), UiKind::Text, 0.0, "HELLO".to_string(), 16.0, 1, 0)];
    });
    let mut ledgers = vec![ledger];
    let mut pass = UiOverlayPass::new();

    // the corner pixel is the grey the glyphs are compared against, so it must stay clear of them
    let first = changed_pixels(&mut pass, &mut ledgers, device, queue, &target);
    assert!(first > 0, "a text-only screen painted nothing on its first frame");
    let second = changed_pixels(&mut pass, &mut ledgers, device, queue, &target);
    assert_eq!(first, second, "the label changed between the first and second frame");
}
