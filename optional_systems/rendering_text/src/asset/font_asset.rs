use curio_core::{
    AssetCommon, Matrix4x4, TextureAsset,
    io::asset_loader::{ASSET_UID_SHADER_UNLIT, ASSET_UID_TEXTURE_FONT_ATLAS, Assets},
};
use ext_rendering::{
    Material, Mesh,
    data::{material::ShaderDesc, mesh::Vertex},
};
use rendering_static::ModelAsset;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct FontDesc {
    pub texture_path: String,
    pub shader_path: String,
    // spacing
    pub char_spacing: f32,
    pub line_spacing: f32,
    pub padding_left: i32,
    pub padding_right: i32,
    pub padding_top: i32,
    pub padding_bottom: i32,
    // spacing
    pub columns: i32,
    pub rows: i32,
    // chars
    pub char_order: String,
}
// asset

pub struct FontAsset {
    // desc: Arc<FontDesc>,
    material: Arc<Material>,
    mesh: HashMap<char, Arc<ModelAsset>>,
    glyph_width: f32,
    glyph_height: f32,
}
impl AssetCommon<FontAsset> for FontAsset {
    fn from_bits(bits: &Vec<u8>) -> FontAsset {
        // let file = File::read(uid);
        let file = String::from_utf8(bits.to_vec()).unwrap();
        // let json: serde_json::Value = serde_json::from_str(&file).expect("file should be proper JSON");
        let my_struct = serde_json::from_str::<FontDesc>(&file).unwrap();
        FontAsset::new(Arc::new(my_struct))
    }
}
impl FontAsset {
    pub fn new(desc: Arc<FontDesc>) -> FontAsset {
        // FontAsset {
        //     desc
        // }

        // let texture = AssetLoader::load_texture_from_path(&File::join_path(&File::get_built_in_asset_path(), &desc.texture_path));
        // let shader = AssetLoader::load_shader_desc(&File::join_path(&File::get_built_in_asset_path(), &desc.shader_path));

        // let texture = AssetLoader::load_texture_from_path(&ASSET_UID_TEXTURE_FONT_ATLAS);
        let texture = Assets::load_asset::<TextureAsset>(&ASSET_UID_TEXTURE_FONT_ATLAS);
        println!("{}, {}", texture.texture.width(), texture.texture.height());

        let shader = Assets::load_asset::<ShaderDesc>(&ASSET_UID_SHADER_UNLIT);

        let w = texture.texture.width() as f32;
        let h = texture.texture.height() as f32;

        let padding_left_01 = desc.padding_left as f32 / w;
        let padding_right_01 = desc.padding_right as f32 / w;
        let padding_top_01 = desc.padding_top as f32 / h;
        let padding_bottom_01 = desc.padding_bottom as f32 / h;

        // Glyph UV layout
        let glyph_width = (1.0 - padding_left_01 - padding_right_01) / desc.columns as f32;
        let glyph_height = (1.0 - padding_top_01 - padding_bottom_01) / desc.rows as f32;

        let mut mesh = HashMap::new();

        let mut material = Material::new("font asset", shader, false);
        material.set_texture_with_label(Some(texture), "diffuse");
        material.finalize();
        let material = Arc::new(material);

        for (index, ch) in desc.char_order.chars().enumerate() {
            let col = index as i32 % desc.columns;
            let row = index as i32 / desc.columns;

            let u_min = padding_left_01 + col as f32 * glyph_width;
            let v_min = padding_top_01 + row as f32 * glyph_height;
            let u_max = u_min + glyph_width;
            let v_max = v_min + glyph_height;
            // let m =
            let vertices = vec![
                Vertex {
                    position: [0.0, 0.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv0: [u_min, v_max],
                    uv1: [0.0, 0.0],
                },
                Vertex {
                    position: [1.0, 0.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv0: [u_max, v_max],
                    uv1: [0.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv0: [u_max, v_min],
                    uv1: [0.0, 0.0],
                },
                Vertex {
                    position: [0.0, 1.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv0: [u_min, v_min],
                    uv1: [0.0, 0.0],
                },
            ];
            let m = Arc::new(Mesh::new(format!("glyph_{}", ch), vertices, vec![0, 1, 2, 0, 2, 3], Matrix4x4::default()));

            mesh.insert(ch, Arc::new(ModelAsset::new(vec![m], vec![material.clone()])));
        }

        FontAsset {
            // desc: desc,
            material: material,
            mesh,
            glyph_width,
            glyph_height,
        }
    }

    pub fn glyph_width(&self) -> f32 {
        self.glyph_width
    }
    pub fn glyph_height(&self) -> f32 {
        self.glyph_height
    }
    pub fn material(&self) -> Arc<Material> {
        self.material.clone()
    }
    pub fn mesh_all() {}
    pub fn mesh_for_char(&self, char: char) -> Arc<ModelAsset> {
        let Some(cached) = self.mesh.get(&char) else {
            return Arc::new(ModelAsset::new(vec![], vec![]));
        };

        return cached.clone();
    }
}
