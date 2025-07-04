use crate::system_adapters::adapter_system_gpu::SYS_GPU;
use crate::texture;

use crate::Window::CameraState::CameraState;
use wgpu::util::DeviceExt;
use wgpu::BindGroup;
use wgpu::Buffer;

pub struct State {
    // pub box_queue: Box<wgpu::Queue>,
    // pub box_device: Box<wgpu::Device>,
    // pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    // pub window: Arc<Window>,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: Buffer,
    pub camera_bind_group: BindGroup,
    pub depth_texture: texture::Texture,
}

impl State {
    pub async fn new() -> State {
        let guard_sys_gpu = SYS_GPU.lock().unwrap();
        //
        let Some(device) = &guard_sys_gpu.device else {
            panic!();
        };
        let Some(queue) = &guard_sys_gpu.queue else {
            panic!();
        };
        let Some(adapter) = &guard_sys_gpu.adapter else {
            panic!();
        };
        let Some(surface) = &guard_sys_gpu.surface else {
            panic!();
        };
        let Some(window) = &guard_sys_gpu.window else {
            panic!();
        };

        let size = window.inner_size();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let mut config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: 1920,
            height: 1080,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let camera = CameraState::default();
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // if width > 0 && height > 0 {
        config.width = 1920;
        config.height = 1080;
        surface.configure(device, &config);
        // is_surface_configured = true;

        //Make sure you update the depth_texture after you update config. If you don't, your program will crash as the depth_texture will be a different size than the surface texture.
        // state.depth_texture = super::super::texture::Texture::create_depth_texture(&device, &state.config, "depth_texture");

        State {
            // box_device: Box::new(device),
            // box_queue: Box::new(queue),
            // surface,
            config,
            is_surface_configured: true,
            // window: window,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            depth_texture,
        }
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &CameraState) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
