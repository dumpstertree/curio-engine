use crate::{
    system::system_game_states::state_time::TimeState,
    Collections::{game_state::GameState, Mesh::Mesh},
};
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
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World) -> bool {
        true
    }

    fn did_tick(&mut self, game_state: &mut GameState, scene: &mut World) {
        let mut dc = game_state.get_value2::<DrawCallsState>();
        let time = game_state.get_value2::<TimeState>();
        for x in scene.query::<(&Renderer, &Transform)>().iter() {
            let r: &Renderer = x.1 .0;
            let t: &Transform = x.1 .1;

            let Some(asset) = &r.asset else {
                continue;
            };

            for m in &asset.mesh {
                dc.draw_calls.push(DrawCall::draw_mesh_single(
                    asset.mesh[0].clone(),
                    asset.materials[0].clone(),
                    t.get_matrix(),
                ));
            }
        }
        game_state.set_value2::<DrawCallsState>(dc);
    }
}
