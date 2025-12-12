use crate::{
    collections::{matrix4x4::Matrix4x4, quaternion::Quaternion, vector2::Vector2, vector3::Vector3},
    gameplay::world_context::{GameObject, WorldContext},
};

#[derive(Clone)]
pub struct Transform2D {
    pub parent: Option<GameObject>,
    pub position: Vector2,
    pub rotation: Quaternion,
    pub scale: Vector3,
}
impl Default for Transform2D {
    fn default() -> Transform2D {
        Transform2D {
            parent: None,
            position: Vector2::zero(),
            rotation: Quaternion::identity(),
            scale: Vector3::one(),
        }
    }
}
unsafe impl Send for Transform2D {}
unsafe impl Sync for Transform2D {}

impl Transform2D {
    pub fn get_matrix(&self) -> Matrix4x4 {
        Matrix4x4::new(self.position.to_vector3(0.0), self.rotation, self.scale)
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
    pub fn get_world_matrix(&self, world: &WorldContext) -> Matrix4x4 {
        let mut matrix = self.get_matrix();
        let mut current = self.parent.clone();

        while let Some(parent_entity) = current {
            if let Some(parent_transform) = parent_entity.get_component::<Transform2D>() {
                // matrix = Matrix4x4::multiply(&matrix, &parent_transform.get_matrix());

                matrix = Matrix4x4::multiply(&parent_transform.get_matrix(), &matrix);
                current = parent_transform.parent.clone();
            } else {
                break;
            }
        }

        matrix
    }
}
