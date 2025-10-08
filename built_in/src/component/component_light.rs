use core::{
    collections::{
        color::Color,
        light_uniform::{DrawCallLight, LightType},
        vector3::Vector3,
    },
    io::model_asset::ModelAsset,
};
use std::sync::Arc;

pub struct ComponentLight {
    pub asset: LightType,
    pub direction: Vector3,
    pub color: Color,
    pub radius: f32,
    pub intensity: f32,
}

impl ComponentLight {
    pub fn default() -> ComponentLight {
        ComponentLight {
            asset: LightType::Point,
            direction: Vector3::zero(),
            color: Color::white(),
            radius: 10.0,
            intensity: 1.0,
        }
    }
}
