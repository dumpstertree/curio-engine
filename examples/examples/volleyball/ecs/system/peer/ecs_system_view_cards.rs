use crate::AssetMappingUIDs;
use crate::cards::card_instance::CardInstance;
use crate::ecs::components::component_card::ComponentCard;
use crate::state::peer::state_peer_input_mode::{InputModes, StatePeerInputMode};
use crate::state::peer::state_peer_selected_card::StatePeerSelectedCards;
use crate::state::state_deck::{self, StateDeck};
use built_in::component::component_renderer_static::Renderer;
use built_in::component::component_renderer_text::{ComponentRendererText, RendererCommon};
// use built_in::component::component_renderer::Renderer;
use built_in::component::component_transform::Transform;
use built_in_state::state_camera::CameraState;
use built_in_state::state_time::TimeState;
use core::collections::color::Color;
use core::collections::quaternion::Quaternion;
use core::collections::vector2::Vector2;
use core::collections::vector3::Vector3;
use core::io::asset_loader::AssetLoader;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_system::global_ecs_system;
use hecs::World;
use std::sync::Arc;

#[global_ecs_system]
pub struct ECSSystemViewCards {
    // asset: Arc<ModelAsset>,
    // asset_card: HashMap<String, Option<Arc<ModelAsset>>>,
    cnt: i32,
}
impl ECSSystemEventless for ECSSystemViewCards {
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn init(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        // self.asset_card = AssetLoader::load_model_static_from_database(AssetMappingUIDs::Card.uid());
    }

    fn tick(&mut self, game_state: &mut GameState, world: &mut World, events: &mut EventQueue) {
        self.cnt += 1;
        if self.cnt < 15 {
            return;
        }
        if self.cnt == 15 {
            let state_deck = game_state.get_value2::<StateDeck>();
            let my_deck = state_deck.deck.get(&game_state.instance_id).unwrap();
            let camera_state = game_state.get_value2::<CameraState>();

            for card in &my_deck.all_cards {
                self.spawn_card(world, card.clone(), camera_state.cameras.rotation);
            }
        }
        let y_selected = 0.25;
        let y_unselected = 0.5;
        let spacing = 0.5;
        let z_selected = 1.0;
        let z_unselected = 1.5;

        let state_input_mode = game_state.get_value2::<StatePeerInputMode>();
        let is_manuever_mode = state_input_mode.mode == InputModes::Manuever;
        for (_, (card, transform, renderer)) in world
            .query::<(&ComponentCard, &mut Transform, &mut Renderer)>()
            .iter()
        {
            let state_deck = game_state.get_value2::<StateDeck>();
            let my_deck = state_deck.deck.get(&game_state.instance_id).unwrap();
            let camera_state = game_state.get_value2::<CameraState>();
            let state_selected = game_state.get_value2::<StatePeerSelectedCards>();

            let Some(card_instance) = &card.card_instance else {
                continue;
            };

            let state_time = game_state.get_value2::<TimeState>();
            match my_deck.get_location(card_instance.clone()) {
                state_deck::CardLocation::Deck(index) => {
                    let pos = camera_state.cameras.position + (camera_state.cameras.rotation * Vector3::new(0.5, 0.5, 1.0));
                    let rot = camera_state.cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, -0.0));
                    transform.position = Vector3::lerp(transform.position, pos, 0.2);
                    transform.rotation = transform.rotation.slerp(rot, 0.2);
                    transform.scale = Vector3::lerp(transform.scale, Vector3::one() * 0.25, 0.2);
                    renderer.set_enabled(index == 0);
                }
                state_deck::CardLocation::Discard(index) => {
                    let pos = camera_state.cameras.position + (camera_state.cameras.rotation * Vector3::new(-0.5, 0.5, 1.0));
                    let rot = camera_state.cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, -0.0));

                    transform.position = Vector3::lerp(transform.position, pos, 0.2);
                    transform.rotation = transform.rotation.slerp(rot, 0.2);
                    transform.scale = Vector3::lerp(transform.scale, Vector3::one() * 0.25, 0.2);
                    renderer.set_enabled(index == 0);
                }
                state_deck::CardLocation::Hand(index) => {
                    let mut z = z_unselected;
                    let mut y = y_unselected;

                    if is_manuever_mode && state_selected.index == index {
                        z = z_selected;
                        y = y_selected;
                    }

                    if !is_manuever_mode {
                        y = y + 0.5;
                    }

                    let pos = camera_state.cameras.position + (camera_state.cameras.rotation * Vector3::forward()) * z + Vector3::right() * ((index - state_selected.index) as f32 * spacing) + camera_state.cameras.rotation * Vector3::down() * y;
                    let rot = camera_state.cameras.rotation;
                    transform.position = Vector3::lerp(transform.position, pos, 0.2);
                    transform.rotation = transform.rotation.slerp(rot, 0.2);
                    transform.scale = Vector3::lerp(transform.scale, Vector3::one(), 0.2);
                    renderer.set_enabled(true);

                    let is_met = card_instance
                        .get_master()
                        .requirements
                        .iter()
                        .all(|x| x.is_met(&game_state, game_state.instance_id));
                    if is_met {
                        renderer.set_tint(Color::white());
                    } else {
                        renderer.set_tint(Color::white() * 0.25);
                    }
                }
            }
        }
    }
}
impl ECSSystemViewCards {
    fn spawn_card(&self, world: &mut World, x: Arc<CardInstance>, rotation: Quaternion) {
        let asset = AssetLoader::load_model_static_from_database(AssetMappingUIDs::Card.uid());
        let parent = world.spawn((Transform::default().set_rotation(rotation), Renderer::default().set_asset(Some(asset.clone())), ComponentCard::default().set_instance(x.clone())));
        // create description
        let mut r = ComponentRendererText::default();
        r.set_bounds(Vector2::new(0.25, 0.2));
        r.set_font_size(0.02);
        r.set_contents(&x.get_master().description);
        r.set_parent(Some(parent));
        world.spawn((
            r,
            Transform::default()
                .set_position(Vector3::back() * 0.02 + Vector3::down() * 0.155)
                .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
                .set_parent(Some(parent)),
        ));
        // create title
        let mut r = ComponentRendererText::default();
        r.set_bounds(Vector2::new(0.5, 0.2));
        r.set_font_size(0.03);
        r.set_contents(&x.get_title());
        r.set_parent(Some(parent));
        world.spawn((
            r,
            Transform::default()
                .set_position(Vector3::back() * 0.02 + Vector3::up() * 0.235)
                .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
                .set_parent(Some(parent)),
        ));
        // create type
        let mut r = ComponentRendererText::default();
        r.set_bounds(Vector2::new(0.25, 0.2));
        r.set_font_size(0.02);
        r.set_contents(&format!("{}", x.get_manuever_type()));
        r.set_parent(Some(parent));
        world.spawn((
            r,
            Transform::default()
                .set_position(Vector3::back() * 0.02 + Vector3::down() * 0.06)
                .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
                .set_parent(Some(parent)),
        ));
        // create cost
        let mut r = ComponentRendererText::default();
        r.set_bounds(Vector2::new(0.25, 0.2));
        r.set_font_size(0.03);
        r.set_contents(&x.get_cost().to_string());
        r.set_parent(Some(parent));
        world.spawn((
            r,
            Transform::default()
                .set_position(Vector3::back() * 0.02 + Vector3::down() * 0.25 + Vector3::right() * 0.135)
                .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
                .set_parent(Some(parent)),
        ));
    }
}
