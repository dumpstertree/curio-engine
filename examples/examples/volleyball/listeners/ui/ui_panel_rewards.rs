use core::collections::{event_queue::EventQueue, game_state::GameState, vector2::Vector2};

use built_in_state::state_input::InputState;
use system_component_default_gameplay::{
    built_in::facet::{renderer::renderer_text::RendererText, transform::transform2d::Transform2D},
    context_2d::Context2D,
    form::Form,
    traits::ui_panel::UIPanel,
    traits_internal::ui_common::UICommon,
};

use crate::game_events::GameEvents;

pub struct UIPanelInstance {
    go_desc: Option<Form>,
    go_opts: Vec<Form>,
}
impl UIPanelInstance {
    pub fn new() -> Box<UIPanelInstance> {
        Box::new(UIPanelInstance { go_desc: None, go_opts: Vec::new() })
    }
}
impl UIPanel for UIPanelInstance {
    fn input_button(&mut self, button: core::input::key_code::ButtonCode, state: core::collections::key_state::KeyState) {}

    fn input_axis(&mut self, axis: core::input::axis_code::AxisCode, state: core::collections::input_cursor::InputAxisState) {}
}
impl UICommon for UIPanelInstance {
    fn init(&mut self) {}

    fn present(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut Context2D) {
        let mut rend = RendererText::default();
        rend.set_contents("Combat Rewards");
        // create obj
        let go_desc = context
            .spawn("text.description", Transform2D::default().set_position_01(Vector2::new(0.5, 0.5)))
            .add_facet(rend);

        for i in 0..3 {
            //
            let mut rend = RendererText::default();
            rend.set_contents("reward!");

            let go_opt_0 = context
                .spawn("text.option_0", Transform2D::default().set_position_01(Vector2::new(0.5, 0.4 - i as f32 * 0.1)))
                .add_facet(rend);

            self.go_opts.push(go_opt_0);
        }

        // save
        self.go_desc = Some(go_desc);
    }

    fn dismiss(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut Context2D) {
        println!("try dismiss!");
        self.go_desc.clone().unwrap().destroy();
        for x in &self.go_opts {
            x.destroy();
        }
        self.go_opts.clear();
    }

    fn tick(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut Context2D) {
        let state_input = game_state.get::<InputState>();
        if state_input.mapped[0]
            .get_button_or_default("turn_end")
            .went_up
        {
            event_queue.enqueue_event(GameEvents::RequestLeaveExplorationRoom);
        }
    }
}
