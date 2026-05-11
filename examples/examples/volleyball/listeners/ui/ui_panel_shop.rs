use curio_core::{
    AxisCode, ButtonCode, InputAxisState, KeyState, PrefabGameObject,
    built_in::record::sys_record_input::SysRecordInput,
    collections::{event_queue::Nerve, ledger::Ledger},
    io::asset_loader::AssetLoader,
};

use gameplay::{
    built_in::facet::{
        animator::{animator_rotation_sin::AnimatorRotationSin, animator_scale_sin::AnimatorScaleSin},
        renderer::renderer_text::RendererText,
    },
    context_2d::Context2D,
    form::Form,
    traits::ui_panel::UIPanel,
    traits_internal::{ui_common::UICommon, world_context_common::ContextCommon},
};

use crate::{
    Assets,
    cards::card_library::CardLibrary,
    game_events::GameEvents,
    state::host::state_shop::{StateShop, Stock, StockItems},
};

pub struct UIPanelInstance {
    selected_index: i32,
    f_ui: Option<Form>,
    stock: Vec<Stock>,
}
impl UIPanelInstance {
    pub fn new() -> Box<UIPanelInstance> {
        Box::new(UIPanelInstance { selected_index: 0, f_ui: None, stock: Vec::new() })
    }
}
impl UIPanel for UIPanelInstance {
    fn input_button(&mut self, _button: ButtonCode, _state: KeyState) {}
    fn input_axis(&mut self, _axis: AxisCode, _state: InputAxisState) {}
}
impl UICommon for UIPanelInstance {
    fn init(&mut self) {}

    fn present(&mut self, ledger: &mut Ledger, _event_queue: &mut Nerve, context: &mut Context2D) {
        //
        let mut stock_names = Vec::new();
        let state_store = ledger.read::<StateShop>();
        for i in 0..state_store.shop.stock.len() {
            let stock = state_store.shop.stock.get(i).unwrap();
            match &stock.item {
                StockItems::Card(card_id) => stock_names.push(format!("{} : {} x{}", CardLibrary::get_master_card(&card_id).title, stock.cost, stock.count)),
                StockItems::Relic(_) => todo!(),
            };
        }

        // spawn the prefab
        let f_ui = context.spawn_prefab_recursive(&AssetLoader::load_asset::<PrefabGameObject>(&Assets::PrefabUIPanelShop.into()));

        // edit the description
        if let Some(f_description) = f_ui.get_child("description") {
            f_description.edit_facet::<RendererText>(|x| {
                x.set_contents("Care to buy?");
            });
        }
        // edit the button 0
        f_ui.try_edit_facet_in_child::<RendererText>("option_0", |x| {
            x.set_contents(&stock_names[0]);
        });
        // edit the button 0
        f_ui.try_edit_facet_in_child::<RendererText>("option_1", |x| {
            x.set_contents(&stock_names[1]);
        });
        // edit the button 0
        f_ui.try_edit_facet_in_child::<RendererText>("option_2", |x| {
            x.set_contents(&stock_names[2]);
        });
        // edit the button 0
        f_ui.try_edit_facet_in_child::<RendererText>("option_3", |x| {
            x.set_contents("Leave");
        });

        self.f_ui = Some(f_ui);
    }

    fn dismiss(&mut self, _ledger: &mut Ledger, _event_queue: &mut Nerve, _context: &mut Context2D) {
        if let Some(f_ui) = &self.f_ui {
            f_ui.destroy();
        }
    }
    fn tick(&mut self, ledger: &mut Ledger, event_queue: &mut Nerve, _context: &mut Context2D) {
        // no items
        if ledger.read::<StateShop>().shop.stock.len() == 0 {
            return;
        }

        let mut is_dirty = false;
        let input_state = ledger.read::<SysRecordInput>();
        if input_state.mapped.len() > 0 {
            if input_state.mapped[0]
                .get_button_or_default("move_back")
                .went_up
            {
                self.selected_index += 1;
                if self.selected_index > 3 as i32 {
                    self.selected_index = 0;
                }
                is_dirty = true;
            }
            if input_state.mapped[0]
                .get_button_or_default("move_forward")
                .went_up
            {
                self.selected_index -= 1;
                if self.selected_index < 0 {
                    self.selected_index = 3 as i32;
                }
                is_dirty = true;
            }
            if input_state.mapped[0]
                .get_button_or_default("turn_end")
                .went_up
            {
                if self.selected_index == 3 as i32 {
                    event_queue.enqueue_event(GameEvents::RequestLeaveExplorationRoom);
                } else {
                    let state_shop = ledger.read::<StateShop>();
                    let stock = &state_shop.shop.stock;

                    if let Some(s) = stock.get(self.selected_index as usize) {
                        event_queue.enqueue_event(GameEvents::RequestPurchase(ledger.network.me().guid, s.instance_id));
                    }
                }
                is_dirty = true;
            }

            let state_store = ledger.read::<StateShop>();

            //
            if !is_dirty && self.stock == state_store.shop.stock {
                return;
            }

            self.stock = state_store.shop.stock.clone();

            if let Some(f_ui) = &self.f_ui {
                let mut stock_names = Vec::new();
                let state_store = ledger.read::<StateShop>();

                for i in 0..state_store.shop.stock.len() {
                    let stock = state_store.shop.stock.get(i).unwrap();
                    if stock.count <= 0 {
                        stock_names.push("OUT OF STOCK".to_string());
                        continue;
                    }
                    match &stock.item {
                        StockItems::Card(card_id) => stock_names.push(format!("{} : {} x{}", CardLibrary::get_master_card(&card_id).title, stock.cost, stock.count)),
                        StockItems::Relic(_) => todo!(),
                    };
                }

                f_ui.try_edit_facets_in_child::<(RendererText, AnimatorScaleSin, AnimatorRotationSin)>("option_0", |(rend, a_scale, a_rot)| {
                    a_scale.set_enabled(self.selected_index == 0);
                    a_rot.set_enabled(self.selected_index == 0);
                    rend.set_contents(&stock_names[0]);
                });
                f_ui.try_edit_facets_in_child::<(RendererText, AnimatorScaleSin, AnimatorRotationSin)>("option_1", |(rend, a_scale, a_rot)| {
                    a_scale.set_enabled(self.selected_index == 1);
                    a_rot.set_enabled(self.selected_index == 1);
                    rend.set_contents(&stock_names[1]);
                });
                f_ui.try_edit_facets_in_child::<(RendererText, AnimatorScaleSin, AnimatorRotationSin)>("option_2", |(rend, a_scale, a_rot)| {
                    a_scale.set_enabled(self.selected_index == 2);
                    a_rot.set_enabled(self.selected_index == 2);
                    rend.set_contents(&stock_names[2]);
                });
                f_ui.try_edit_facets_in_child::<(AnimatorScaleSin, AnimatorRotationSin)>("option_3", |(a_scale, a_rot)| {
                    a_scale.set_enabled(self.selected_index == 3);
                    a_rot.set_enabled(self.selected_index == 3);
                });
            }
        }
    }
}
