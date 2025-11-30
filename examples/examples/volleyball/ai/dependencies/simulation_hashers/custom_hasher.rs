use crate::{
    ai::dependencies::simulation_hasher::SimulationHasher,
    state::{
        host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack, other::state_terminated::StateTerminated, state_deck::StateDeck, state_energy::StateEnergy, state_position_ball::StatePositionBall, state_position_player::StatePositionPlayer, state_teams::StateTeamAssignments,
        state_turn::StateTurn,
    },
};
use core::collections::game_state::GameState;
use std::hash::{DefaultHasher, Hash, Hasher};

pub struct CustomHasher {}
impl SimulationHasher for CustomHasher {
    fn hash(&self, game_state: &GameState) -> u64 {
        let mut hasher = DefaultHasher::new();

        game_state.get::<StateTerminated>().hash(&mut hasher);
        game_state.get::<StateTurn>().hash(&mut hasher);
        game_state.get::<StateTeamAssignments>().hash(&mut hasher);
        game_state.get::<StatePositionPlayer>().hash(&mut hasher);
        game_state.get::<StatePositionBall>().hash(&mut hasher);
        game_state.get::<StateEnergy>().hash(&mut hasher);
        game_state.get::<StateDeck>().hash(&mut hasher);
        game_state
            .get::<StateCardAttributeModifierStack>()
            .hash(&mut hasher);

        hasher.finish()
    }
}
