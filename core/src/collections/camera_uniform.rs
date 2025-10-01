use core::f32;

use crate::collections::{projection::Projection, quaternion::Quaternion, vector3::Vector3};
use cgmath::{prelude::*, Matrix4, Point3};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_position: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
}
impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    // UPDATED!
    // pub fn update_view_proj(&mut self, camera: &CameraState, projection: &Projection) {
    //     let p = Point3::new(camera.position.x, camera.position.y, camera.position.z);
    //     self.view_position = p.to_homogeneous().into();
    //     self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into()
    // }
    pub fn update_view_proj2(&mut self, camera: &CameraSnapshot, projection: &Projection) {
        let p = Point3::new(camera.position.x, camera.position.y, camera.position.z);
        self.view_position = p.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into()
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CameraSnapshot {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}
impl Default for CameraSnapshot {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            fovy: 60.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}
impl CameraSnapshot {
    pub fn new(position: Vector3) -> CameraSnapshot {
        CameraSnapshot {
            position: position.into(),
            rotation: Quaternion::identity(),
            fovy: 60.0,
            znear: 0.1,
            zfar: 512.0,
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let p3 = Point3::new(self.position.x, self.position.y, self.position.z);
        let f0 = self.rotation * Vector3::forward();
        let f = cgmath::Vector3::new(f0.x, f0.y, f0.z);
        let u0 = self.rotation * Vector3::up();
        let u = cgmath::Vector3::new(u0.x, u0.y, u0.z);

        Matrix4::look_to_rh(p3, f, u)
    }
    pub fn get_projection(&self, width: i32, height: i32) -> Projection {
        Projection::new(width as u32, height as u32, cgmath::Deg(self.fovy), self.znear, self.zfar)
    }
    pub fn get_uniform(&self, width: i32, height: i32) -> CameraUniform {
        let mut c = CameraUniform::new();
        c.update_view_proj2(self, &self.get_projection(width, height));
        c
    }
    // pub fn world_to_screen(&self, world_pos: Vector3) -> Option<(f32, f32)> {
    //     // let proj_matrix = Matrix4x4::perspective_lh(self.fovy, self.width as f32 / self.height as f32, self.znear, self.zfar);
    //     let proj_matrix = Matrix4x4::from_cgmath(
    //         Projection::new(self.width as u32, self.height as u32, cgmath::Deg(self.fovy), self.znear, self.zfar).calc_matrix(),
    //     );

    //     let view_matrix = Matrix4x4::look_at(self.position, self.position + Vector3::forward(), Vector3::up());
    //     // Convert position to homogeneous coordinates
    //     let mut clip_space = proj_matrix.multiply_vec4(view_matrix.multiply_vec4(world_pos.to_vector4(1.0)));

    //     // Avoid division by zero
    //     if clip_space.w.abs() < 1e-5 {
    //         return None;
    //     }

    //     // Perspective divide
    //     clip_space.x /= clip_space.w;
    //     clip_space.y /= clip_space.w;
    //     clip_space.z /= clip_space.w;

    //     // Clip check (optional)
    //     // if clip_space.z < 0.0 || clip_space.z > 1.0 {
    //     //     return None; // Behind the camera or too far
    //     // }

    //     // Convert to screen space
    //     // let ndc_x = ((clip_space.x + 1.0) * 0.5);
    //     let ndc_x = (clip_space.x + 10.0) / 20.0;
    //     let ndc_y = (clip_space.y + 10.0) / 20.0; // (1.0 - clip_space.y) * 0.5; // Y flipped for screen space

    //     let screen_x = ndc_x * self.width as f32;
    //     let screen_y = ndc_y * self.height as f32;

    //     Some((screen_x, screen_y))
    // }
}
