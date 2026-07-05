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
    enabled: bool,
    tint: Color,
    cached_enabled_in_hierachy: bool,
    cached_tint_in_hierachy: Color,
}
impl RendererImage {
    pub fn default() -> RendererImage {
        RendererImage {
            asset: None,
            enabled: true,
            tint: Color::white(),
            cached_enabled_in_hierachy: true,
            cached_tint_in_hierachy: Color::white(),
            owner: None,
            bounds_matrix: Matrix4x4::default(),
        }
    }
}
impl FieldOverride for RendererImage {
    fn apply(&mut self, field: &str, value: &str) {
        match field {
            "asset" => self.set_asset(Some(Assets::load_asset::<TextureAsset>(&Assets::try_lookup_key_for_name(value).unwrap()))),
            "enabled" => self.enabled = value.parse().unwrap_or_default(),
            "tint" => self.tint = value.parse().unwrap_or_default(),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("asset", "TODO"),
            FieldState::new("enabled", self.enabled), //
            FieldState::new("tint", self.tint),
        ]
    }
}

impl RendererImage {
    pub fn set_asset(&mut self, asset: Option<Arc<TextureAsset>>) {
        if let Some(asset) = asset.clone() {
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
impl RendererImage {
    /// gets a model asset taking into account tint
    pub fn get_model_asset(asset: Option<Arc<ModelAsset>>, tint: Color) -> Option<Arc<ModelAsset>> {
        // no asset
        let Some(asset) = asset else {
            return None;
        };

        // no tint
        if tint == Color::white() {
            return Some(asset);
        }

        // edit material to include tint
        let mut mats = Vec::new();
        for mat in &asset.materials {
            let mut m = mat.instantiate("new");
            m.set_color_with_label(tint, "tint");
            m.finalize();
            mats.push(Arc::new(m));
        }

        // return edited
        Some(Arc::new(ModelAsset::new(asset.mesh.clone(), mats)))
    }
}

pub struct Primitives {}
impl Primitives {
    pub fn quad() -> Arc<Mesh> {
        // let m =
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
