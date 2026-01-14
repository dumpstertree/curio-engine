use std::sync::Arc;

use crate::random::Random;

use super::super::collections::material::Material;
use super::super::collections::mesh::Mesh;
use super::asset::Asset;

//data

#[derive(Clone)]
pub struct ModelAsset {
    pub instance_id: i32,
    pub mesh: Vec<Arc<Mesh>>,
    pub materials: Vec<Arc<Material>>,
}

// construction
impl ModelAsset {
    pub fn new(mesh: Vec<Arc<Mesh>>, materials: Vec<Arc<Material>>) -> ModelAsset {
        ModelAsset {
            instance_id: Random::range_int(-9999999, 99999999),
            mesh,
            materials,
        }
    }
}
// public
impl ModelAsset {}
// private
impl ModelAsset {}
// asset

impl Asset for ModelAsset {}
