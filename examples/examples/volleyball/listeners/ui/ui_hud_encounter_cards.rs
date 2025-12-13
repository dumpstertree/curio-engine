use built_in::component::{
    component_renderer_animated::RendererAnimated,
    component_renderer_static::Renderer,
    component_renderer_text::{ComponentRendererText, RendererCommon},
};
use built_in_state::state_camera::CameraState;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState, quaternion::Quaternion, vector2::Vector2, vector3::Vector3},
    dumpster_engine::NetworkModes,
    gameplay::{
        ecs::{
            component::component_transform::Transform,
            traits::ecs_event_reciever::{self, InstanceLimiter},
        },
        world_context::WorldContext,
    },
    io::asset_loader::AssetLoader,
};
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    AssetMappingUIDs,
    cards::card_instance::CardInstance,
    ecs::components::{component_card::ComponentCard, component_energy_token::ComponentEnergyToken, component_player::ComponentPlayer, component_ui_ball_state::ComponentUIBallState, component_ui_score::ComponentUIScoreState, component_ui_turn::ComponentUITurnState},
    game_events::GameEvents,
    state::{
        peer::state_peer_entity_ids::{EntityIDTypes, StateEntityIDs},
        state_deck::{Deck, StateDeck},
        state_teams::{StateTeamAssignments, Teams},
    },
};

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct Listener {}

// Impl - Instance
impl InstanceLimiter for Listener {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
// Impl - Listener
impl ecs_event_reciever::EventReciever<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::EnableUICombat => {
                Self::spawn_ui_cards(game_state, world);
            }
            GameEvents::DisableUICombat => {
                Self::despawn_ui_cards(game_state, world);
            }
            _ => {}
        }
    }
}
impl Listener {
    fn despawn_ui_cards(game_state: &mut GameState, world: &mut WorldContext) {
        let id = EntityIDTypes::UICards;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            // let _ = world.despawn(e);
            e.destroy();
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    pub fn spawn_ui_cards(game_state: &mut GameState, world: &mut WorldContext) {
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
            return;
        } else {
            return;
        }
        let camera_state = game_state.get::<CameraState>();

        for card in &my_deck.all_cards {
            Self::spawn_card(world, card.clone(), camera_state.cameras.rotation, game_state);
        }
    }
    fn spawn_card(world: &mut WorldContext, x: Arc<CardInstance>, rotation: Quaternion, game_state: &mut GameState) {
        let asset = AssetLoader::load_model_static_from_database(AssetMappingUIDs::Card.uid());
        let parent = world
            .instantiate("", Transform::default().set_rotation(rotation))
            .add_component_value(Renderer::default().set_asset(Some(asset.clone())))
            .add_component_value(ComponentCard::default().set_instance(x.clone()));

        // create description
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
            }
        }
        let mut r = ComponentRendererText::default();
        r.set_bounds(Vector2::new(0.25, 0.2));
        r.set_font_size(0.02);
        r.set_contents(&desc);
        r.set_parent(Some(parent.clone()));
        let e0 = world
            .instantiate(
                "",
                Transform::default()
                    .set_position(Vector3::back() * 0.02 + Vector3::down() * 0.155)
                    .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
                    .set_parent(Some(parent.clone())),
            )
            .add_component_value(r);

        // create title
        let mut r = ComponentRendererText::default();
        r.set_bounds(Vector2::new(0.5, 0.2));
        r.set_font_size(0.03);
        r.set_contents(&x.get_title());
        r.set_parent(Some(parent.clone()));
        // create title
        let mut r = ComponentRendererText::default();
        r.set_bounds(Vector2::new(0.5, 0.2));
        r.set_font_size(0.03);
        r.set_contents(&x.get_title());
        r.set_parent(Some(parent.clone()));
        let e1 = world
            .instantiate(
                "",
                Transform::default()
                    .set_position(Vector3::back() * 0.02 + Vector3::up() * 0.235)
                    .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
                    .set_parent(Some(parent.clone())),
            )
            .add_component_value(r);
        // create type
        let mut r = ComponentRendererText::default();
        r.set_bounds(Vector2::new(0.25, 0.2));
        r.set_font_size(0.02);
        r.set_contents(&format!("{}", x.get_manuever_type()));
        r.set_parent(Some(parent.clone()));
        let e2 = world
            .instantiate(
                "",
                Transform::default()
                    .set_position(Vector3::back() * 0.02 + Vector3::down() * 0.06)
                    .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
                    .set_parent(Some(parent.clone())),
            )
            .add_component_value(r);
        // create cost
        let mut r = ComponentRendererText::default();
        r.set_bounds(Vector2::new(0.25, 0.2));
        r.set_font_size(0.03);
        r.set_contents(&x.get_cost(&game_state, game_state.instance_id).to_string());
        r.set_parent(Some(parent.clone()));
        let e3 = world
            .instantiate(
                "",
                Transform::default()
                    .set_position(Vector3::back() * 0.02 + Vector3::down() * 0.25 + Vector3::right() * 0.135)
                    .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
                    .set_parent(Some(parent.clone())),
            )
            .add_component_value(r);
        game_state.edit::<StateEntityIDs>(|x: &mut StateEntityIDs| {
            x.add(EntityIDTypes::UICards, parent.clone());
            x.add(EntityIDTypes::UICards, e0.clone());
            x.add(EntityIDTypes::UICards, e1.clone());
            x.add(EntityIDTypes::UICards, e2.clone());
            x.add(EntityIDTypes::UICards, e3.clone());
        });
    }
}
