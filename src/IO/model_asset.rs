use super::super::Collections::material::Material;
use super::super::Collections::Mesh::Mesh;
use super::Asset::Asset;

//data

pub struct Model_asset {
    pub mesh: Vec<Mesh>,
    pub materials: Vec<Material>,
}

// construction
impl Model_asset {
    pub fn new(mesh: Vec<Mesh>, materials: Vec<Material>) -> Model_asset {
        Model_asset { mesh, materials }
    }
}
// public
impl Model_asset {}
// private
impl Model_asset {}
// asset

impl Asset for Model_asset {}
