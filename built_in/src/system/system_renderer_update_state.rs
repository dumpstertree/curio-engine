use crate::component::{component_renderer::Renderer, component_transform::Transform};
use built_in_state::state_draw::DrawCallsState;
use core::{
    collections::{draw_call::DrawCall, event_queue::EventQueue, game_state::GameState},
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
        state.edit::<DrawCallsState>(|x| {
            // iterate over each renderer
            for (_, (renderer, transform)) in world.query::<(&Renderer, &Transform)>().iter() {
                // guard - no mesh
                let Some(asset) = &renderer.asset else {
                    continue;
                };

                // add draw call
                for _ in &asset.mesh {
                    x.draw_calls
                        .push(DrawCall::draw_mesh_single(asset.mesh[0].clone(), asset.materials[0].clone(), transform.get_matrix()));
                }
            }
        });
    }
}
