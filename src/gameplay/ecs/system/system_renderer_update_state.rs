use crate::Collections::game_state::GameState;
use hecs::World;

use crate::{
    gameplay::ecs::component::{component_renderer::Renderer, component_transform::Transform},
    system::{
        system_components::gameplay_components::gameplay_component_default::ECSSystemEventless, system_game_states::state_draw::DrawCallsState,
    },
    Collections::DrawCall::DrawCall,
};

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

    fn did_tick(&mut self, game_state: &mut GameState, scene: &mut World) {
        //edit draw call states
        game_state.edit::<DrawCallsState>(|x| {
            // iterate over each renderer
            for (_, (renderer, transform)) in scene.query::<(&Renderer, &Transform)>().iter() {
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
