use curio_core::{
    built_in::record::state_time::TimeState,
    collections::{color::Color, event_queue::EventQueue, game_state::GameState, input_cursor::InputAxisState, key_state::KeyState, quaternion::Quaternion, vector2::Vector2, vector3::Vector3},
    input::{axis_code::AxisCode, key_code::ButtonCode},
    io::asset_loader::AssetLoader,
};
use std::sync::Arc;

use system_component_default_gameplay::{
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
    open_gos: Option<(Form, Vec<Form>)>,
    last_open: f64,
}
impl UIHUD {
    pub fn new() -> Box<UIHUD> {
        Box::new(UIHUD { history_len: 0, open_gos: None, last_open: 0.0 })
    }
}
impl UIPanel for UIHUD {
    fn input_button(&mut self, _button: ButtonCode, _state: KeyState) {}
    fn input_axis(&mut self, _axis: AxisCode, _state: InputAxisState) {}
}
impl UICommon for UIHUD {
    fn init(&mut self) {}
    fn present(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut Context2D) {}
    fn dismiss(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut Context2D) {}
    fn tick(&mut self, game_state: &mut GameState, _event_queue: &mut EventQueue, context: &mut Context2D) {
        if let Some(open_gos) = &self.open_gos {
            let dt = game_state.get::<TimeState>().unscaled_time - self.last_open;
            open_gos.0.edit_facet::<Transform2D>(|x| {
                x.position = Vector2::new(0.75, Self::remap(Self::ease_in_hold_ease_out(dt as f32 / 2.5), 0.0, 1.0, -0.35, 1.35));
            });

            if dt >= 2.5 {
                for x in &open_gos.1 {
                    x.destroy();
                }
                println!("destroy");
                self.open_gos = None;
            }
        }

        let state_play_history = game_state.get::<StatePlayHistory>();
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

        if state_play_history.history.last().unwrap().0 == game_state.instance_id {
            return;
        };

        self.open_gos = Some(Self::spawn_card(&game_state, context, state_play_history.history.last().unwrap().1.clone()));
        self.last_open = game_state.get::<TimeState>().unscaled_time;

        self.history_len = state_play_history.history.len() as i32;

        println!("create");
    }
}
impl UIHUD {
    fn spawn_card(game_state: &GameState, world: &mut Context2D, card_inst: Arc<CardInstance>) -> (Form, Vec<Form>) {
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
        let f_card = world.spawn_prefab_recursive(&AssetLoader::load_prefab(&Assets::PrefabUICard.into()));

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
            x.set_contents(&format!("{}", card_inst.get_cost(game_state, game_state.instance_id)));
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

        // pass back
        return (f_card.clone(), vec![f_card]);
        // parent

        // let parent = world
        //     .instantiate(
        //         "",
        //         Transform2D::default()
        //             .set_render_order(0)
        //             .set_position_01(Vector2::new(-1.0, -1.0)),
        //     )
        //     .add_component_value(Renderer::default().set_asset(Some(asset.clone())));

        // let text = world
        //     .instantiate(
        //         "",
        //         Transform2D::default()
        //             .set_parent(Some(parent.clone()))
        //             .set_render_order(1)
        //             .set_position_01(Vector2::new(0.5, 0.5)),
        //     )
        //     .add_component_value(ComponentRendererText::default());

        // (parent.clone(), vec![parent, text])

        // create description
        let asset = AssetLoader::load_model_static_from_database(&Assets::Card.into());
        let parent = world
            .spawn("", Transform2D::default())
            .add_facet(RendererStatic::default().set_asset(Some(asset.clone())))
            .add_facet(ComponentCard::default().set_instance(card_inst.clone()));

        let mut desc = card_inst.get_master().description.clone();
        for life in card_inst.get_attributes_lifecycle() {
            match life {
                crate::state::state_deck::CardAttributeLifecycle::Quick => desc = desc + ".QUICK. ",
                crate::state::state_deck::CardAttributeLifecycle::Exhuast => desc = desc + ".EXHUAST. ",
                crate::state::state_deck::CardAttributeLifecycle::Exile => desc = desc + ".EXILE. ",
                crate::state::state_deck::CardAttributeLifecycle::Linger => desc = desc + ".LINGER. ",
                crate::state::state_deck::CardAttributeLifecycle::Light => desc = desc + ".LIGHT. ",
                crate::state::state_deck::CardAttributeLifecycle::Persistant => desc = desc + ".PERSISTANT. ",
                crate::state::state_deck::CardAttributeLifecycle::Reliable(_) => {}
                crate::state::state_deck::CardAttributeLifecycle::Light => {}
                crate::state::state_deck::CardAttributeLifecycle::Heavy => {}
                crate::state::state_deck::CardAttributeLifecycle::Consume => desc = desc + ".CONSUME. ",
            }
        }
        let mut r = RendererText::default();
        r.set_bounds(Vector2::new(0.25, 0.2));
        r.set_font_size(0.02);
        r.set_contents(&desc);
        let e0: Form = world
            .spawn(
                "",
                Transform2D::default()
                    .set_render_order(1)
                    .set_position_01(Vector2::new(0.5, 0.375))
                    .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 0.0, 0.0)))
                    .set_parent(Some(parent.clone())),
            )
            .add_facet(r);

        // r.set_parent(Some(parent.clone()));
        // create title
        let mut r = RendererText::default();
        r.set_bounds(Vector2::new(0.5, 0.2));
        r.set_font_size(0.03);
        r.set_contents(&card_inst.get_title());
        // r.set_parent(Some(parent.clone()));
        let e1 = world
            .spawn(
                "",
                Transform2D::default()
                    .set_render_order(1)
                    .set_position_01(Vector2::new(0.5, 0.69))
                    .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 0.0, 0.0)))
                    .set_parent(Some(parent.clone())),
            )
            .add_facet(r);
        // create type
        let mut r = RendererText::default();
        r.set_bounds(Vector2::new(0.25, 0.2));
        r.set_font_size(0.02);
        r.set_contents(&format!("{}", card_inst.get_manuever_type()));
        // r.set_parent(Some(parent.clone()));
        let e2 = world
            .spawn(
                "",
                Transform2D::default()
                    .set_render_order(1)
                    .set_position_01(Vector2::new(0.5, 0.45))
                    .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 0.0, 0.0)))
                    .set_parent(Some(parent.clone())),
            )
            .add_facet(r);
        // create cost
        let mut r = RendererText::default();
        r.set_bounds(Vector2::new(0.25, 0.2));
        r.set_font_size(0.03);
        r.set_contents(
            &card_inst
                .get_cost(&game_state, game_state.instance_id)
                .to_string(),
        );
        // r.set_contents("0");
        // r.set_parent(Some(parent.clone()));
        let e3 = world
            .spawn(
                "",
                Transform2D::default()
                    .set_render_order(1)
                    .set_position_01(Vector2::new(0.58, 0.27))
                    .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 0.0, 0.0)))
                    .set_parent(Some(parent.clone())),
            )
            .add_facet(r);

        parent.edit_facet::<RendererStatic>(|rend| {
            let col_spell = Color::new_hex("#f7a5f3");
            let col_persistent = Color::new_hex("#f7c8a5");
            let col_bump = Color::new_hex("#4efff9");
            let col_set = Color::new_hex("#abff4e");
            let col_spike = Color::new_hex("#ff4e85");

            // let mut cur_tint = renderer.get_tint();

            match &card_inst.clone().get_manuever_type() {
                CardTypes::Serve => rend.set_tint(col_persistent),
                CardTypes::Rest => rend.set_tint(col_persistent),
                CardTypes::Bump => rend.set_tint(col_bump),
                CardTypes::Set => rend.set_tint(col_set),
                CardTypes::Spike => rend.set_tint(col_spike),
                CardTypes::Move => rend.set_tint(Color::white()),
                CardTypes::Spell => rend.set_tint(col_spell),
                CardTypes::Food => rend.set_tint(Color::white()),
                // renderer.set_tint(cur_tint);
            }
        });

        (parent.clone(), vec![parent, e0, e1, e2, e3])
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
