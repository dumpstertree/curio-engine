use crate::{
    system::system_game_state::IState,
    Collections::{camera_uniform::CameraUniform, matrix4x4::Matrix4x4, projection::Projection, quaternion::Quaternion, vector3::Vector3},
};
use cgmath::*;
use cgmath::{Matrix4, Rad};

#[derive(Clone)]
pub struct CameraState {
    pub width: i32,
    pub height: i32,
    pub position: Vector3,
    pub rotation: Quaternion,
    // pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}
impl CameraState {
    pub fn new(width: i32, height: i32, position: Vector3, yaw: Rad<f32>, pitch: Rad<f32>) -> CameraState {
        CameraState {
            position: position.into(),
            rotation: Quaternion::identity(),
            yaw: yaw,
            pitch: pitch,
            // aspect: 1.0,
            fovy: 60.0,
            znear: 0.1,
            zfar: 100.0,
            width: width,
            height: height,
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(
            self.position.to_point3(),
            (self.rotation * Vector3::forward()).to_cg_math(),
            (self.rotation * Vector3::up()).to_cg_math(),
        )
    }
    pub fn get_projection(&self) -> Projection {
        Projection::new(self.width as u32, self.height as u32, cgmath::Deg(self.fovy), self.znear, self.zfar)
    }
    pub fn get_uniform(&self) -> CameraUniform {
        let mut c = CameraUniform::new();
        c.update_view_proj(self, &self.get_projection());
        c
    }
}

impl IState<CameraState> for CameraState {
    fn default() -> CameraState {
        CameraState {
            // matrix: Matrix4x4::default(),
            pitch: Rad::<f32> { 0: 0.0 },
            yaw: Rad::<f32> { 0: 0.0 },
            position: Vector3::zero(),
            rotation: Quaternion::identity(),
            width: 128,
            height: 128,
            // aspect: 1.0, //config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    fn id() -> i32 {
        9879897
    }
}
impl CameraState {
    pub fn default() -> CameraState {
        CameraState {
            pitch: Rad::<f32> { 0: 0.0 },
            yaw: Rad::<f32> { 0: 0.0 },
            // matrix: Matrix4x4::default(),
            position: Vector3::zero(),
            rotation: Quaternion::identity(),
            // aspect: 1.0, //config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            width: 128,
            height: 128,
        }
    }

    pub fn world_to_screen(&self, world_pos: Vector3) -> Option<(f32, f32)> {
        // let proj_matrix = Matrix4x4::perspective_lh(self.fovy, self.width as f32 / self.height as f32, self.znear, self.zfar);
        let proj_matrix = Matrix4x4::from_cgmath(
            Projection::new(self.width as u32, self.height as u32, cgmath::Deg(self.fovy), self.znear, self.zfar).calc_matrix(),
        );

        let view_matrix = Matrix4x4::look_at(self.position, self.position + Vector3::forward(), Vector3::up());
        // Convert position to homogeneous coordinates
        let mut clip_space =
            proj_matrix.multiply_vec4(view_matrix.multiply_vec4(crate::Collections::vector4::Vector4::new_from_vec3(world_pos, 1.0)));

        // Avoid division by zero
        if clip_space.w.abs() < 1e-5 {
            return None;
        }

        // Perspective divide
        clip_space.x /= clip_space.w;
        clip_space.y /= clip_space.w;
        clip_space.z /= clip_space.w;

        // Clip check (optional)
        // if clip_space.z < 0.0 || clip_space.z > 1.0 {
        //     return None; // Behind the camera or too far
        // }

        // Convert to screen space
        // let ndc_x = ((clip_space.x + 1.0) * 0.5);
        let ndc_x = (clip_space.x + 10.0) / 20.0;
        let ndc_y = (clip_space.y + 10.0) / 20.0; // (1.0 - clip_space.y) * 0.5; // Y flipped for screen space

        let screen_x = ndc_x * self.width as f32;
        let screen_y = ndc_y * self.height as f32;

        Some((screen_x, screen_y))
    }
    // pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
    //     let mut pos = self.position;
    //     pos.x = pos.x * 1.0;
    //     pos.z = pos.z * 1.0;

    //     let mut rot = self.rotation;
    //     // rot = rot * Quaternion::from_angle_axis(Vector3::up(), 180.0);

    //     let view = Matrix4x4::new(pos, rot, Vector3::one()).to_cg_math();
    //     // let view = matrix4x4::Matrix4x4::new(Vector3::new(0.0, 0.0, 20.0), Quaternion::identity(), Vector3::one()).to_cg_math();
    //     // let view = view.inverse_transform().unwrap();
    //     let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
    //     return CameraState::OPENGL_TO_WGPU_MATRIX * proj * view;
    // }

    #[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
        cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
        cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
        cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
        cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
    );
}
