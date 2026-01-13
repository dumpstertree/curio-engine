use core::{
    collections::{color::Color, event_queue::EventQueue, game_state::GameState, quaternion::Quaternion, vector2::Vector2, vector3::Vector3},
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
    traits_internal::ui_common::UICommon,
};

use crate::{
    AssetMappingUIDs,
    cards::card_instance::CardInstance,
    ecs::components::component_card::ComponentCard,
    state::{
        peer::{
            state_peer_entity_ids::{EntityIDTypes, StateEntityIDs},
            state_peer_input_mode::{InputModes, StatePeerInputMode},
            state_peer_select_targets::StatePeerSelectTargets,
            state_peer_selected_card::StatePeerSelectedCards,
        },
        state_deck::{self, CardTypes, Deck, StateDeck},
        state_teams::{StateTeamAssignments, Teams},
    },
};

#[derive(Default)]
pub struct UIHUDInstance {
    open_gos: Option<Vec<Form>>,
}
impl UIHUDInstance {
    pub fn new() -> Box<UIHUDInstance> {
        Box::new(UIHUDInstance { open_gos: None })
    }
}
impl UIPanel for UIHUDInstance {
    fn input_button(&mut self, _button: core::input::key_code::ButtonCode, _state: core::collections::key_state::KeyState) {}
    fn input_axis(&mut self, _axis: core::input::axis_code::AxisCode, _state: core::collections::input_cursor::InputAxisState) {}
}
impl UICommon for UIHUDInstance {
    fn init(&mut self) {}
    fn present(&mut self, game_state: &mut GameState, _event_queue: &mut EventQueue, context: &mut Context2D) {
        self.open_gos = Some(Self::spawn_ui_cards(game_state, context));
    }
    fn dismiss(&mut self, game_state: &mut GameState, _event_queue: &mut EventQueue, context: &mut Context2D) {
        Self::despawn_ui_cards(game_state, context);
    }
    fn tick(&mut self, game_state: &mut GameState, _event_queue: &mut EventQueue, context: &mut Context2D) {
        let Some(open_gos) = &self.open_gos else {
            return;
        };

        let state_cards = game_state.get::<StateDeck>();
        let Some(deck) = state_cards.deck.get(&game_state.instance_id) else {
            return;
        };

        let pos_deck = Vector3::new(0.1, 0.9, 0.0);
        let pos_discard = Vector3::new(0.9, 0.9, 0.0);
        let pos_out_of_play = Vector3::new(0.95, 0.9, 0.0);

        for go in open_gos {
            let c = go.get_facet::<ComponentCard>().unwrap();
            let card = c.card_instance.unwrap();
            let location = deck
                .get_location(card.clone(), |x| x.get_manuever_type() != CardTypes::Move)
                .unwrap();
            match location {
                crate::state::state_deck::CardLocation::Deck(i) => {
                    go.edit_facet::<Transform2D>(|x| {
                        x.position = Vector3::lerp(x.position.to_vector3(0.0), pos_deck, 0.2).to_vector2();
                        x.scale = Vector3::one() * 0.25;
                    });
                    go.edit_facet::<RendererStatic>(|x| {
                        x.set_enabled(i == 0);
                    });
                }
                crate::state::state_deck::CardLocation::Discard(i) => {
                    go.edit_facet::<Transform2D>(|x| {
                        x.position = Vector3::lerp(x.position.to_vector3(0.0), pos_discard, 0.2).to_vector2();
                        x.scale = Vector3::one() * 0.25;
                    });
                    go.edit_facet::<RendererStatic>(|x| {
                        x.set_enabled(i == 0);
                    });
                }
                crate::state::state_deck::CardLocation::Hand(i) => {
                    let state_selected_card = game_state.get::<StatePeerSelectedCards>();
                    let state_mode = game_state.get::<StatePeerInputMode>();
                    let state_peer_select_targets = game_state.get::<StatePeerSelectTargets>();

                    let index = state_selected_card.index;
                    let mut y = if state_mode.mode == InputModes::Manuever { 0.1 } else { -0.2 };
                    if state_mode.mode == InputModes::Manuever && i == index {
                        y = 0.2;
                    }
                    if state_peer_select_targets.enabled.is_some() {
                        y = if i == index { 0.1 } else { -0.2 };
                    }

                    let xx = 0.5 + (i - index) as f32 * 0.2;

                    go.edit_facet::<RendererStatic>(|renderer: &mut RendererStatic| {
                        let is_met = card.has_statement(game_state, game_state.instance_id);
                        let col_spell = Color::new_hex("#f7a5f3");
                        let col_persistent = Color::new_hex("#f7c8a5");
                        let col_bump = Color::new_hex("#4efff9");
                        let col_set = Color::new_hex("#abff4e");
                        let col_spike = Color::new_hex("#ff4e85");

                        if is_met {
                            match &card.clone().get_manuever_type() {
                                state_deck::CardTypes::Serve => renderer.set_tint(col_persistent),
                                state_deck::CardTypes::Rest => renderer.set_tint(col_persistent),
                                state_deck::CardTypes::Bump => renderer.set_tint(col_bump),
                                state_deck::CardTypes::Set => renderer.set_tint(col_set),
                                state_deck::CardTypes::Spike => renderer.set_tint(col_spike),
                                state_deck::CardTypes::Move => renderer.set_tint(Color::white()),
                                state_deck::CardTypes::Spell => renderer.set_tint(col_spell),
                                state_deck::CardTypes::Food => renderer.set_tint(Color::white()),
                            }
                        } else {
                            match &card.clone().get_manuever_type() {
                                state_deck::CardTypes::Serve => renderer.set_tint(col_persistent * 0.15),
                                state_deck::CardTypes::Rest => renderer.set_tint(col_persistent * 0.15),
                                state_deck::CardTypes::Bump => renderer.set_tint(col_bump * 0.15),
                                state_deck::CardTypes::Set => renderer.set_tint(col_set * 0.15),
                                state_deck::CardTypes::Spike => renderer.set_tint(col_spike * 0.15),
                                state_deck::CardTypes::Move => renderer.set_tint(Color::white() * 0.15),
                                state_deck::CardTypes::Spell => renderer.set_tint(col_persistent * 0.15),
                                state_deck::CardTypes::Food => renderer.set_tint(col_persistent * 0.15),
                            }
                        }
                    });
                    go.edit_facet::<Transform2D>(|x| {
                        x.position = Vector3::lerp(x.position.to_vector3(0.0), Vector3::new(xx, y, 0.0), 0.2).to_vector2();
                        x.scale = Vector3::one() * 0.75;
                    });
                }
                crate::state::state_deck::CardLocation::OutOfPlay(i) => {
                    go.edit_facet::<Transform2D>(|x| {
                        x.position = Vector3::lerp(x.position.to_vector3(0.0), pos_out_of_play, 0.2).to_vector2();
                        x.scale = Vector3::one() * 0.25;
                    });
                    go.edit_facet::<RendererStatic>(|x| {
                        x.set_enabled(i == 0);
                    });
                }
            }
        }
    }
}
impl UIHUDInstance {
    fn spawn_card(game_state: &mut GameState, world: &mut Context2D, x: Arc<CardInstance>) -> (Form, Vec<Form>) {
        // card asset
        let asset = AssetLoader::load_model_static_from_database(AssetMappingUIDs::Card.uid());

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
        let asset = AssetLoader::load_model_static_from_database(AssetMappingUIDs::Card.uid());
        let parent = world
            .spawn("", Transform2D::default())
            .add_facet(RendererStatic::default().set_asset(Some(asset.clone())))
            .add_facet(ComponentCard::default().set_instance(x.clone()));

        let mut desc = x.get_master().description.clone();
        for life in x.get_attributes_lifecycle() {
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
        // r.set_parent(Some(parent.clone()));
        let mut e0: Form = world
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
        r.set_contents(&x.get_title());
        // r.set_parent(Some(parent.clone()));
        let mut e1 = world
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
        r.set_contents(&format!("{}", x.get_manuever_type()));
        // r.set_parent(Some(parent.clone()));
        let mut e2 = world
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
        r.set_contents(&x.get_cost(&game_state, game_state.instance_id).to_string());
        // r.set_contents("0");
        // r.set_parent(Some(parent.clone()));
        let mut e3 = world
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

            match &x.clone().get_manuever_type() {
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

        e0.set_parent(Some(parent.clone()));
        e1.set_parent(Some(parent.clone()));
        e2.set_parent(Some(parent.clone()));
        e3.set_parent(Some(parent.clone()));

        (parent.clone(), vec![parent, e0, e1, e2, e3])
    }
    fn despawn_ui_cards(game_state: &mut GameState, world: &mut Context2D) {
        let id = EntityIDTypes::UICards;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            // let _ = world.despawn(e);
            e.destroy();
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    pub fn spawn_ui_cards(game_state: &mut GameState, world: &mut Context2D) -> Vec<Form> {
        let state_deck = game_state.get::<StateDeck>();
        let state_teams = game_state.get::<StateTeamAssignments>();

        let my_deck: &Deck;
        if let Some(deck) = state_deck.deck.get(&game_state.instance_id) {
            my_deck = deck
        } else if let Some(_) = state_deck
            .deck
            .get(&state_teams.team_assignments.get(&Teams::Red).unwrap()[0])
        {
            // my_deck = deck;
            return vec![];
        } else {
            return vec![];
        }

        let mut parent = Vec::new();
        for card in &my_deck.get_cards_from_all(|x| x.get_manuever_type() != CardTypes::Move) {
            let go = Self::spawn_card(game_state, world, card.clone());
            parent.push(go.0);
        }

        parent
    }
    // fn spawn_card(world: &mut WorldContext, x: Arc<CardInstance>, rotation: Quaternion, game_state: &mut GameState) {
    //     let asset = AssetLoader::load_model_static_from_database(AssetMappingUIDs::Card.uid());
    //     let parent = world
    //         .instantiate("", Transform::default().set_rotation(rotation))
    //         .add_component_value(Renderer::default().set_asset(Some(asset.clone())))
    //         .add_component_value(ComponentCard::default().set_instance(x.clone()));

    //     // create description
    //     let mut desc = x.get_master().description.clone();
    //     for life in x.get_attributes_lifecycle() {
    //         match life {
    //             crate::state::state_deck::CardAttributeLifecycle::Quick => desc = desc + ".QUICK. ",
    //             crate::state::state_deck::CardAttributeLifecycle::Exhuast => desc = desc + ".EXHUAST. ",
    //             crate::state::state_deck::CardAttributeLifecycle::Exile => desc = desc + ".EXILE. ",
    //             crate::state::state_deck::CardAttributeLifecycle::Linger => desc = desc + ".LINGER. ",
    //             crate::state::state_deck::CardAttributeLifecycle::Light => desc = desc + ".LIGHT. ",
    //             crate::state::state_deck::CardAttributeLifecycle::Persistant => desc = desc + ".PERSISTANT. ",
    //             crate::state::state_deck::CardAttributeLifecycle::Reliable(_) => {}
    //             crate::state::state_deck::CardAttributeLifecycle::Light => {}
    //             crate::state::state_deck::CardAttributeLifecycle::Heavy => {}
    //             crate::state::state_deck::CardAttributeLifecycle::Consume => desc = desc + ".CONSUME. ",
    //         }
    //     }
    //     let mut r = ComponentRendererText::default();
    //     r.set_bounds(Vector2::new(0.25, 0.2));
    //     r.set_font_size(0.02);
    //     r.set_contents(&desc);
    //     r.set_parent(Some(parent.clone()));
    //     let e0 = world
    //         .instantiate(
    //             "",
    //             Transform::default()
    //                 .set_position(Vector3::back() * 0.02 + Vector3::down() * 0.155)
    //                 .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
    //                 .set_parent(Some(parent.clone())),
    //         )
    //         .add_component_value(r);

    //     // create title
    //     let mut r = ComponentRendererText::default();
    //     r.set_bounds(Vector2::new(0.5, 0.2));
    //     r.set_font_size(0.03);
    //     r.set_contents(&x.get_title());
    //     r.set_parent(Some(parent.clone()));
    //     // create title
    //     let mut r = ComponentRendererText::default();
    //     r.set_bounds(Vector2::new(0.5, 0.2));
    //     r.set_font_size(0.03);
    //     r.set_contents(&x.get_title());
    //     r.set_parent(Some(parent.clone()));
    //     let e1 = world
    //         .instantiate(
    //             "",
    //             Transform::default()
    //                 .set_position(Vector3::back() * 0.02 + Vector3::up() * 0.235)
    //                 .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
    //                 .set_parent(Some(parent.clone())),
    //         )
    //         .add_component_value(r);
    //     // create type
    //     let mut r = ComponentRendererText::default();
    //     r.set_bounds(Vector2::new(0.25, 0.2));
    //     r.set_font_size(0.02);
    //     r.set_contents(&format!("{}", x.get_manuever_type()));
    //     r.set_parent(Some(parent.clone()));
    //     let e2 = world
    //         .instantiate(
    //             "",
    //             Transform::default()
    //                 .set_position(Vector3::back() * 0.02 + Vector3::down() * 0.06)
    //                 .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
    //                 .set_parent(Some(parent.clone())),
    //         )
    //         .add_component_value(r);
    //     // create cost
    //     let mut r = ComponentRendererText::default();
    //     r.set_bounds(Vector2::new(0.25, 0.2));
    //     r.set_font_size(0.03);
    //     r.set_contents(&x.get_cost(&game_state, game_state.instance_id).to_string());
    //     r.set_parent(Some(parent.clone()));
    //     let e3 = world
    //         .instantiate(
    //             "",
    //             Transform::default()
    //                 .set_position(Vector3::back() * 0.02 + Vector3::down() * 0.25 + Vector3::right() * 0.135)
    //                 .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
    //                 .set_parent(Some(parent.clone())),
    //         )
    //         .add_component_value(r);
    //     game_state.edit::<StateEntityIDs>(|x: &mut StateEntityIDs| {
    //         x.add(EntityIDTypes::UICards, parent.clone());
    //         x.add(EntityIDTypes::UICards, e0.clone());
    //         x.add(EntityIDTypes::UICards, e1.clone());
    //         x.add(EntityIDTypes::UICards, e2.clone());
    //         x.add(EntityIDTypes::UICards, e3.clone());
    //     });
    // }
}
