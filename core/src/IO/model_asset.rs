use std::sync::Arc;

use super::super::collections::material::Material;
use super::super::collections::mesh::Mesh;
use super::asset::Asset;

//data

#[derive(Clone)]
pub struct ModelAsset {
    pub mesh: Vec<Arc<Mesh>>,
    pub materials: Vec<Arc<Material>>,
}

// construction
impl ModelAsset {
    pub fn new(mesh: Vec<Arc<Mesh>>, materials: Vec<Arc<Material>>) -> ModelAsset {
        ModelAsset { mesh, materials }
    }
}
// public
impl ModelAsset {}
// private
impl ModelAsset {}
// asset

impl Asset for ModelAsset {}
