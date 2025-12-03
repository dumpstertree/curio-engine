use crate::{
    cards::{card_dependencies::data_dep_filled::DataDepsFilled, enums::attribute_clear_flag::ModifierClearFlag},
    state::state_ball_mode::BallModes,
};

#[derive(Clone)]
pub enum CardEvents {
    // modifier
    ModifierEnergyForEntities(DataDepsFilled, ModifierClearFlag, i32),
    ModifierCostForEntities(DataDepsFilled, ModifierClearFlag, i32),
    ModifierRangeForEntities(DataDepsFilled, ModifierClearFlag, i32),
    // events
    EventHeatDrain(DataDepsFilled),
    EventEnergyFill(DataDepsFilled),
    EventEnergyEdit(DataDepsFilled, i32),
    EventMoveEntities(DataDepsFilled, DataDepsFilled),
    EventCardDraw(DataDepsFilled, i32),
    EventCardDiscard(DataDepsFilled),
    EventChangeBallMode(BallModes),
    EventMoveBall(DataDepsFilled),
    // clear
    ClearModifiersForFlag(ModifierClearFlag),
    ClearModifiersAll(),
}
