use cgmath::Quaternion;
use hecs::World;

use crate::{
    game_state::GameState,
    gameplay::game_events::GameEvents,
    system::system_components::{
        gameplay_components::gameplay_component_default::{ECSSystem, EngineCommands, EventQueue},
        graphics_components::graphics_component_wgpu::DrawCallsState,
    },
    Collections::{matrix4x4::Matrix4x4, vector3::Vector3, DrawCall::DrawCall},
    IO::{model_asset::Model_asset, AssetLoader::AssetLoader},
};

pub struct TestECSSystem {
    rotation: f32,
    model_asset_0: Option<Model_asset>,
    model_asset_1: Option<Model_asset>,
    model_asset_2: Option<Model_asset>,
}
impl TestECSSystem {
    pub fn new() -> TestECSSystem {
        TestECSSystem {
            rotation: 0.0,
            model_asset_0: None,
            model_asset_1: None,
            model_asset_2: None,
        }
    }
    fn ToQuaternion(roll: f32, pitch: f32, yaw: f32) -> Quaternion<f32> {
        // // Abbreviations for the various angular functions

        let cr: f32 = f32::cos(roll * 0.5);
        let sr: f32 = f32::sin(roll * 0.5);
        let cp: f32 = f32::cos(pitch * 0.5);
        let sp: f32 = f32::sin(pitch * 0.5);
        let cy: f32 = f32::cos(yaw * 0.5);
        let sy: f32 = f32::sin(yaw * 0.5);

        Quaternion::<f32>::new(
            cr * cp * cy + sr * sp * sy,
            sr * cp * cy - cr * sp * sy,
            cr * sp * cy + sr * cp * sy,
            cr * cp * sy - sr * sp * cy,
        )

        // return q;
    }
}
impl ECSSystem<GameEvents> for TestECSSystem {
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<GameEvents>) -> bool {
        true
    }
    fn enable(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<GameEvents>) {
        println!("enable render");
    }
    fn init(&mut self, game_state: &mut GameState, scene: &mut World, event_queue: &mut EventQueue<GameEvents>) {
        // get assets again
        self.model_asset_0 = AssetLoader::load_gltf("Cube3.glb");
        self.model_asset_1 = AssetLoader::load_gltf("Cube4.glb");
        self.model_asset_2 = AssetLoader::load_gltf("Cone.glb");

        event_queue.enqueue_event(GameEvents::B(123));
    }
    fn tick(&mut self, game_state: &mut GameState, scene: &mut World, event_queue: &mut EventQueue<GameEvents>) {
        self.rotation = self.rotation + 0.1;

        //get draw call state
        let mut x: Vec<DrawCall> = Vec::new();

        let a0 = self.model_asset_0.as_ref().unwrap();
        let a1 = self.model_asset_1.as_ref().unwrap();
        let a2 = self.model_asset_2.as_ref().unwrap();
        x.push(DrawCall::draw_mesh_single(
            a0.mesh[0].clone(),
            a0.materials[0].clone(),
            Matrix4x4::new(
                Vector3::new(0.0, -1.0, -5.0),
                TestECSSystem::ToQuaternion(0.0, self.rotation, 0.0),
                Vector3::one(),
            ),
        ));
        x.push(DrawCall::draw_mesh_single(
            a1.mesh[0].clone(),
            a1.materials[0].clone(),
            Matrix4x4::new(
                Vector3::new(-5.0, f32::sin(self.rotation), -5.0),
                TestECSSystem::ToQuaternion(self.rotation / 2.0, self.rotation, 0.0),
                Vector3::new(f32::sin(self.rotation), 1.0, 1.0),
            ),
        ));
        x.push(DrawCall::draw_mesh_single(
            a2.mesh[0].clone(),
            a2.materials[0].clone(),
            Matrix4x4::new(
                Vector3::new(5.0, 1.0, -5.0),
                TestECSSystem::ToQuaternion(0.0, self.rotation, 0.0),
                Vector3::one(),
            ),
        ));

        game_state.set_draw_calls(DrawCallsState::new2(x));
    }
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<GameEvents>, event: &GameEvents) {
        match event {
            GameEvents::A(x) => {
                println!("test {}", x);
                event_queue.enqueue_command(EngineCommands::Exit);
                event_queue.enqueue_command(EngineCommands::Resize);
                event_queue.enqueue_command(EngineCommands::Exit);
            }
            GameEvents::B(x) => {
                println!("test {}", x);
                event_queue.enqueue_event(GameEvents::A(String::from("send a")));
            }
        }
    }
}
