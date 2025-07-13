use cgmath::{Matrix4, Rad, SquareMatrix, Transform};

use crate::{
    system::system_game_state::IState,
    Collections::{
        matrix4x4::{self, Matrix4x4},
        quaternion::{self, Quaternion},
        vector3::Vector3,
    },
};

use cgmath::*;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use winit::dpi::PhysicalPosition;
use winit::event::*;
use winit::keyboard::KeyCode;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

impl CameraState {
    pub fn new(position: Vector3, yaw: Rad<f32>, pitch: Rad<f32>) -> CameraState {
        CameraState {
            position: position.into(),
            rotation: Quaternion::identity(),
            yaw: yaw,
            pitch: pitch,
            aspect: 1.0,
            fovy: 60.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position.to_point3(),
            // cgmath::Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            cgmath::Vector3::new(0.0, 0.0, 1.0),
            cgmath::Vector3::unit_y(),
        )
    }
}

#[derive(Clone)]

pub struct CameraState {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}
impl IState<CameraState> for CameraState {
    fn default() -> CameraState {
        CameraState {
            // matrix: Matrix4x4::default(),
            pitch: Rad::<f32> { 0: 0.0 },
            yaw: Rad::<f32> { 0: 0.0 },
            position: Vector3::zero(),
            rotation: Quaternion::identity(),
            aspect: 1.0, //config.width as f32 / config.height as f32,
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
            aspect: 1.0, //config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
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

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        CameraState::OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

use cgmath::*;
