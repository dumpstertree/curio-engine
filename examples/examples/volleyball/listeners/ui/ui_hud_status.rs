use core::collections::{event_queue::EventQueue, game_state::GameState, vector2::Vector2};

use system_component_default_gameplay::{
    built_in::facet::{facet_renderer::component_renderer_text::ComponentRendererText, facet_transform::component_transform2d::Transform2D},
    gameobject::GameObject,
    traits::ui_panel::UIPanel,
    traits_internal::ui_common::UICommon,
    world_context_2d::WorldContext2D,
};

use crate::state::host::{state_currency::StateCurrency, state_deck_exploration::StateDeckExploration, state_health_exploration::StateHealthExploration};

pub struct UIHUD {
    go_health: Option<GameObject>,
    go_gold: Option<GameObject>,
    go_cards_cnt: Option<GameObject>,
}
impl UIHUD {
    pub fn new() -> Box<UIHUD> {
        Box::new(UIHUD { go_health: None, go_gold: None, go_cards_cnt: None })
    }
}
impl UIPanel for UIHUD {
    fn input_button(&mut self, _button: core::input::key_code::ButtonCode, _state: core::collections::key_state::KeyState) {}
    fn input_axis(&mut self, _axis: core::input::axis_code::AxisCode, _state: core::collections::input_cursor::InputAxisState) {}
}
impl UICommon for UIHUD {
    fn init(&mut self) {}

    fn present(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, context: &mut WorldContext2D) {
        let go_helath = context
            .instantiate("text.health", Transform2D::default().set_position_01(Vector2::new(0.2, 0.95)))
            .add_component_default::<ComponentRendererText>();
        let go_card_cnt = context
            .instantiate("text.cards", Transform2D::default().set_position_01(Vector2::new(0.5, 0.95)))
            .add_component_default::<ComponentRendererText>();
        let go_gold = context
            .instantiate("text.gold", Transform2D::default().set_position_01(Vector2::new(0.9, 0.95)))
            .add_component_default::<ComponentRendererText>();

        self.go_health = Some(go_helath);
        self.go_cards_cnt = Some(go_card_cnt);
        self.go_gold = Some(go_gold);
    }

    fn dismiss(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut WorldContext2D) {}

    fn tick(&mut self, game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut WorldContext2D) {
        let state_health = game_state.get::<StateHealthExploration>();
        if let Some(x) = &self.go_health {
            x.edit_component::<ComponentRendererText>(|y| {
                y.set_contents(&format!("{} of {} Health ", state_health.all.get(&game_state.instance_id).unwrap().0, state_health.all.get(&game_state.instance_id).unwrap().1));
            });
        }

        let state_deck_exploration = game_state.get::<StateDeckExploration>();
        if let Some(x) = &self.go_cards_cnt {
            x.edit_component::<ComponentRendererText>(|y| {
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
            x.edit_component::<ComponentRendererText>(|y| {
                y.set_contents(&format!("{} Gold", state_currency.currency));
            });
        }
    }
}
