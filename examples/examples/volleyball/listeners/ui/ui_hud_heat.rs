use curio_core::{
    Vector2,
    collections::{event_queue::EventQueue, game_state::GameState, input_cursor::InputAxisState, key_state::KeyState},
    input::{axis_code::AxisCode, key_code::ButtonCode},
};
use std::collections::HashMap;

use gameplay::{
    built_in::facet::{renderer::renderer_text::RendererText, transform::transform2d::Transform2D},
    context_2d::Context2D,
    form::Form,
    traits::ui_panel::UIPanel,
    traits_internal::ui_common::UICommon,
};

use crate::state::{
    host::state_heat::StateHeat,
    state_teams::{StateTeamAssignments, Teams},
};

pub struct UIHUD {
    go_text: HashMap<i32, Form>,
}
impl UIHUD {
    pub fn new() -> Box<UIHUD> {
        Box::new(UIHUD { go_text: HashMap::new() })
    }
}
impl UIPanel for UIHUD {
    fn input_button(&mut self, _button: ButtonCode, _state: KeyState) {}
    fn input_axis(&mut self, _axis: AxisCode, _state: InputAxisState) {}
}
impl UICommon for UIHUD {
    fn init(&mut self) {}

    fn present(&mut self, game_state: &mut GameState, _event_queue: &mut EventQueue, context: &mut Context2D) {
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
                let mut r = RendererText::default();
                r.set_font_size(0.03);
                let guid = user_guid_heat.1[i];
                self.go_text.insert(
                    guid,
                    context
                        .spawn("", Transform2D::default().set_position_01(Vector2::new(x_pos, 0.8)))
                        .add_facet(r),
                );
            }
        }
    }

    fn dismiss(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut Context2D) {
        for x in &self.go_text {
            x.1.destroy();
        }

        self.go_text.clear();
    }

    fn tick(&mut self, game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut Context2D) {
        // get cur turn
        let cur_heat = game_state.get::<StateHeat>().all_players;

        // edit the text
        for user_guid_heat in &cur_heat {
            if let Some(go) = self.go_text.get(&user_guid_heat.0) {
                go.edit_facet::<RendererText>(|x| {
                    x.set_contents(&format!("HEAT: {}", user_guid_heat.1));
                });
            }
        }
    }
}
