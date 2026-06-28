use std::sync::Arc;

use camera::SysRecordCamera;
use curio_core::{
    Color, GraphicsMapping, Ledger, Matrix4x4, Quaternion, TextureAsset, Vector2, Vector3,
    io::asset_loader::{ASSET_UID_SHADER_UNLIT, AssetLoader},
    services,
};
use egui_wgpu::wgpu::{AddressMode, BindGroup, BindGroupLayout, CompareFunction, Device, Extent3d, FilterMode, RenderPass, RenderPassDepthStencilAttachment, SamplerDescriptor, TextureDescriptor, TextureDimension, TextureUsages, TextureViewDescriptor};
use ext_rendering::{
    DrawCall, Material, Mesh, SysRecordRendering,
    data::{material::ShaderDesc, mesh::Vertex},
};
use skybox::{SkyboxTypes, SysRecordSkybox};

use crate::{camera_rendering_components::CameraRenderingComponents, render_feature_3ds::render_feature_draw_mesh::RenderFeatureDrawMesh, shadow_system::ShadowSystem};

pub trait RenderFeature3D {
    fn render(&mut self, ledger: &mut Ledger, render_pass: &mut RenderPass, camera: &CameraRenderingComponents, camera_index: usize, shadow_system_bind_group_layout: &BindGroupLayout, shadow_system_bind_group: &BindGroup);
    fn clear(&mut self, ledger: &mut Ledger);
}

pub struct RenderFeature3DHelper {
    camera_rendering: CameraRenderingComponents,
    features: Vec<Box<dyn RenderFeature3D>>,
    skybox_mesh: Arc<Mesh>,
}
impl RenderFeature3DHelper {
    pub fn new() -> RenderFeature3DHelper {
        println!("3d");

        RenderFeature3DHelper {
            camera_rendering: CameraRenderingComponents::new(1),
            features: vec![RenderFeatureDrawMesh::new()],
            skybox_mesh: Arc::new(skybox_mesh()),
        }
    }
    pub fn set_graphics_mappings(&mut self, graphics_mappings: &[GraphicsMapping]) {
        self.camera_rendering = CameraRenderingComponents::new(graphics_mappings.len());
    }
    fn create_depth_texture(device: &Device, width: u32, height: u32) -> TextureAsset {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("depth_texture"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureAsset::DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        TextureAsset { texture, view, sampler }
    }
    pub fn draw_3d_features(&mut self, graphics_mappings: &mut Vec<GraphicsMapping>, ledger: &mut Vec<Ledger>, encoder: &mut egui_wgpu::wgpu::CommandEncoder, target_view: &mut egui_wgpu::wgpu::TextureView, shadow_system: &ShadowSystem) {
        let s = services();
        // generate a render pass for this instance
        // let depth = s.gpu.depth();
        let device = s.gpu.device();
        let depth = Self::create_depth_texture(device, 1280, 720);

        //
        let state_skybox = ledger[0].read::<SysRecordSkybox>();
        let clear_color = match &state_skybox.skybox {
            SkyboxTypes::CubeMap(m) => {
                // add draw call
                ledger[0].write::<SysRecordRendering>(|x| {
                    // x.draw_calls.push( );

                    let s = AssetLoader::load_asset::<ShaderDesc>(&ASSET_UID_SHADER_UNLIT);
                    let mut mat = Material::new("skybox", s, false);
                    mat.set_texture_with_label(Some(m.clone()), "diffuse");
                    mat.finalize();

                    // println!("{},{}", m.texture.size().width, m.texture.size().height);

                    let material = Arc::new(mat);

                    x.draw_calls
                        .push(DrawCall::draw_mesh_single(self.skybox_mesh.clone(), material, Matrix4x4::new(Vector3::zero(), Quaternion::identity(), Vector3::one()), Color::clear(), false));
                });
                egui_wgpu::wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
            }
            SkyboxTypes::Color(c) => egui_wgpu::wgpu::Color {
                r: c.as_r_01() as f64,
                g: c.as_g_01() as f64,
                b: c.as_b_01() as f64,
                a: c.as_a_01() as f64,
            },
            _ => egui_wgpu::wgpu::Color { r: 1.0, g: 0.1, b: 0.1, a: 1.0 },
        };

        // let clear_color =
        //
        let mut render_pass = encoder.begin_render_pass(&egui_wgpu::wgpu::RenderPassDescriptor {
            label: Some("3D render pass"),
            color_attachments: &[Some(egui_wgpu::wgpu::RenderPassColorAttachment {
                view: target_view, // <-- use the texture view
                resolve_target: None,
                ops: egui_wgpu::wgpu::Operations {
                    load: egui_wgpu::wgpu::LoadOp::Clear(clear_color),
                    store: egui_wgpu::wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Self::get_depth_attatchment(&depth),
            timestamp_writes: None,
            occlusion_query_set: None, // keep or add depth if you use it
        });

        // iterate over each camera in state
        for i in 0..graphics_mappings.len() {
            //
            let ledger = ledger.get_mut(i).unwrap();

            let state_camera = ledger.read::<SysRecordCamera>();

            // get camera data
            let cur_camera_snapshot = &state_camera.cameras;
            let cur_graphics_mapping = &graphics_mappings[i];

            let width = services().gpu.capture_width;
            let height = services().gpu.capture_height;

            // create viewport bounds
            let viewport = Viewport::new(Vector2::new(width as f32, height as f32), cur_graphics_mapping.viewport_min, cur_graphics_mapping.viewport_max);

            // set the viewport based on mapping
            render_pass.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height, 0.0, 1.0);

            // calculate camera binding values
            let camera_uniform = cur_camera_snapshot.get_uniform(viewport.width as i32, viewport.height as i32);

            //
            self.camera_rendering.update(i, &camera_uniform);
            let camera_rendering = &self.camera_rendering;

            // render features
            for feature in self.features.iter_mut() {
                feature.render(ledger, &mut render_pass, camera_rendering, i, &shadow_system.bind_group_layout, shadow_system.sampling_bind_group_for(i).unwrap());
            }
        }

        // cleanup
        for feature in self.features.iter_mut() {
            for ledger in &mut *ledger {
                feature.clear(ledger);
            }
        }
    }
    // get
    fn get_depth_attatchment<'a>(depth: &'a TextureAsset) -> Option<RenderPassDepthStencilAttachment<'a>> {
        Some(egui_wgpu::wgpu::RenderPassDepthStencilAttachment {
            view: &depth.view,
            depth_ops: Some(egui_wgpu::wgpu::Operations {
                load: egui_wgpu::wgpu::LoadOp::Clear(1.0),
                store: egui_wgpu::wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        })
    }
}

pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Viewport {
    pub fn new(resolution: Vector2, min: Vector2, max: Vector2) -> Viewport {
        Viewport {
            x: min.x * resolution.x,
            y: min.y * resolution.y,
            width: ((max.x - min.x) * resolution.x).round(),
            height: ((max.y - min.y) * resolution.y).round(),
        }
    }
}
pub fn skybox_mesh() -> Mesh {
    let s = 100.0;

    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u32>::new();

    let mut add_face = |corners: [[f32; 3]; 4], normal: [f32; 3], uv_min: [f32; 2], uv_max: [f32; 2]| {
        let start = vertices.len() as u32;

        vertices.push(Vertex::new(corners[0], normal, [1.0, 1.0, 1.0, 1.0], [uv_min[0], uv_max[1]], [0.0, 0.0]));
        vertices.push(Vertex::new(corners[1], normal, [1.0, 1.0, 1.0, 1.0], [uv_max[0], uv_max[1]], [0.0, 0.0]));
        vertices.push(Vertex::new(corners[2], normal, [1.0, 1.0, 1.0, 1.0], [uv_max[0], uv_min[1]], [0.0, 0.0]));
        vertices.push(Vertex::new(corners[3], normal, [1.0, 1.0, 1.0, 1.0], [uv_min[0], uv_min[1]], [0.0, 0.0]));

        // inward-facing winding
        indices.extend_from_slice(&[start, start + 2, start + 1, start, start + 3, start + 2]);
    };

    // Atlas cells
    let (front_min, front_max) = atlas_uv(1.0, 1.0);
    let (left_min, left_max) = atlas_uv(0.0, 1.0);
    let (right_min, right_max) = atlas_uv(2.0, 1.0);
    let (back_min, back_max) = atlas_uv(3.0, 1.0);
    let (up_min, up_max) = atlas_uv(1.0, 0.0);
    let (down_min, down_max) = atlas_uv(1.0, 2.0);

    // FRONT (+Z)
    add_face([[-s, -s, s], [s, -s, s], [s, s, s], [-s, s, s]], [0.0, 0.0, -1.0], front_min, front_max);

    // BACK (-Z)
    add_face([[s, -s, -s], [-s, -s, -s], [-s, s, -s], [s, s, -s]], [0.0, 0.0, 1.0], back_min, back_max);

    // LEFT (-X)
    add_face([[-s, -s, -s], [-s, -s, s], [-s, s, s], [-s, s, -s]], [1.0, 0.0, 0.0], left_min, left_max);

    // RIGHT (+X)
    add_face([[s, -s, s], [s, -s, -s], [s, s, -s], [s, s, s]], [-1.0, 0.0, 0.0], right_min, right_max);

    // TOP (+Y)
    add_face([[-s, s, s], [s, s, s], [s, s, -s], [-s, s, -s]], [0.0, -1.0, 0.0], up_min, up_max);

    // BOTTOM (-Y)
    add_face([[-s, -s, -s], [s, -s, -s], [s, -s, s], [-s, -s, s]], [0.0, 1.0, 0.0], down_min, down_max);

    Mesh::new(String::from("Skybox"), vertices, indices, Matrix4x4::default())
}
fn atlas_uv(col: f32, row: f32) -> ([f32; 2], [f32; 2]) {
    let w = 1.0 / 4.0;
    let h = 1.0 / 3.0;

    ([col * w, row * h], [(col + 1.0) * w, (row + 1.0) * h])
}
