use crate::ecs::components::component_gameboard_selection::ComponentGameBoardSelection;
use crate::exploration::exploration_path::RoomTypes;
use crate::game_board::GameBoard;
use crate::state::host::state_exploration::StateExploration;
use crate::state::peer::state_peer_select_targets::StatePeerSelectTargets;

use curio_core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use gameplay::{
    built_in::facet::{renderer::renderer_static::RendererStatic, renderer_common::RendererCommon, transform::transform3d::Transform3D},
    context_3d::Context3D,
    traits::{habit::Habit, scope::Scope},
    traits_internal::world_context_common::ContextCommon,
};
use habit::habit;

#[habit]
pub struct Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool {
        let state_exploration = game_state.get::<StateExploration>();
        state_exploration.exploration.get_cur_room().room_type == RoomTypes::Combat && !state_exploration.is_selecting_next
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn tick(&mut self, game_state: &mut GameState, world: &mut Context3D, events: &mut EventQueue) {
        let state_selection = game_state.get::<StatePeerSelectTargets>();

        world.edit::<(&mut Transform3D, &ComponentGameBoardSelection, &mut RendererStatic)>(|query| {
            for (_, (transform, _, renderer)) in query {
                let pos = GameBoard::get_world_position(state_selection.selected_index.x, state_selection.selected_index.y);
                transform.position = pos;
                renderer.set_enabled(state_selection.enabled.is_some());
            }
        });
    }
}
