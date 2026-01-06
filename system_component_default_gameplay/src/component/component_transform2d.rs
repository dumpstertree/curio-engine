use core::collections::{matrix4x4::Matrix4x4, quaternion::Quaternion, vector2::Vector2, vector3::Vector3};
use std::cell::RefMut;

use hecs::World;
use serde::{Deserialize, de::DeserializeOwned};

use crate::{
    field_override::FieldDeserialize,
    world_context::{GameObject, WorldContext},
};

#[derive(Clone)]
pub struct Transform2D {
    pub parent: Option<GameObject>,
    pub position: Vector2,
    pub rotation: Quaternion,
    pub scale: Vector3,
    ws_matrix: Matrix4x4,
    render_order: i32,
}
impl FieldDeserialize for Transform2D {
    fn override_field(&mut self, field: &str, val: &str) {
        match field {
            "position" => self.position = val.parse().unwrap_or_default(),
            "rotation" => self.rotation = Quaternion::from_euler(val.parse().unwrap_or_default()),
            "scale" => self.scale = val.parse().unwrap_or_default(),
            _ => {}
        }
    }
}
impl Default for Transform2D {
    fn default() -> Transform2D {
        Transform2D {
            parent: None,
            position: Vector2::zero(),
            rotation: Quaternion::identity(),
            scale: Vector3::one(),
            ws_matrix: Matrix4x4::default(),
            render_order: 0,
        }
    }
}
unsafe impl Send for Transform2D {}
unsafe impl Sync for Transform2D {}

impl Transform2D {
    pub fn get_matrix(&self) -> Matrix4x4 {
        let frustrum_w = 2.1; // these values are made up because the camera is perspective
        let frustrum_h = 1.1;
        Matrix4x4::new(
            Vector3::new(
                //
                remap(self.position.x, 0.0, 1.0, -frustrum_w / 2.0, frustrum_w / 2.0),
                remap(self.position.y, 0.0, 1.0, -frustrum_h / 2.0, frustrum_h / 2.0),
                0.01 * self.render_order as f32,
            ),
            self.rotation,
            self.scale,
        )
    }

    pub fn set_render_order(mut self, order: i32) -> Transform2D {
        self.render_order = order;
        self
    }
    pub fn set_position_01(mut self, position: Vector2) -> Transform2D {
        self.position = position;
        self
    }
    pub fn set_rotation(mut self, rotation: Quaternion) -> Transform2D {
        self.rotation = rotation;
        self
    }
    pub fn set_scale(mut self, scale: Vector3) -> Transform2D {
        self.scale = scale;
        self
    }
    pub fn set_parent(mut self, parent: Option<GameObject>) -> Transform2D {
        self.parent = parent;
        self
    }
    pub fn update_matrix_in_heirarchy(&mut self, world: &RefMut<'_, World>) -> Matrix4x4 {
        let my_matrix = self.get_matrix();

        if let Some(parent_entity) = &self.parent {
            if let Ok(mut parent_renderer) = world.get::<&mut Transform2D>(parent_entity.entity) {
                let parent_matrix = parent_renderer.update_matrix_in_heirarchy(world);
                self.ws_matrix = Matrix4x4::multiply(&parent_matrix, &my_matrix);
                return self.ws_matrix.clone();
            }
        }
        self.ws_matrix = my_matrix;
        return self.ws_matrix.clone();
    }
    pub fn get_world_matrix(&self, world: &WorldContext) -> Matrix4x4 {
        self.ws_matrix
        // let mut matrix = self.get_matrix();
        // let mut current = self.parent.clone();

        // while let Some(parent_entity) = current {
        //     if let Some(parent_transform) = parent_entity.get_component::<Transform2D>() {
        //         // matrix = Matrix4x4::multiply(&matrix, &parent_transform.get_matrix());

        //         matrix = Matrix4x4::multiply(&parent_transform.get_matrix(), &matrix);
        //         current = parent_transform.parent.clone();
        //     } else {
        //         break;
        //     }
        // }

        // matrix
    }
}
pub fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    (value - from_min) / (from_max - from_min) * (to_max - to_min) + to_min
}
