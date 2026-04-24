use crate::{
    ai::dependencies::simulation_hasher::SimulationHasher,
    state::{
        host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack, other::state_terminated::StateTerminated, state_deck::StateDeck, state_energy::StateEnergy, state_position_ball::StatePositionBall, state_position_player::StatePositionEntities, state_teams::StateTeamAssignments,
        state_turn::StateTurn,
    },
};
use curio_core::collections::ledger::Ledger;
use std::hash::{DefaultHasher, Hash, Hasher};

pub struct CustomHasher {}
impl SimulationHasher for CustomHasher {
    fn hash(&self, ledger: &Ledger) -> u64 {
        let mut hasher = DefaultHasher::new();

        ledger.get::<StateTerminated>().hash(&mut hasher);
        ledger.get::<StateTurn>().hash(&mut hasher);
        ledger.get::<StateTeamAssignments>().hash(&mut hasher);
        ledger.get::<StatePositionEntities>().hash(&mut hasher);
        ledger.get::<StatePositionBall>().hash(&mut hasher);
        ledger.get::<StateEnergy>().hash(&mut hasher);
        ledger.get::<StateDeck>().hash(&mut hasher);
        ledger
            .get::<StateCardAttributeModifierStack>()
            .hash(&mut hasher);

        hasher.finish()
    }
}
