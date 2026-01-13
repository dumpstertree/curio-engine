use core::collections::{event_queue::EventQueue, game_state::GameState, vector2::Vector2};
use system_component_default_gameplay::{
    built_in::facet::{renderer::renderer_text::RendererText, transform::transform2d::Transform2D},
    context_2d::Context2D,
    form::Form,
    traits::ui_panel::UIPanel,
    traits_internal::ui_common::UICommon,
};

use crate::state::state_ball_mode::{BallModes, StateBallMode};

pub struct UIHUD {
    go_text: Option<Form>,
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

    fn present(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, context: &mut Context2D) {
        // create obj
        let go_text = context
            .spawn("text.ball_mode", Transform2D::default().set_position_01(Vector2::new(0.5, 0.7)))
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
        let cur_mode = game_state.get::<StateBallMode>().mode;

        // edit the text
        go_text.edit_facet::<RendererText>(|x| {
            match cur_mode {
                BallModes::Serve => x.set_contents("SERVE"),
                BallModes::Bump => x.set_contents("BUMP"),
                BallModes::Set => x.set_contents("SET"),
                BallModes::Spike => x.set_contents("SPIKE"),
                BallModes::Scored => x.set_contents("SERVE"),
            };
        });
    }
}
