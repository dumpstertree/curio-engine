use core::collections::{event_queue::EventQueue, game_state::GameState, vector2::Vector2, vector3::Vector3};

use built_in_state::{state_input::InputState, state_time::TimeState};
use system_component_default_gameplay::{
    built_in::facet::{facet_renderer::component_renderer_text::ComponentRendererText, facet_transform::component_transform2d::Transform2D},
    gameobject::GameObject,
    traits::ui_panel::UIPanel,
    traits_internal::ui_common::UICommon,
    world_context_2d::WorldContext2D,
};

use crate::{
    cards::card_library::CardLibrary,
    game_events::GameEvents,
    state::host::state_shop::{StateShop, StockItems},
};

pub struct UIPanelInstance {
    selected_index: i32,
    go_desc: Option<GameObject>,
    go_stock: Vec<GameObject>,
    go_leave: Option<GameObject>,
}
impl UIPanelInstance {
    pub fn new() -> Box<UIPanelInstance> {
        Box::new(UIPanelInstance {
            selected_index: 0,
            go_desc: None,
            go_stock: Vec::new(),
            go_leave: None,
        })
    }
}
impl UIPanel for UIPanelInstance {
    fn input_button(&mut self, button: core::input::key_code::ButtonCode, state: core::collections::key_state::KeyState) {}

    fn input_axis(&mut self, axis: core::input::axis_code::AxisCode, state: core::collections::input_cursor::InputAxisState) {}
}
impl UICommon for UIPanelInstance {
    fn init(&mut self) {}

    fn present(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext2D) {
        let state_store = game_state.get::<StateShop>();
        for i in 0..state_store.shop.stock.len() {
            let mut rend = ComponentRendererText::default();
            let stock = state_store.shop.stock.get(i).unwrap();
            match &stock.item {
                StockItems::Card(card_id) => rend.set_contents(&format!("{} : {} x{}", CardLibrary::get_master_card(&card_id).title, stock.cost, stock.count)),
                StockItems::Relic(_) => todo!(),
            };

            let go_opt_0 = context
                .instantiate("text.option_0", Transform2D::default().set_position_01(Vector2::new(0.5, 0.4 - i as f32 * 0.1)))
                .add_component_value(rend);

            self.go_stock.push(go_opt_0);
        }

        let mut rend = ComponentRendererText::default();
        rend.set_contents("Leave");
        let go_leave = context
            .instantiate("text.leave", Transform2D::default().set_position_01(Vector2::new(0.5, 0.5)))
            .add_component_value(rend);

        self.go_leave = Some(go_leave);

        let mut rend = ComponentRendererText::default();
        rend.set_contents("Care to buy something?");
        let go_desc = context
            .instantiate("text.leave", Transform2D::default().set_position_01(Vector2::new(0.5, 0.75)))
            .add_component_value(rend);

        self.go_desc = Some(go_desc);
    }

    fn dismiss(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext2D) {
        self.go_desc.clone().unwrap().destroy();
        self.go_leave.clone().unwrap().destroy();
        for x in &self.go_stock {
            x.destroy();
        }
    }
    fn tick(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext2D) {
        if game_state.get::<StateShop>().shop.stock.len() == 0 {
            return;
        }
        let input_state = game_state.get::<InputState>();
        if input_state.mapped.len() > 0 {
            if input_state.mapped[0]
                .get_button_or_default("move_back")
                .went_up
            {
                self.selected_index += 1;
                if self.selected_index > self.go_stock.len() as i32 {
                    self.selected_index = 0;
                }
            }
            if input_state.mapped[0]
                .get_button_or_default("move_forward")
                .went_up
            {
                self.selected_index -= 1;
                if self.selected_index < 0 {
                    self.selected_index = self.go_stock.len() as i32;
                }
            }
            if input_state.mapped[0]
                .get_button_or_default("turn_end")
                .went_up
            {
                if self.selected_index == self.go_stock.len() as i32 {
                    event_queue.enqueue_event(GameEvents::RequestLeaveExplorationRoom);
                } else {
                    let state_shop = game_state.get::<StateShop>();
                    let stock = &state_shop.shop.stock;

                    if let Some(s) = stock.get(self.selected_index as usize) {
                        event_queue.enqueue_event(GameEvents::RequestPurchase(game_state.instance_id, s.instance_id));
                    }
                }
            }

            let sin = f32::sin(game_state.get::<TimeState>().unscaled_time as f32 * 5.0);
            let scale_selected = Vector3::one() * 0.5 + Vector3::one() * sin * 0.1;
            let scale_unselected = Vector3::one() * 0.5;

            // edit selections
            for i in 0..self.go_stock.len() {
                if let Some(x) = self.go_stock.get(i) {
                    let state_shop = game_state.get::<StateShop>();
                    if state_shop.shop.stock[i].count <= 0 {
                        x.edit_component::<ComponentRendererText>(|y| {
                            y.set_contents("Out of Stock");
                        });
                    }
                    // x.edit_component::<Transform2D>(|y| {
                    //     y.scale = if i as i32 == self.selected_index { scale_selected } else { scale_unselected };
                    // });
                    x.edit_component::<Transform2D>(|y| {
                        y.scale = if i as i32 == self.selected_index { scale_selected } else { scale_unselected };
                    });
                }
            }

            // edit leave
            if let Some(x) = &self.go_leave {
                x.edit_component::<Transform2D>(|y| {
                    y.scale = if self.go_stock.len() as i32 == self.selected_index { scale_selected } else { scale_unselected };
                });
            }
        }
    }
}
