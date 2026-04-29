use curio_core::{
    AxisCode, ButtonCode, InputAxisState, Vector2, Vector3,
    built_in::record::{sys_record_input::SysRecordInput, sys_record_time::SysRecordTime},
    collections::{event_queue::EventQueue, ledger::Ledger, key_state::KeyState},
};

use gameplay::{
    built_in::facet::{renderer::renderer_text::RendererText, transform::transform2d::Transform2D},
    context_2d::Context2D,
    form::Form,
    traits::ui_panel::UIPanel,
    traits_internal::ui_common::UICommon,
};

use crate::{game_events::GameEvents, state::host::state_currency::StateCurrency};

pub struct UIPanelMedic {
    selected_index: i32,
    go_desc: Option<Form>,
    go_opt_0: Option<Form>,
    go_opt_1: Option<Form>,
}
impl UIPanelMedic {
    pub fn new() -> Box<UIPanelMedic> {
        Box::new(UIPanelMedic {
            selected_index: 0,
            go_desc: None,
            go_opt_0: None,
            go_opt_1: None,
        })
    }
}
impl UIPanel for UIPanelMedic {
    fn input_button(&mut self, _button: ButtonCode, _state: KeyState) {}

    fn input_axis(&mut self, _axis: AxisCode, _state: InputAxisState) {}
}
impl UICommon for UIPanelMedic {
    fn init(&mut self) {}

    fn present(&mut self, _ledger: &mut Ledger, _event_queue: &mut EventQueue, context: &mut Context2D) {
        // create obj
        let go_desc = context
            .spawn("text.description", Transform2D::default().set_position_01(Vector2::new(0.5, 0.5)))
            .add_facet_default::<RendererText>();

        let go_opt_0 = context
            .spawn("text.option_0", Transform2D::default().set_position_01(Vector2::new(0.5, 0.4)))
            .add_facet_default::<RendererText>();

        let go_opt_1 = context
            .spawn("text.option_1", Transform2D::default().set_position_01(Vector2::new(0.5, 0.3)))
            .add_facet_default::<RendererText>();

        // save
        self.go_desc = Some(go_desc);
        self.go_opt_0 = Some(go_opt_0);
        self.go_opt_1 = Some(go_opt_1);
    }

    fn dismiss(&mut self, _ledger: &mut Ledger, _event_queue: &mut EventQueue, _context: &mut Context2D) {
        self.go_desc.clone().unwrap().destroy();
        self.go_opt_0.clone().unwrap().destroy();
        self.go_opt_1.clone().unwrap().destroy();
    }

    fn tick(&mut self, ledger: &mut Ledger, event_queue: &mut EventQueue, _context: &mut Context2D) {
        let input_state = ledger.read::<SysRecordInput>();
        if input_state.mapped.len() > 0 {
            if input_state.mapped[0]
                .get_button_or_default("move_forward")
                .went_up
            {
                self.selected_index += 1;
                if self.selected_index > 1 {
                    self.selected_index = 0;
                }
            }
            if input_state.mapped[0]
                .get_button_or_default("move_back")
                .went_up
            {
                self.selected_index -= 1;
                if self.selected_index < 0 {
                    self.selected_index = 1;
                }
            }
            if input_state.mapped[0]
                .get_button_or_default("turn_end")
                .went_up
            {
                if self.selected_index == 0 {
                    event_queue.enqueue_event(GameEvents::RequestHeal(ledger.instance_id));
                }
                if self.selected_index == 1 {
                    event_queue.enqueue_event(GameEvents::RequestLeaveExplorationRoom);
                }
            }
        }

        let sin = f32::sin(ledger.read::<SysRecordTime>().unscaled_time as f32 * 5.0);

        if let Some(a) = &self.go_desc {
            // edit text renderer
            a.edit_facet::<RendererText>(|x| {
                x.set_contents(&format!("Heal? You have {} of {}", 0, 10));
            });
        }
        if let Some(a) = &self.go_opt_0 {
            a.edit_facet::<Transform2D>(|x| x.scale = Vector3::one() * 0.5 + Vector3::one() * if self.selected_index == 0 { sin * 0.1 } else { 0.0 });
            // edit text renderer
            a.edit_facet::<RendererText>(|x| {
                let state_currency = ledger.read::<StateCurrency>();
                if state_currency.currency >= 100 {
                    x.set_contents(&format!("Heal +1 for 100g"));
                } else {
                    x.set_contents(&format!("Not enough money.Need 100 have {}", state_currency.currency));
                }
            });
        }
        if let Some(a) = &self.go_opt_1 {
            a.edit_facet::<Transform2D>(|x| x.scale = Vector3::one() * 0.5 + Vector3::one() * if self.selected_index == 1 { sin * 0.1 } else { 0.0 });
            // edit text renderer
            a.edit_facet::<RendererText>(|x| {
                x.set_contents("Leave");
            });
        }
    }
}
