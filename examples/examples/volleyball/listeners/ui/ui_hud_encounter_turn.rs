use curio_core::{
    Vector2,
    collections::{event_queue::EventQueue, game_state::GameState, input_cursor::InputAxisState, key_state::KeyState},
    input::{axis_code::AxisCode, key_code::ButtonCode},
};

use gameplay::{
    built_in::facet::{renderer::renderer_text::RendererText, transform::transform2d::Transform2D},
    context_2d::Context2D,
    form::Form,
    traits::ui_panel::UIPanel,
    traits_internal::ui_common::UICommon,
};

use crate::state::{state_teams::Teams, state_turn::StateTurn};

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
            .spawn("text.turn", Transform2D::default().set_position_01(Vector2::new(0.5, 0.8)))
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
        let cur_turn = game_state.get::<StateTurn>().active_instance_id;

        // edit the text
        go_text.edit_facet::<RendererText>(|x| {
            match cur_turn {
                Teams::Red => x.set_contents("RED"),
                Teams::Blue => x.set_contents("BLUE"),
            };
        });
    }
}
