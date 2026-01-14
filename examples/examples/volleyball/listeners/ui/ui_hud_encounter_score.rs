use curio_core::{
    collections::{event_queue::EventQueue, game_state::GameState, input_cursor::InputAxisState, key_state::KeyState, vector2::Vector2},
    input::{axis_code::AxisCode, key_code::ButtonCode},
};
use system_component_default_gameplay::{
    built_in::facet::{renderer::renderer_text::RendererText, transform::transform2d::Transform2D},
    context_2d::Context2D,
    form::Form,
    traits::ui_panel::UIPanel,
    traits_internal::ui_common::UICommon,
};

use crate::state::{state_score::StateScore, state_teams::Teams};

pub struct UIHUD {
    go_text: Option<Form>,
}
impl UIHUD {
    pub fn new() -> Box<UIHUD> {
        Box::new(UIHUD { go_text: None })
    }
}
impl UIPanel for UIHUD {
    fn input_button(&mut self, _button: ButtonCode, _state: KeyState) {}
    fn input_axis(&mut self, _axis: AxisCode, _state: InputAxisState) {}
}
impl UICommon for UIHUD {
    fn init(&mut self) {}

    fn present(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, context: &mut Context2D) {
        // create obj
        let go_text = context
            .spawn("text.score", Transform2D::default().set_position_01(Vector2::new(0.5, 0.9)))
            .add_facet_default::<RendererText>();

        // save
        self.go_text = Some(go_text);
    }

    fn dismiss(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut Context2D) {
        self.go_text.clone().unwrap().destroy();
    }

    fn tick(&mut self, game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut Context2D) {
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
        go_text.edit_facet::<RendererText>(|x| {
            x.set_contents(&format!("{} : {}", cur_scores[&Teams::Red], cur_scores[&Teams::Blue]));
        });
    }
}
