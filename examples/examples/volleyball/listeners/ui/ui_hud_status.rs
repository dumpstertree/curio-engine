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

use crate::state::host::{state_currency::StateCurrency, state_deck_exploration::StateDeckExploration, state_health_exploration::StateHealthExploration};

pub struct UIHUD {
    go_health: Option<Form>,
    go_gold: Option<Form>,
    go_cards_cnt: Option<Form>,
}
impl UIHUD {
    pub fn new() -> Box<UIHUD> {
        Box::new(UIHUD { go_health: None, go_gold: None, go_cards_cnt: None })
    }
}
impl UIPanel for UIHUD {
    fn input_button(&mut self, _button: ButtonCode, _state: KeyState) {}
    fn input_axis(&mut self, _axis: AxisCode, _state: InputAxisState) {}
}
impl UICommon for UIHUD {
    fn init(&mut self) {}

    fn present(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, context: &mut Context2D) {
        return;
        let go_helath = context
            .spawn("text.health", Transform2D::default().set_position_01(Vector2::new(0.2, 0.95)))
            .add_facet_default::<RendererText>();
        let go_card_cnt = context
            .spawn("text.cards", Transform2D::default().set_position_01(Vector2::new(0.5, 0.95)))
            .add_facet_default::<RendererText>();
        let go_gold = context
            .spawn("text.gold", Transform2D::default().set_position_01(Vector2::new(0.9, 0.95)))
            .add_facet_default::<RendererText>();

        self.go_health = Some(go_helath);
        self.go_cards_cnt = Some(go_card_cnt);
        self.go_gold = Some(go_gold);
    }

    fn dismiss(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut Context2D) {}

    fn tick(&mut self, game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut Context2D) {
        let state_health = game_state.get::<StateHealthExploration>();
        if let Some(x) = &self.go_health {
            x.edit_facet::<RendererText>(|y| {
                y.set_contents(&format!("{} of {} Health ", state_health.all.get(&game_state.instance_id).unwrap().0, state_health.all.get(&game_state.instance_id).unwrap().1));
            });
        }

        let state_deck_exploration = game_state.get::<StateDeckExploration>();
        if let Some(x) = &self.go_cards_cnt {
            x.edit_facet::<RendererText>(|y| {
                y.set_contents(&format!(
                    "{} Cards",
                    state_deck_exploration
                        .deck
                        .get(&game_state.instance_id)
                        .unwrap()
                        .all_cards
                        .len()
                ));
            });
        }

        let state_currency = game_state.get::<StateCurrency>();
        if let Some(x) = &self.go_gold {
            x.edit_facet::<RendererText>(|y| {
                y.set_contents(&format!("{} Gold", state_currency.currency));
            });
        }
    }
}
