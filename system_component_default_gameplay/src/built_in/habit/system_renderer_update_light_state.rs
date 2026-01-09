use built_in_state::state_time::TimeState;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};

use crate::{
    traits::{habit::Habit, scope::Scope},
    world_context_3d::WorldContext,
};

#[derive(Default)]
pub struct Instance {}
impl Instance {
    pub fn new() -> Box<Instance> {
        Box::new(Instance {})
    }
}
impl Scope for Instance {
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Habit for Instance {
    fn enable(&mut self, game_state: &mut GameState, _: &mut WorldContext, _: &mut EventQueue) {
        // game_state.edit::<StateSun>(|x| {
        //     x.cast_shadows = true;
        //     x.color = Color::green();
        //     x.direction = (Vector3::down() + Vector3::forward()).normalize_and_copy()
        // });
    }
    fn did_tick(&mut self, state: &mut GameState, world: &mut WorldContext, _: &mut EventQueue) {
        //edit draw call states
        let t = state.get::<TimeState>().scaled_time;
        // state.edit::<StateLights>(|x| {
        //     for (_, (light, transform)) in world.query::<(&ComponentLight, &Transform)>().iter() {
        //         x.all_lights.push(DrawCallLight {
        //             light_type: core::collections::light_uniform::LightType::Directional,
        //             position: [f32::sin(t as f32) * 10.0, transform.position.y, 0.0],
        //             direction: [20.0, f32::sin(t as f32) * -45.0, -15.0],
        //             color: [1.0, 0.0, 0.0],
        //             intensity: 1.0,
        //             radius: 10.0,
        //         });
        //     }
        // });

        // state.edit::<StateSun>(|x| {
        //     x.cast_shadows = true;
        //     x.color = Color::green();
        //     x.direction = (Vector3::down() + Vector3::forward()).normalize_and_copy()
        // });
        // state.edit::<StateLights>(|x| {
        //     for (_, (light, transform)) in world.query::<(&ComponentLight, &Transform)>().iter() {
        //         let dir = Vector3::new(20.0, -45.0, -15.0).normalize_and_copy();
        //         x.all_lights.push(DrawCallLight {
        //             light_type: core::collections::light_uniform::LightType::Directional,
        //             position: [0.0, 50.0, 100.0],
        //             direction: [dir.x, dir.y, dir.z],
        //             color: [1.0, 0.0, 0.0],
        //             intensity: 1.0,
        //             radius: 10.0,
        //         });
        //     }
        // });
    }
}

// use crate::component::{component_light::ComponentLight, component_renderer_animated::RendererAnimated, component_renderer_static::Renderer, component_transform::Transform};
// use built_in_state::{state_draw::DrawCallsState, state_lights::StateLights, state_sun::StateSun, state_time::TimeState};
// use core::{
//     collections::{
//         color::Color,
//         draw_call::DrawCall,
//         event_queue::EventQueue,
//         game_state::{self, GameState},
//         light_uniform::DrawCallLight,
//         vector3::Vector3,
//     },
//     gameplay::ecs::traits::ecs_system::ECSSystemEventless,
// };
// use ecs_system::habit;
// use hecs::World;

// #[global_ecs_system]
// pub struct SystemRendererUpdateState {}
// impl SystemRendererUpdateState {
//     pub fn new() -> Box<SystemRendererUpdateState> {
//         Box::new(SystemRendererUpdateState {})
//     }
// }
// impl ECSSystemEventless for SystemRendererUpdateState {
//     fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
//         true
//     }

//     fn did_tick(&mut self, state: &mut GameState, world: &mut WorldContext, _: &mut EventQueue) {
//         //edit draw call states
//         state.edit::<StateLights>(|x| {
//             // get all lights with transforms
//             for (_, (light, transform)) in world.query::<(&ComponentLight, &Transform)>().iter() {
//                 // add a light
//                 x.all_lights.push(DrawCallLight {
//                     light_type: light.asset,
//                     position: [transform.position.x, transform.position.y, transform.position.z],
//                     direction: [light.direction.x, light.direction.y, light.direction.z],
//                     color: [light.color.as_r_01(), light.color.as_g_01(), light.color.as_b_01()],
//                     intensity: light.intensity,
//                     radius: light.radius,
//                 });
//             }
//         });
//     }
// }
