use serde::{Deserialize, Serialize};

use crate::cards::enums::attribute_clear_flag::ModifierClearFlag;

#[derive(Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct CardModifier {
    pub clear_flag: ModifierClearFlag,
    pub applies_to_players: Vec<i32>,
    pub applies_to_entities: Vec<i32>,
    pub applies_to_cards: Vec<i32>,
    pub range: i32,
    pub cost: i32,
    pub energy: i32,
}
impl CardModifier {
    pub fn flatten(stack: &Vec<&CardModifier>) -> CardModifier {
        let mut r = 0;
        let mut c = 0;
        let mut e = 0;
        for x in stack {
            r += x.range;
            c += x.cost;
            e += x.energy;
        }

        CardModifier {
            range: r,
            cost: c,
            energy: e,
            applies_to_players: vec![],
            applies_to_entities: vec![],
            applies_to_cards: vec![],
            clear_flag: ModifierClearFlag::Turn,
        }
    }
}
