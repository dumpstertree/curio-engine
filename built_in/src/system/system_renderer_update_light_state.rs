use crate::component::{component_light::ComponentLight, component_renderer_animated::RendererAnimated, component_renderer_static::Renderer, component_transform::Transform};
use built_in_state::{state_draw::DrawCallsState, state_lights::StateLights, state_time::TimeState};
use core::{
    collections::{
        color::Color,
        draw_call::DrawCall,
        event_queue::EventQueue,
        game_state::{self, GameState},
        light_uniform::DrawCallLight,
        vector3::Vector3,
    },
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct SystemRendererUpdateState {}
impl SystemRendererUpdateState {
    pub fn new() -> Box<SystemRendererUpdateState> {
        Box::new(SystemRendererUpdateState {})
    }
}
impl ECSSystemEventless for SystemRendererUpdateState {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }

    fn did_tick(&mut self, state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        //edit draw call states
        let t = state.get_value2::<TimeState>().scaled_time;
        state.edit::<StateLights>(|x| {
            for (_, (light, transform)) in world.query::<(&ComponentLight, &Transform)>().iter() {
                let dir = Vector3::new(20.0, -45.0, -15.0).normalize_and_copy();
                let pos = dir * -1.0 * 1.0;
                x.all_lights.push(DrawCallLight {
                    light_type: core::collections::light_uniform::LightType::Directional,
                    position: [pos.x, pos.y, pos.z],
                    direction: [dir.x, dir.y, dir.z],
                    color: [1.0, 0.0, 0.0],
                    intensity: 1.0,
                    radius: 10.0,
                });
            }
        });
    }
}
