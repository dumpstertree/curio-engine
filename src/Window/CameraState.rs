// use cgmath::Matrix4;

// use crate::Collections::vector3::Vector3;

// #[derive(Clone)]
// pub struct CameraState {
//     pub position: Vector3,
//     pub aspect: f32,
//     pub fovy: f32,
//     pub znear: f32,
//     pub zfar: f32,
// }

// impl CameraState {
//     pub fn default() -> CameraState {
//         CameraState {
//             position: Vector3::zero(),
//             aspect: 1.0, //config.width as f32 / config.height as f32,
//             fovy: 45.0,
//             znear: 0.1,
//             zfar: 100.0,
//         }
//     }
//     pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
//         let view = Matrix4::from_translation(Vector3::new(self.position.x, self.position.y, self.position.z).to_cg_math());
//         let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
//         return CameraState::OPENGL_TO_WGPU_MATRIX * proj * view;
//     }

//     #[rustfmt::skip]
//     pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
//         cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
//         cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
//         cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
//         cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
//     );
// }
