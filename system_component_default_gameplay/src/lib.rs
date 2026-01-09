pub mod gameobject;
pub mod gameplay_instance;
pub mod world_context_2d;
pub mod world_context_3d;

pub mod static_fns {
    pub mod register_built_in_facets;
    pub mod register_built_in_habits;
}
pub mod traits_internal {
    pub mod ui_common;
    pub mod world_context_common;
}
pub mod built_in {
    pub mod impulse {
        pub mod ui_events;
    }
    pub mod habit {
        pub mod system_camera_update_state;
        pub mod system_debug_gui_screen;
        pub mod system_debug_gui_time;
        pub mod system_debug_toggle;
        pub mod system_renderer_update_light_state;
        pub mod system_renderer_update_state;
    }
    pub mod facet {
        pub mod facet_collider {
            pub mod component_collider_box;
            pub mod component_collider_sphere;
        }
        pub mod facet_transform {
            pub mod component_transform;
            pub mod component_transform2d;
        }
        pub mod facet_renderer {
            pub mod component_renderer_animated;
            pub mod component_renderer_static;
            pub mod component_renderer_text;
        }
        pub mod component_camera;
        pub mod component_input_index;
        pub mod component_light;
    }
}

pub mod traits {
    pub mod field_override;
    pub mod habit;
    pub mod impulse;
    pub mod scope;
    pub mod ui_dialog;
    pub mod ui_events;
    pub mod ui_hud;
    pub mod ui_panel;
}
pub mod static_data {
    pub mod global_components;
    pub mod global_ecs;
    pub mod global_event_recievers;
}

use core::{
    collections::{
        event_queue::{EventQueue, EventScope, IGameEvent},
        game_state::GameState,
        key_state::KeyState,
        vector3::Vector3,
    },
    dumpster_engine::GameMode,
    input::{axis_code::AxisCode, key_code::ButtonCode},
    system::{system_component::SystemComponent, system_components::system_component_gameplay::SystemComponentGameplay},
};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, hash::Hash, vec};

use crate::{
    gameplay_instance::GameplayInstance,
    static_fns::{register_built_in_facets::register_built_in_component, register_built_in_habits::register_built_in_ecs},
    traits::{ui_events::IUIEvent, ui_panel::UIPanel},
};

pub struct SystemComponentDefaultGameplay<T, U>
where
    T: IGameEvent + Clone + 'static,
    U: IUIEvent + 'static,
{
    game_instance: Vec<GameplayInstance<T, U>>,
}

impl<T, U> SystemComponentDefaultGameplay<T, U>
where
    T: IGameEvent + Clone + 'static,
    U: IUIEvent + 'static,
{
    pub fn new() -> Box<SystemComponentDefaultGameplay<T, U>> {
        // register any built in components to static data

        register_built_in_ecs();
        register_built_in_component();

        // return instance
        Box::new(SystemComponentDefaultGameplay::<T, U> { game_instance: vec![] })
    }
}
impl<T, U> SystemComponentGameplay for SystemComponentDefaultGameplay<T, U>
where
    T: IGameEvent + Display + 'static + Clone,
    U: IUIEvent + 'static,
{
    // fn set_systems(&mut self, _ecs_systems_eventless: Vec<fn() -> Box<dyn ECSSystemEventless>>) {}
}
impl<T, U> SystemComponent for SystemComponentDefaultGameplay<T, U>
where
    T: IGameEvent + Display + 'static + Clone,
    U: IUIEvent + 'static,
{
    fn input_button(&mut self, _game_state: &mut Vec<GameState>, _key_code: ButtonCode, _val: KeyState) {}
    fn input_axis(&mut self, _game_statee: &mut Vec<GameState>, _axis_code: AxisCode, _val: Vector3) {}
    fn order(&self) -> i32 {
        5000
    }
    fn init(&mut self, _: &mut Vec<GameState>) {}
    fn set_game_mode(&mut self, _game_state: &mut Vec<GameState>, game_mode: &GameMode) {
        for _ in &game_mode.game_instances {
            self.game_instance.push(GameplayInstance::new());
        }
    }
    fn debug(&mut self, _game_state: &mut Vec<GameState>, _system_queue: &mut Vec<EventQueue>) {}
    fn tick(&mut self, game_state: &mut Vec<GameState>, event_queue: &mut Vec<EventQueue>) {
        // iterate over each gamestate
        for i in 0..game_state.len() {
            // get this index values
            let game_state = &mut game_state[i];
            let event_queue = &mut event_queue[i];

            // tick the instance
            self.game_instance[i].tick(game_state, event_queue);
        }
    }
}
