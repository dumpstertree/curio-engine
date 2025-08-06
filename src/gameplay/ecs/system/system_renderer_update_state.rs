use crate::{
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
    Collections::{event_queue::EventQueue2, game_state::GameState},
};
use ecs_system::ECSSystem;
use hecs::World;

use crate::{
    gameplay::ecs::component::{component_renderer::Renderer, component_transform::Transform},
    system::system_game_states::state_draw::DrawCallsState,
    Collections::DrawCall::DrawCall,
};
#[ECSSystem]
pub struct TestECSSystem {}
impl TestECSSystem {
    pub fn new() -> Box<TestECSSystem> {
        Box::new(TestECSSystem {})
    }
}
impl ECSSystemEventless for TestECSSystem {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }

    fn did_tick(&mut self, state: &mut GameState, world: &mut World, events: &mut EventQueue2) {
        //edit draw call states
        state.edit::<DrawCallsState>(|x| {
            // iterate over each renderer
            for (_, (renderer, transform)) in world.query::<(&Renderer, &Transform)>().iter() {
                // guard - no mesh
                let Some(asset) = &renderer.asset else {
                    continue;
                };

                // add draw call
                for m in &asset.mesh {
                    x.draw_calls.push(DrawCall::draw_mesh_single(
                        asset.mesh[0].clone(),
                        asset.materials[0].clone(),
                        transform.get_matrix(),
                    ));
                }
            }
        });
    }
}
