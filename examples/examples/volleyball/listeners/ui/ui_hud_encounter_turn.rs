use core::collections::{event_queue::EventQueue, game_state::GameState, vector2::Vector2};

use system_component_default_gameplay::{
    built_in::facet::{facet_renderer::component_renderer_text::ComponentRendererText, facet_transform::component_transform2d::Transform2D},
    gameobject::GameObject,
    traits::ui_panel::UIPanel,
    traits_internal::ui_common::UICommon,
    world_context_2d::WorldContext2D,
};

use crate::state::{state_teams::Teams, state_turn::StateTurn};

pub struct UIHUD {
    go_text: Option<GameObject>,
}
impl UIHUD {
    pub fn new() -> Box<UIHUD> {
        Box::new(UIHUD { go_text: None })
    }
}
impl UIPanel for UIHUD {
    fn input_button(&mut self, _button: core::input::key_code::ButtonCode, _state: core::collections::key_state::KeyState) {}
    fn input_axis(&mut self, _axis: core::input::axis_code::AxisCode, _state: core::collections::input_cursor::InputAxisState) {}
}
impl UICommon for UIHUD {
    fn init(&mut self) {}

    fn present(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, context: &mut WorldContext2D) {
        // create obj
        let go_text = context
            .instantiate("text.turn", Transform2D::default().set_position_01(Vector2::new(0.5, 0.8)))
            .add_component_default::<ComponentRendererText>();

        // save
        self.go_text = Some(go_text);
    }

    fn dismiss(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut WorldContext2D) {
        self.go_text.clone().unwrap().destroy();
    }

    fn tick(&mut self, game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut WorldContext2D) {
        // try to unwrap
        let Some(go_text) = &self.go_text else {
            return;
        };

        // get cur turn
        let cur_turn = game_state.get::<StateTurn>().active_instance_id;

        // edit the text
        go_text.edit_component::<ComponentRendererText>(|x| {
            match cur_turn {
                Teams::Red => x.set_contents("RED"),
                Teams::Blue => x.set_contents("BLUE"),
            };
        });
    }
}
