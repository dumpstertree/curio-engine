use std::cell::RefMut;

use hecs::World;

use crate::{
    collections::{matrix4x4::Matrix4x4, quaternion::Quaternion, vector3::Vector3},
    gameplay::{
        ecs::component::component_transform2d::Transform2D,
        world_context::{GameObject, WorldContext},
    },
};

#[derive(Clone)]
pub struct Transform {
    pub parent: Option<GameObject>,
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
    pub ws_matrix: Matrix4x4,
}
impl Default for Transform {
    fn default() -> Transform {
        Transform {
            parent: None,
            position: Vector3::zero(),
            rotation: Quaternion::identity(),
            scale: Vector3::one(),
            ws_matrix: Matrix4x4::default(),
        }
    }
}
unsafe impl Send for Transform {}
unsafe impl Sync for Transform {}

impl Transform {
    pub fn get_matrix(&self) -> Matrix4x4 {
        Matrix4x4::new(self.position, self.rotation, self.scale)
    }
    pub fn move_towards_position(&mut self, position: Vector3, delta: f32) -> f32 {
        let dist = f32::min((position - self.position).magnitude(), delta);
        if dist == 0.0 {
            return 0.0;
        }
        let dir = (position - self.position).normalize_and_copy();

        self.position = self.position + dir * dist;
        dist
    }
    pub fn set_position(mut self, position: Vector3) -> Transform {
        self.position = position;
        self
    }
    pub fn set_rotation(mut self, rotation: Quaternion) -> Transform {
        self.rotation = rotation;
        self
    }
    pub fn set_scale(mut self, scale: Vector3) -> Transform {
        self.scale = scale;
        self
    }
    pub fn set_parent(mut self, parent: Option<GameObject>) -> Transform {
        self.parent = parent;
        self
    }

    fn update_matrix_in_heirarchy(&mut self, world: &RefMut<'_, World>) -> Matrix4x4 {
        let my_matrix = self.get_matrix();

        if let Some(parent_entity) = &self.parent {
            if let Ok(mut parent_renderer) = world.get::<&mut Transform>(parent_entity.entity) {
                let parent_matrix = parent_renderer.update_matrix_in_heirarchy(world);
                self.ws_matrix = Matrix4x4::multiply(&parent_matrix, &my_matrix);
                return self.ws_matrix.clone();
            }
        }
        self.ws_matrix = my_matrix;
        return self.ws_matrix.clone();
    }
    pub fn get_world_matrix(&self, world: &WorldContext) -> Matrix4x4 {
        self.ws_matrix.clone()
        // let mut matrix = self.get_matrix();
        // let mut current = self.parent.clone();

        // while let Some(parent_entity) = current {
        //     if let Some(parent_transform) = parent_entity.get_component::<Transform>() {
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

pub fn update_transform(world: &mut WorldContext) {
    let borrow = world.world.borrow_mut();
    for w in borrow.iter() {
        if let Some(mut x) = w.get::<&mut Transform>() {
            _ = x.update_matrix_in_heirarchy(&borrow);
        }
        if let Some(mut x) = w.get::<&mut Transform2D>() {
            _ = x.update_matrix_in_heirarchy(&borrow);
        }
    }
}
