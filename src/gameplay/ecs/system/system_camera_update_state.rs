use hecs::World;

use crate::{
    game_state::GameState,
    gameplay::{
        ecs::component::{component_camera::Camera, component_transform::Transform},
        game_events::GameEvents,
    },
    system::system_components::gameplay_components::gameplay_component_default::{ECSSystem, EventQueue},
};

pub struct PostCameraECSSystem {}
impl PostCameraECSSystem {}
impl ECSSystem<GameEvents> for PostCameraECSSystem {
    fn init(&mut self, game_state: &mut GameState, scene: &mut World, event_queue: &mut EventQueue<GameEvents>) {
        scene.spawn((Transform::default(), Camera::default()));
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<GameEvents>) {
        for (_, (t, _)) in world.query_mut::<(&mut Transform, &Camera)>() {
            // set camera gamestate
            let mut state_camera = game_state.get_camera().clone();
            state_camera.position = t.position.clone();
            game_state.set_camera(state_camera);
        }
    }
}
