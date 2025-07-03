mod IO {
    pub(crate) mod Asset;
    pub(crate) mod AssetLoader;
    pub(crate) mod model_asset;
    pub(crate) mod texture_asset;
}
mod Collections {
    pub(crate) mod Color;
    pub(crate) mod DrawCall;
    pub(crate) mod GraphicsBufferCache;
    pub(crate) mod Mesh;
    pub(crate) mod key_state;
    pub(crate) mod material;
    pub(crate) mod matrix4x4;
    pub(crate) mod vector3;
}

mod Window {
    pub(crate) mod CameraState;
    pub(crate) mod SystemWindow;
    pub(crate) mod app;
    pub(crate) mod state;
}
mod gameplay {
    pub mod ecs {
        pub mod component {
            pub(crate) mod component_camera;
            pub(crate) mod component_renderer;
            pub(crate) mod component_transform;
        }
        pub mod system {
            pub(crate) mod system_camera_fps;
            pub(crate) mod system_camera_update_state;
            pub(crate) mod system_renderer_update_state;
        }
    }
    pub(crate) mod game_events;
}
mod system {
    pub(crate) mod system_component;
    pub mod system_components {
        pub(crate) mod gameplay_component;
        pub mod gameplay_components {
            pub(crate) mod gameplay_component_default;
        }
        pub(crate) mod time_component;
        pub mod time_components {
            pub(crate) mod time_component;
        }
        pub(crate) mod graphics_component;
        pub mod graphics_components {
            pub(crate) mod graphics_component_wgpu;
        }
        pub(crate) mod input_component;
        pub mod input_components {
            pub(crate) mod input_component_default;
        }
    }
}
mod game_state;

mod texture;

use std::{
    any::{self, TypeId},
    collections::HashMap,
};

use crate::{
    game_state::AnyMap,
    gameplay::ecs::system::{
        system_camera_fps::FPSCameraECSSystem, system_camera_update_state::PostCameraECSSystem, system_renderer_update_state::TestECSSystem,
    },
    system::{
        system_component::ISystemComponent,
        system_components::{
            gameplay_components::gameplay_component_default::GameplayComponentDefault,
            input_components::input_component_default::InputComponentDefault, time_component::time_component,
            time_components::time_component::TimeComponent,
        },
    },
    Collections::vector3::{self, Vector3},
};
use cgmath::Point3;
use system::system_components::graphics_components::graphics_component_wgpu::WGPUGraphicsComponent;
use wgpu::wgc::device::queue;
use Window::CameraState::CameraState;

use crate::Window::{state, SystemWindow::SystemWindow};

pub fn run() -> anyhow::Result<()> {
    // init logger
    env_logger::init();
    println!("a");
    let mut c: Vec<Box<dyn ISystemComponent>> = Vec::new();
    c.push(Box::new(InputComponentDefault::new()));
    c.push(Box::new(WGPUGraphicsComponent::new()));
    c.push(Box::new(TimeComponent::new()));
    c.push(Box::new(GameplayComponentDefault::new(vec![
        Box::new(PostCameraECSSystem {}),
        Box::new(FPSCameraECSSystem {}),
        Box::new(TestECSSystem::new()),
    ])));
    let mut w = SystemWindow::new(Box::new(WGPUGraphicsComponent::new()), c);
    w.run();

    // passed
    Ok(())
}
trait IEvent {}

// pub struct Events {
//     queue: Vec<Box<IEvent>>,
// }
// impl Events {
//     pub fn enqueue(&mut self, event: Box<IEvent>) {
//         self.queue.push(event);
//     }
//     pub fn dequeue(&mut self) {
//         for x in &self.queue {}
//     }
//     // pub fn get_value<T>(&self,  val :T) -> Result<&T, GetError> {
//     //     self.cache.get_result::<T, i32>(&key)
//     // }
// }
// impl Events {
//     pub fn subscibe<T, U>(thing: fn(T, U)) {}
//     pub fn unsubscibe<T>() {}

//     pub fn raise(self) {
//         for x in self.hash {
//             x.1();
//         }
//     }
// }
// fn handle_window_on_key(key_code: KeyCode, key_state: Window::app::KeyState) {
//     unsafe {
//         match CAMERA_CONTROLLER {
//             None => {
//                 println!("create controlleer");
//                 CAMERA_CONTROLLER = Some(CameraController::new());
//             }
//             _ => {}
//         }

//         match &mut CAMERA_CONTROLLER {
//             Some(x) => {
//                 // println!("prc");
//                 x.process_events(key_code, key_state);
//                 x.update(&mut CAMERA_STATE);
//             }
//             _ => {
//                 println!("no controller");
//             }
//         }
//     }
// }

// fn handle_window_on_resize() {
//     println!("get  resize");
// }
// static mut CAMERA_STATE: CameraState = CameraState {
//     position: Vector3::zero(),
//     aspect: 1.0,
//     fovy: 45.0,
//     znear: 0.1,
//     zfar: 100.0,
// };

// static mut ASSET_LOADER: Option<AssetLoader::AssetLoader> = None;
// static mut MESH0: Option<Model_asset> = None;
// static mut MESH1: Option<Model_asset> = None;
// static mut CAMERA_CONTROLLER: Option<CameraController> = None;
// static mut WINDOW: Option<winit::window::Window> = None;
// static mut STATE: Option<State> = None;

// fn get_camera_state() -> CameraState {
//     unsafe { CAMERA_STATE.clone() }
// }
// fn get_draw_calls<'a>() -> Option<Vec<DrawCall<'a>>> {
//     let mut draw_calls: Vec<DrawCall> = Vec::new();

//     unsafe {
//         match &mut ASSET_LOADER {
//             Some(x) => match MESH0 {
//                 None => {
//                     println!("load 0");
//                     MESH0 = x.load_gltf("char.glb");
//                 }
//                 _ => {}
//             },
//             _ => {}
//         }
//         match &mut ASSET_LOADER {
//             Some(x) => match MESH1 {
//                 None => {
//                     println!("load 1");
//                     MESH1 = x.load_gltf("cone.glb");
//                 }
//                 _ => {}
//             },
//             _ => {}
//         }

//         match &MESH0 {
//             Some(mesh) => {
//                 let matrix = Matrix4x4::new(Vector3::<f32>::new(0.0, 0.0, 0.0), Quaternion::<f32>::new(0.0, 0.0, 0.0, 0.0));
//                 for m in &mesh.mesh {
//                     let dc: DrawCall<'_> = DrawCall::draw_mesh_single(m, &mesh.materials[0], matrix);
//                     draw_calls.push(dc);
//                 }
//             }
//             None => {}
//         }
//         match &MESH1 {
//             Some(mesh) => {
//                 let matrix = Matrix4x4::new(Vector3::<f32>::new(0.0, 5.0, 0.0), Quaternion::<f32>::new(0.0, 0.0, 0.0, 0.0));
//                 for m in &mesh.mesh {
//                     let dc: DrawCall<'_> = DrawCall::draw_mesh_single(m, &mesh.materials[0], matrix);
//                     draw_calls.push(dc);
//                 }
//             }
//             None => {}
//         }
//     }
//     Some(draw_calls)
// }

// struct Engine<'a> {
//     MESH0: Option<Model_asset<'a>>,
//     MESH1: Option<Model_asset<'a>>,
//     ASSET_LOADER: Option<AssetLoader::AssetLoader<'a>>,
//     CAMERA_CONTROLLER: CameraController,
//     // WINDOW_Manager: winit::window::Window,
//     Window: Option<Window::app::Window<'a>>,
//     // evnt_loop: EventLoop<State<'static>>,
// }

// //
// impl DatasourceWindow for Engine<'_> {
//     fn get_draw_calls<'a>(&self) -> Vec<DrawCall> {
//         println!("get draw calls");
//         Vec::new()
//     }
// }

// //engine
// impl<'a> Engine<'_> {
//     // pub fn set_asset_loader(&mut self, asset_loader: AssetLoader::AssetLoader<'a>) {
//     //     self.ASSET_LOADER = Some(asset_loader);
//     // }
//     pub fn build() -> Engine<'a> {
//         //
//         let mut window_attributes = winit::window::Window::default_attributes();

//         // create os window
//         let event_loop: EventLoop<State> = EventLoop::with_user_event().build().unwrap();
//         let os_window: winit::window::Window = event_loop.create_window(window_attributes).unwrap();

//         // generate state
//         let state: State = State::new(os_window).block_on();

//         // create window

//         let mut e = Engine {
//             ASSET_LOADER: None,
//             MESH0: None,
//             MESH1: None,
//             CAMERA_CONTROLLER: CameraController::new(),
//             // WINDOW_Manager: os_window,
//             Window: None,
//             // evnt_loop: event_loop,
//         };

//         // let mut window = Window::app::Window::new(state, self.get_draw_calls, get_camera_state, &e);
//         let mut window = Window::app::Window::new(state, &e);

//         let s = window.get_state();
//         let asset_loader = AssetLoader::AssetLoader::new(ShaderCache::new(s.device.clone()), &s.device, &s.queue);
//         // e.Window = Some(window);
//         // e.ASSET_LOADER = Some(asset_loader);

//         // e.MESH0 = e.ASSET_LOADER.unwrap().load_gltf("char.glb");
//         // e.MESH1 = e.ASSET_LOADER.unwrap().load_gltf("cone.glb");

//         // &self.Window.set_on_key_callback(handle_window_on_key);
//         // &self.Window.set_on_resize_callback(handle_window_on_resize);

//         // window.run(event_loop);

//         return e;
//     }

//     pub fn run(&mut self) {}

//     fn get_camera_state(&self) -> CameraState {
//         unsafe { CAMERA_STATE.clone() }
//     }
//     fn get_draw_calls(&'a mut self) -> Option<Vec<DrawCall<'a>>> {
//         let mut draw_calls: Vec<DrawCall> = Vec::new();

//         match &self.MESH0 {
//             Some(mesh) => {
//                 let matrix = Matrix4x4::new(Vector3::<f32>::new(0.0, 0.0, 0.0), Quaternion::<f32>::new(0.0, 0.0, 0.0, 0.0));
//                 for m in &mesh.mesh {
//                     let dc: DrawCall<'_> = DrawCall::draw_mesh_single(m, &mesh.materials[0], matrix);
//                     draw_calls.push(dc);
//                 }
//             }
//             None => {}
//         }
//         match &self.MESH1 {
//             Some(mesh) => {
//                 let matrix = Matrix4x4::new(Vector3::<f32>::new(0.0, 5.0, 0.0), Quaternion::<f32>::new(0.0, 0.0, 0.0, 0.0));
//                 for m in &mesh.mesh {
//                     let dc: DrawCall<'_> = DrawCall::draw_mesh_single(m, &mesh.materials[0], matrix);
//                     draw_calls.push(dc);
//                 }
//             }
//             None => {}
//         }
//         Some(draw_calls)
//     }
//     // fn handle_window_on_key(key_code: KeyCode, key_state: Window::app::KeyState) {
//     //     unsafe {
//     //         match CAMERA_CONTROLLER {
//     //             None => {
//     //                 println!("create controlleer");
//     //                 CAMERA_CONTROLLER = Some(CameraController::new());
//     //             }
//     //             _ => {}
//     //         }

//     //         match &mut CAMERA_CONTROLLER {
//     //             Some(x) => {
//     //                 // println!("prc");
//     //                 x.process_events(key_code, key_state);
//     //                 x.update(&mut CAMERA_STATE);
//     //             }
//     //             _ => {
//     //                 println!("no controller");
//     //             }
//     //         }
//     //     }
//     // }
// }
