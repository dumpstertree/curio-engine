use curio_core::{
    AxisCode, ButtonCode, Color, InputAxisState, PrefabGameObject, Vector2,
    collections::{event_queue::EventQueue, ledger::Ledger, key_state::KeyState},
    io::asset_loader::AssetLoader,
};
use std::sync::Arc;

use gameplay::{
    built_in::facet::{
        renderer::{renderer_static::RendererStatic, renderer_text::RendererText},
        renderer_common::RendererCommon,
        transform::transform2d::Transform2D,
    },
    context_2d::Context2D,
    form::Form,
    traits::ui_panel::UIPanel,
    traits_internal::{ui_common::UICommon, world_context_common::ContextCommon},
};

use crate::{
    Assets,
    cards::card_instance::CardInstance,
    ecs::components::component_card::ComponentCard,
    state::{
        host::state_play_history::StatePlayHistory,
        state_deck::{CardAttributeLifecycle, CardTypes},
    },
};

static COLOR_SET: Color = Color::new_hex("#abff4e");
static COLOR_BUMP: Color = Color::new_hex("#4efff9");
static COLOR_SPIKE: Color = Color::new_hex("#ff4e85");
static COLOR_SPELL: Color = Color::new_hex("#f7a5f3");
static COLOR_PERSISTENT: Color = Color::new_hex("#f7c8a5");
static COLOR_OTHER: Color = Color::new_hex("#ffffffff");

pub struct UIHUD {
    history_len: i32,
    open_gos: Vec<(Form, f64)>,
}
impl UIHUD {
    pub fn new() -> Box<UIHUD> {
        Box::new(UIHUD { history_len: 0, open_gos: Vec::new() })
    }
}
impl UIPanel for UIHUD {
    fn input_button(&mut self, _button: ButtonCode, _state: KeyState) {}
    fn input_axis(&mut self, _axis: AxisCode, _state: InputAxisState) {}
}
impl UICommon for UIHUD {
    fn init(&mut self) {}
    fn present(&mut self, _ledger: &mut Ledger, _event_queue: &mut EventQueue, _context: &mut Context2D) {}
    fn dismiss(&mut self, _ledger: &mut Ledger, _event_queue: &mut EventQueue, _context: &mut Context2D) {}
    fn tick(&mut self, ledger: &mut Ledger, _event_queue: &mut EventQueue, context: &mut Context2D) {
        for i in (0..self.open_gos.len()).rev() {
            let f = &self.open_gos[i].0;
            let t = &self.open_gos[i].1;
            let dt = ledger.time().scaled_time - t;
            f.edit_facet::<Transform2D>(|x| {
                x.position = Vector2::new(0.75, Self::remap(Self::ease_in_hold_ease_out(dt as f32 / 2.0), 0.0, 1.0, -0.35, 1.35));
            });

            if ledger.time().scaled_time - t > 2.0 {
                let f = self.open_gos.remove(i);
                f.0.destroy();
            }
        }

        let state_play_history = ledger.get::<StatePlayHistory>();
        if state_play_history.history.len() as i32 == self.history_len {
            return;
        }

        if state_play_history
            .history
            .last()
            .unwrap()
            .1
            .get_manuever_type()
            == CardTypes::Move
        {
            return;
        };

        if state_play_history.history.last().unwrap().0 == ledger.instance_id {
            return;
        };

        let x = Self::spawn_card(&ledger, context, state_play_history.history.last().unwrap().1.clone());
        x.try_edit_facet::<Transform2D>(|x| {
            x.position = Vector2::new(-100.0, 100.0);
        });
        self.open_gos.push((x, ledger.time().scaled_time));

        self.history_len = state_play_history.history.len() as i32;

        println!("create");
    }
}
impl UIHUD {
    fn spawn_card(ledger: &Ledger, world: &mut Context2D, card_inst: Arc<CardInstance>) -> Form {
        let mut desc = card_inst.get_master().description.clone();
        for life in card_inst.get_attributes_lifecycle() {
            match life {
                CardAttributeLifecycle::Quick => desc = desc + ".QUICK. ",
                CardAttributeLifecycle::Exhuast => desc = desc + ".EXHUAST. ",
                CardAttributeLifecycle::Exile => desc = desc + ".EXILE. ",
                CardAttributeLifecycle::Linger => desc = desc + ".LINGER. ",
                CardAttributeLifecycle::Light => desc = desc + ".LIGHT. ",
                CardAttributeLifecycle::Persistant => desc = desc + ".PERSISTANT. ",
                CardAttributeLifecycle::Consume => desc = desc + ".CONSUME. ",
                _ => {}
            }
        }

        // spawn prefab
        let f_card = world.spawn_prefab_recursive(&AssetLoader::load_asset::<PrefabGameObject>(&Assets::PrefabUICard.into()));

        // edit component on child
        f_card.try_edit_facet_in_child::<RendererText>("description", |x| {
            x.set_contents(&format!("{}", card_inst.get_description()));
        });
        f_card.try_edit_facet_in_child::<RendererText>("type", |x| {
            x.set_contents(&format!("{}", card_inst.get_manuever_type()));
        });
        f_card.try_edit_facet_in_child::<RendererText>("title", |x| {
            x.set_contents(&format!("{}", card_inst.get_title()));
        });
        f_card.try_edit_facet_in_child::<RendererText>("cost", |x| {
            x.set_contents(&format!("{}", card_inst.get_cost(ledger, ledger.instance_id)));
        });

        f_card.try_edit_facet_in_child::<RendererStatic>("background", |renderer: &mut RendererStatic| {
            // match to manuever type
            match &card_inst.clone().get_manuever_type() {
                CardTypes::Serve => renderer.set_tint(COLOR_PERSISTENT),
                CardTypes::Rest => renderer.set_tint(COLOR_PERSISTENT),
                CardTypes::Bump => renderer.set_tint(COLOR_BUMP),
                CardTypes::Set => renderer.set_tint(COLOR_SET),
                CardTypes::Spike => renderer.set_tint(COLOR_SPIKE),
                CardTypes::Spell => renderer.set_tint(COLOR_SPELL),
                _ => renderer.set_tint(COLOR_OTHER),
            }
        });

        // edit component on self
        f_card.try_edit_facet::<ComponentCard>(|x| {
            x.card_instance = Some(card_inst);
        });

        return f_card;
    }
    pub fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
        (value - from_min) / (from_max - from_min) * (to_max - to_min) + to_min
    }

    pub fn ease_in_hold_ease_out(t: f32) -> f32 {
        // Clamp input
        let t = t.clamp(0.0, 1.0);

        // Portion of time spent easing in and out
        let ease_portion = 0.2; // 20% in, 60% hold, 20% out

        let hold_start = ease_portion;
        let hold_end = 1.0 - ease_portion;

        if t < hold_start {
            // Ease-in from 0 → 0.5
            let x = t / hold_start;
            0.5 * Self::ease_in_cubic(x)
        } else if t < hold_end {
            // Hold at 0.5
            0.5
        } else {
            // Ease-out from 0.5 → 1.0
            let x = (t - hold_end) / ease_portion;
            0.5 + 0.5 * Self::ease_out_cubic(x)
        }
    }

    fn ease_in_cubic(x: f32) -> f32 {
        x * x * x
    }

    fn ease_out_cubic(x: f32) -> f32 {
        1.0 - (1.0 - x).powi(3)
    }
}
