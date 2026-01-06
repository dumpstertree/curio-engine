use core::collections::{event_queue::EventQueue, game_state::GameState, vector2::Vector2};
use system_component_default_gameplay::{
    UI, UIPanel,
    component::{component_renderer_text::ComponentRendererText, component_transform2d::Transform2D},
    world_context::{GameObject, WorldContext2D},
};

use crate::state::{state_score::StateScore, state_teams::Teams};

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
impl UI for UIHUD {
    fn init(&mut self) {}

    fn present(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, context: &mut WorldContext2D) {
        // create obj
        let go_text = context
            .instantiate("text.score", Transform2D::default().set_position_01(Vector2::new(0.5, 0.9)))
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
        let cur_scores = game_state.get::<StateScore>().all_scores;

        // missing scores for some reason
        if !cur_scores.contains_key(&Teams::Red) || !cur_scores.contains_key(&Teams::Blue) {
            return;
        }

        // edit the text
        go_text.edit_component::<ComponentRendererText>(|x| {
            x.set_contents(&format!("{} : {}", cur_scores[&Teams::Red], cur_scores[&Teams::Blue]));
        });
    }
}
