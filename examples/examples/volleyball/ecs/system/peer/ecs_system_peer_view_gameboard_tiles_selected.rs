use crate::ecs::components::component_gameboard_selection::ComponentGameBoardSelection;
use crate::exploration::exploration_path::RoomTypes;
use crate::game_board::GameBoard;
use crate::state::host::state_exploration::StateExploration;
use crate::state::peer::state_peer_select_targets::StatePeerSelectTargets;

use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_system::global_ecs_system;
use system_component_default_gameplay::component::component_renderer_static::Renderer;
use system_component_default_gameplay::component::component_renderer_text::RendererCommon;
use system_component_default_gameplay::component::component_transform::Transform;
use system_component_default_gameplay::ecs_system::ECSSystemEventless;
use system_component_default_gameplay::world_context::{WorldContext, WorldContextCommon};

#[global_ecs_system]
pub struct Instance {}
impl ECSSystemEventless for Instance {
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_peer()
    }
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut WorldContext) -> bool {
        let state_exploration = game_state.get::<StateExploration>();
        state_exploration.exploration.get_cur_room().room_type == RoomTypes::Combat && !state_exploration.is_selecting_next
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut WorldContext, events: &mut EventQueue) {
        let state_selection = game_state.get::<StatePeerSelectTargets>();

        world.query_mut::<(&mut Transform, &ComponentGameBoardSelection, &mut Renderer)>(|query| {
            for (_, (transform, _, renderer)) in query {
                let pos = GameBoard::get_world_position(state_selection.selected_index.x, state_selection.selected_index.y);
                transform.position = pos;
                renderer.set_enabled(state_selection.enabled.is_some());
            }
        });
    }
}
