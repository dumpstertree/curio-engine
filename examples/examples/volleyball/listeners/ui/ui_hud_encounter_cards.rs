use crate::{
    Assets,
    cards::card_instance::CardInstance,
    ecs::components::component_card::ComponentCard,
    state::{
        peer::{
            state_peer_input_mode::{InputModes, StatePeerInputMode},
            state_peer_select_targets::StatePeerSelectTargets,
            state_peer_selected_card::StatePeerSelectedCards,
        },
        state_deck::{CardAttributeLifecycle, CardLocation, CardTypes, StateDeck},
    },
};
use curio_core::{
    AxisCode, ButtonCode, Color, InputAxisState, PrefabGameObject, Quaternion, Vector2, Vector3,
    collections::{event_queue::EventQueue, game_state::GameState, key_state::KeyState},
    io::asset_loader::AssetLoader,
};
use gameplay::{
    built_in::facet::{
        animator::animator_rotation_sin::AnimatorRotationSin,
        renderer::{renderer_static::RendererStatic, renderer_text::RendererText},
        renderer_common::RendererCommon,
        transform::transform2d::Transform2D,
    },
    context_2d::Context2D,
    form::Form,
    traits::ui_panel::UIPanel,
    traits_internal::{ui_common::UICommon, world_context_common::ContextCommon},
};
use std::sync::Arc;

static COLOR_SET: Color = Color::new_hex("#abff4e");
static COLOR_BUMP: Color = Color::new_hex("#4efff9");
static COLOR_SPIKE: Color = Color::new_hex("#ff4e85");
static COLOR_SPELL: Color = Color::new_hex("#f7a5f3");
static COLOR_PERSISTENT: Color = Color::new_hex("#f7c8a5");
static COLOR_OTHER: Color = Color::new_hex("#ffffffff");

#[derive(Default)]
pub struct UIHUDInstance {
    f_cards: Vec<Form>,
}
impl UIHUDInstance {
    pub fn new() -> Box<UIHUDInstance> {
        Box::new(UIHUDInstance { f_cards: Vec::new() })
    }
}
impl UIPanel for UIHUDInstance {
    fn input_button(&mut self, _button: ButtonCode, _state: KeyState) {}
    fn input_axis(&mut self, _axis: AxisCode, _state: InputAxisState) {}
}
impl UICommon for UIHUDInstance {
    fn init(&mut self) {}
    fn present(&mut self, game_state: &mut GameState, _event_queue: &mut EventQueue, context: &mut Context2D) {
        // get state
        let state_deck = game_state.get::<StateDeck>();

        // get deck
        let Some(deck) = state_deck.deck.get(&game_state.instance_id) else {
            return;
        };

        // spawn card for each card in deck
        for card in &deck.get_cards_from_all(|x| x.get_manuever_type() != CardTypes::Move) {
            self.f_cards
                .push(Self::spawn_card(game_state, context, card.clone()));
        }
    }
    fn dismiss(&mut self, _game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut Context2D) {
        for f in &self.f_cards {
            f.destroy();
        }
    }
    fn tick(&mut self, game_state: &mut GameState, _event_queue: &mut EventQueue, _context: &mut Context2D) {
        // get the state
        let state_cards = game_state.get::<StateDeck>();
        let Some(deck) = state_cards.deck.get(&game_state.instance_id) else {
            return;
        };

        // get state
        let state_input_mode = game_state.get::<StatePeerInputMode>();
        let state_peer_select_targets = game_state.get::<StatePeerSelectTargets>();
        let state_selected_card = game_state.get::<StatePeerSelectedCards>();

        //get the target positions
        let pos_deck = Vector3::new(0.1, 0.9, 0.0);
        let pos_discard = Vector3::new(0.9, 0.9, 0.0);
        let pos_out_of_play = Vector3::new(0.95, 0.9, 0.0);

        // create values to edit
        let mut order: i32;
        let mut target_pos: Vector2;
        let mut target_scl: Vector3;
        let mut target_rot: Quaternion;
        let mut rendering_enabled: bool;
        let mut animation_enabled: bool;

        // iterate over each form
        for form_card in &self.f_cards {
            // get the facet
            let Some(facet_card) = form_card.get_facet::<ComponentCard>() else {
                continue;
            };
            let Some(card_inst) = facet_card.card_instance else {
                continue;
            };
            let Some(location) = deck.get_location(card_inst.clone(), |x| x.get_manuever_type() != CardTypes::Move) else {
                continue;
            };

            match location {
                CardLocation::Deck(i) => {
                    order = 0;
                    animation_enabled = false;
                    rendering_enabled = i == 0;
                    target_pos = pos_deck.to_vector2();
                    target_rot = Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
                    target_scl = Vector3::one() * 0.25;
                }
                CardLocation::Discard(i) => {
                    order = 0;
                    animation_enabled = false;
                    rendering_enabled = i == 0;
                    target_pos = pos_discard.to_vector2();
                    target_rot = Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
                    target_scl = Vector3::one() * 0.25;
                }
                CardLocation::OutOfPlay(i) => {
                    order = 0;
                    animation_enabled = false;
                    rendering_enabled = i == 0;
                    target_pos = pos_out_of_play.to_vector2();
                    target_rot = Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
                    target_scl = Vector3::one() * 0.25;
                }
                CardLocation::Hand(i) => {
                    // edit background color
                    form_card.try_edit_facet_in_child::<RendererStatic>("background", |renderer: &mut RendererStatic| {
                        // check if can use order to change visual
                        let req_met = card_inst.has_statement(game_state, game_state.instance_id);

                        // match to manuever type
                        match &card_inst.clone().get_manuever_type() {
                            CardTypes::Serve => renderer.set_tint(COLOR_PERSISTENT * if req_met { 1.0 } else { 0.15 }),
                            CardTypes::Rest => renderer.set_tint(COLOR_PERSISTENT * if req_met { 1.0 } else { 0.15 }),
                            CardTypes::Bump => renderer.set_tint(COLOR_BUMP * if req_met { 1.0 } else { 0.15 }),
                            CardTypes::Set => renderer.set_tint(COLOR_SET * if req_met { 1.0 } else { 0.15 }),
                            CardTypes::Spike => renderer.set_tint(COLOR_SPIKE * if req_met { 1.0 } else { 0.15 }),
                            CardTypes::Spell => renderer.set_tint(COLOR_SPELL * if req_met { 1.0 } else { 0.15 }),
                            _ => renderer.set_tint(COLOR_OTHER * if req_met { 1.0 } else { 0.15 }),
                        }
                    });

                    //
                    let index = state_selected_card.index;
                    let mut y = if state_input_mode.mode == InputModes::Manuever { 0.1 } else { -0.2 };
                    if state_input_mode.mode == InputModes::Manuever && i == index {
                        y = 0.2;
                    }
                    if state_peer_select_targets.enabled.is_some() {
                        y = if i == index { 0.1 } else { -0.2 };
                    }

                    let d = (i - index).abs() as f32;
                    let direction = (i - index).signum() as f32;

                    let max_spread = 0.5; // how far cards can go
                    let falloff = 0.6; // smaller = tighter stack

                    let offset = max_spread * (1.0 - (-d * falloff).exp()) * direction;
                    let xx = 0.5 + offset;

                    order = -(i - index).abs();
                    rendering_enabled = true;
                    animation_enabled = i == index;
                    target_pos = Vector3::new(xx, y, 0.0).to_vector2();
                    target_rot = Quaternion::from_euler(Vector3::new(0.0, 0.0, (index - i) as f32 * 10.0));
                    target_scl = Vector3::one() * 0.75;
                }
            }
            // move
            form_card.try_edit_facets::<(Transform2D, AnimatorRotationSin)>(|(transform, anim)| {
                transform.render_order = order;
                transform.position = Vector2::lerp(transform.position, target_pos, 0.2);
                transform.rotation = Quaternion::slerp(transform.rotation, target_rot, 0.2);
                transform.scale = Vector3::lerp(transform.scale, target_scl, 0.2);
                anim.set_enabled(animation_enabled);
            });
            // set rendering
            form_card.try_edit_facet_in_child::<RendererStatic>("background", |x| {
                x.set_enabled(rendering_enabled);
            });
            form_card.try_edit_facet_in_child::<RendererText>("title", |x| {
                x.set_enabled(rendering_enabled);
            });
            form_card.try_edit_facet_in_child::<RendererText>("description", |x| {
                x.set_enabled(rendering_enabled);
            });
            form_card.try_edit_facet_in_child::<RendererText>("cost", |x| {
                x.set_enabled(rendering_enabled);
            });
            form_card.try_edit_facet_in_child::<RendererText>("type", |x| {
                x.set_enabled(rendering_enabled);
            });
        }
    }
}
impl UIHUDInstance {
    fn spawn_card(game_state: &mut GameState, world: &mut Context2D, card_inst: Arc<CardInstance>) -> Form {
        //
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
            x.set_contents(&format!("{}", card_inst.get_cost(game_state, game_state.instance_id)));
        });

        // edit component on self
        f_card.try_edit_facet::<ComponentCard>(|x| {
            x.card_instance = Some(card_inst);
        });

        // pass back
        return f_card;
    }
}
