use super::super::Collections::material::Material;
use super::super::Collections::Mesh::Mesh;
use super::asset::Asset;

//data

#[derive(Clone)]
pub struct ModelAsset {
    pub mesh: Vec<Mesh>,
    pub materials: Vec<Material>,
}

// construction
impl ModelAsset {
    pub fn new(mesh: Vec<Mesh>, materials: Vec<Material>) -> ModelAsset {
        ModelAsset { mesh, materials }
    }
}
// public
impl ModelAsset {}
// private
impl ModelAsset {}
// asset

impl Asset for ModelAsset {}
