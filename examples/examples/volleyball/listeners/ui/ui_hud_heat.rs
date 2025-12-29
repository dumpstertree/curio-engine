use core::{
    collections::{event_queue::EventQueue, game_state::GameState, vector2::Vector2},
    gameplay::{
        ecs::component::component_transform2d::Transform2D,
        world_context::{GameObject, WorldContext2D},
    },
};
use std::collections::HashMap;

use built_in::component::component_renderer_text::ComponentRendererText;
use hecs::World;
use system_component_default_gameplay::{UI, UIPanel};

use crate::state::{
    host::state_heat::StateHeat,
    state_teams::{StateTeamAssignments, Teams},
    state_turn::StateTurn,
};

pub struct UIHUD {
    go_text: HashMap<i32, GameObject>,
}
impl UIHUD {
    pub fn new() -> Box<UIHUD> {
        Box::new(UIHUD { go_text: HashMap::new() })
    }
}
impl UIPanel for UIHUD {
    fn input_button(&mut self, _button: core::input::key_code::ButtonCode, _state: core::collections::key_state::KeyState) {}
    fn input_axis(&mut self, _axis: core::input::axis_code::AxisCode, _state: core::collections::input_cursor::InputAxisState) {}
}
impl UI for UIHUD {
    fn init(&mut self) {}

    fn present(&mut self, game_state: &mut GameState, _event_queue: &mut EventQueue, context: &mut WorldContext2D) {
        // get cur turn
        let cur_heat = game_state.get::<StateTeamAssignments>();

        for user_guid_heat in cur_heat.team_assignments {
            for i in 0..user_guid_heat.1.len() {
                let mut x_pos = if user_guid_heat.0 == Teams::Red { 0.15 } else { 0.85 };
                if user_guid_heat.1.len() > 1 {
                    if i == 0 {
                        x_pos -= 0.05;
                    } else {
                        x_pos += 0.05;
                    }
                }
                let mut r = ComponentRendererText::default();
                r.set_font_size(0.03);
                let guid = user_guid_heat.1[i];
                self.go_text.insert(
                    guid,
                    context
                        .instantiate("", Transform2D::default().set_position_01(Vector2::new(x_pos, 0.8)))
                        .add_component_value(r),
                );
            }
        }
    }

    fn dismiss(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut WorldContext2D) {}

    fn tick(&mut self, game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut WorldContext2D) {
        // get cur turn
        let cur_heat = game_state.get::<StateHeat>().all_players;

        // edit the text
        for user_guid_heat in &cur_heat {
            if let Some(go) = self.go_text.get(&user_guid_heat.0) {
                go.edit_component::<ComponentRendererText>(|x| {
                    x.set_contents(&format!("HEAT: {}", user_guid_heat.1));
                });
            }
        }
    }
}
