use curio_core::{ASSET_UID_SHADER_UNLIT, Assets, Color, FieldState, Matrix4x4, Quaternion, TextureAsset, Vector3};
use ext_rendering::{
    Material, Mesh,
    data::{material::ShaderDesc, mesh::Vertex},
};
use facet::facet;
use gameplay::traits::field_override::FieldOverride;
use rendering_static::ModelAsset;
use std::sync::Arc;

#[facet]
pub struct RendererImage {
    pub asset: Option<Arc<ModelAsset>>,
    pub bounds_matrix: Matrix4x4,
    pub enabled: bool,
    pub tint: Color,
}
impl RendererImage {
    pub fn default() -> RendererImage {
        RendererImage {
            asset: None,
            enabled: true,
            tint: Color::white(),
            owner: None,
            bounds_matrix: Matrix4x4::default(),
        }
    }
    pub fn builder() -> RendererImageBuilder {
        RendererImageBuilder { texture: None, enabled: true, tint: Color::white() }
    }
}
impl FieldOverride for RendererImage {
    fn apply(&mut self, field: &str, value: &str) {
        let value = value.trim();
        match field {
            "asset" => {
                // ignore unknown names rather than panicking on the lookup
                if let Some(uid) = Assets::try_lookup_key_for_name(value) {
                    self.set_asset(Some(Assets::load_asset::<TextureAsset>(&uid)));
                }
            }
            "enabled" => self.enabled = value.parse().unwrap_or_default(),
            "tint" => self.tint = value.parse().unwrap_or(self.tint),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        // NOTE: asset is intentionally not serialized. The texture it was loaded from is not
        // recoverable from the built quad material, and a placeholder would fail the name lookup
        // when re-applied (see FormBuilder3D::facet).
        vec![
            FieldState::new("enabled", self.enabled), //
            FieldState::new("tint", self.tint),
        ]
    }
}

impl RendererImage {
    /// Rebuilds the internal quad material to display `asset`, adjusting bounds to the texture aspect ratio.
    pub fn set_asset(&mut self, asset: Option<Arc<TextureAsset>>) {
        if let Some(asset) = &asset {
            self.bounds_matrix = Matrix4x4::new(Vector3::zero(), Quaternion::identity(), Vector3::new(1.0, 0.5 * (asset.texture.height() as f32 / asset.texture.width() as f32), 1.0));
        }
        let shader = Assets::load_asset::<ShaderDesc>(&ASSET_UID_SHADER_UNLIT);
        let mut material = Material::new("image", shader, false);
        material.set_texture_with_label(asset, "diffuse");
        material.finalize();
        let material = Arc::new(material);
        self.asset = Some(Arc::new(ModelAsset::new(vec![Primitives::quad()], vec![material])));
    }
}

pub struct RendererImageBuilder {
    texture: Option<Arc<TextureAsset>>,
    enabled: bool,
    tint: Color,
}
impl RendererImageBuilder {
    pub fn texture(mut self, texture: Option<Arc<TextureAsset>>) -> Self {
        self.texture = texture;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }
    pub fn build(self) -> RendererImage {
        let mut image = RendererImage {
            asset: None,
            enabled: self.enabled,
            tint: self.tint,
            owner: None,
            bounds_matrix: Matrix4x4::default(),
        };
        if self.texture.is_some() {
            image.set_asset(self.texture);
        }
        image
    }
}

pub struct Primitives {}
impl Primitives {
    pub fn quad() -> Arc<Mesh> {
        let vertices = vec![
            Vertex {
                position: [-0.5, -0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                uv0: [0.0, 1.0],
                uv1: [0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                uv0: [1.0, 1.0],
                uv1: [0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                uv0: [1.0, 0.0],
                uv1: [0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
                uv0: [0.0, 0.0],
                uv1: [0.0, 0.0],
            },
        ];
        let m = Mesh::new(String::from("quad"), vertices, vec![0, 1, 2, 0, 2, 3], Matrix4x4::default());
        Arc::new(m)
    }
}
