use crate::component::{
    component_renderer_animated::RendererAnimated,
    component_renderer_static::Renderer,
    component_renderer_text::{ComponentRendererText, RendererCommon},
    component_transform::Transform,
};
use built_in_state::{state_draw::DrawCallsState, state_time::TimeState};
use core::{
    collections::{draw_call::DrawCall, event_queue::EventQueue, game_state::GameState, matrix4x4::Matrix4x4},
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
        let time = state.get_value2::<TimeState>().scaled_time;
        //edit draw call states
        state.edit::<DrawCallsState>(|x| {
            // iterate over each renderer
            for (_, (renderer, transform)) in world.query::<(&Renderer, &Transform)>().iter() {
                if !renderer.enabled_in_hierarchy(&world) {
                    continue;
                }
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
            for (_, (renderer, _)) in world.query::<(&mut RendererAnimated, &Transform)>().iter() {
                if !renderer.enabled_in_hierarchy(&world) {
                    continue;
                }
                // update all mesh
                renderer.update_mesh(time);
            }
            for (_, (renderer, transform)) in world.query::<(&mut RendererAnimated, &Transform)>().iter() {
                if !renderer.enabled_in_hierarchy(&world) {
                    continue;
                }
                // guard - no mesh
                if renderer.asset.is_some() {
                    let Some(asset) = &renderer.asset else {
                        continue;
                    };

                    // add draw call
                    for m in &renderer.mesh {
                        x.draw_calls
                            .push(DrawCall::draw_mesh_single(m.clone(), asset.material.clone(), transform.get_matrix()));
                    }
                }
            }
            for (_, (renderer, transform)) in world
                .query::<(&mut ComponentRendererText, &Transform)>()
                .iter()
            {
                if !renderer.enabled_in_hierarchy(&world) {
                    continue;
                }
                renderer.rebuild();
                for asset_for_matricies in &renderer.asset {
                    for arc_mesh in &asset_for_matricies.0.mesh {
                        let transform_matrix = transform.get_world_matrix(world);
                        let mut inst_matricies = Vec::new();
                        for mesh_matrix in &asset_for_matricies.1 {
                            inst_matricies.push(Matrix4x4::multiply(&transform_matrix, mesh_matrix));
                        }
                        // for inst_matrix in inst_matricies {
                        //     x.draw_calls
                        //         .push(DrawCall::draw_mesh_single(arc_mesh.clone(), asset_for_matricies.0.materials[0].clone(), inst_matrix));
                        // }
                        x.draw_calls
                            .push(DrawCall::draw_mesh_instanced(arc_mesh.clone(), asset_for_matricies.0.materials[0].clone(), inst_matricies));
                    }
                }
            }
        });
    }
}
