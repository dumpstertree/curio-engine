use crate::Collections::{matrix4x4::Matrix4x4, quaternion::Quaternion, vector3::Vector3};

#[derive(Clone, Copy)]
pub struct Transform {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
}

impl Transform {
    pub fn default() -> Transform {
        Transform {
            position: Vector3::zero(),
            rotation: Quaternion::identity(),
            scale: Vector3::one(),
        }
    }
    pub fn get_matrix(self) -> Matrix4x4 {
        Matrix4x4::new(self.position, self.rotation, self.scale)
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
}
