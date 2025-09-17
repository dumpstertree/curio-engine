use serde::{Deserialize, Serialize};

use crate::{
    game_events::GameEvents,
    state::{
        state_deck::{AttributeTargets, Card},
        state_teams::StateTeamAssignments,
        state_turn::StateTurn,
    },
};
use core::{
    collections::{
        event_queue::EventQueue,
        game_state::{self, GameState},
        vector2_int::Vector2Int,
    },
    random::Random,
};
use std::sync::{Arc, Mutex};

static ACTIVE_ATTS: Mutex<Vec<(AttributeTargets, AttributeClearFlag, CardData)>> = Mutex::new(vec![]);

pub struct CardParser {}
impl CardParser {
    pub fn active_attributes_apply(card: Arc<Card>, game_state: &GameState) {
        // println!("add state {}", game_state.instance_id);
        // let Ok(mut guard) = ACTIVE_ATTS.lock() else {
        //     return;
        // };

        // for x in &card.attributes {
        //     let t = x.0(game_state);
        //     let d = x.2.get_card_data(game_state, &t);
        //     guard.push((t, x.1.clone(), d));
        // }
    }
    pub fn active_attributes_clear(flag: AttributeClearFlag) {
        // let Ok(mut guard) = ACTIVE_ATTS.lock() else {
        //     return;
        // };

        // for i in (0..guard.len()).rev() {
        //     if guard[i].1 == flag {
        //         guard.remove(i);
        //     }
        // }
    }
    // pub fn test_card(user_id: &i32, card: Arc<Card>) -> CardData {
    //     //lock the mutex - this needs to happen after the application to not double lock
    //     let Ok(guard) = ACTIVE_ATTS.lock() else {
    //         panic!();
    //     };

    //     // take all the data and get how its applicable for this user/card
    //     let mut data = vec![];
    //     // for att in card.attributes.iter() {
    //     //     let includes_entity = att.0
    //     //     data.push(att.1.get_card_data());
    //     // }
    //     // for att in guard.iter() {
    //     //     data.push(att.1.get_card_data());
    //     // }

    //     // flatten all the data into one
    //     CardData::flatten(&data)
    // }
    pub fn play_card(game_state: &GameState, user_id: &i32, card: Arc<Card>) -> CardData {
        // add attributes
        CardParser::active_attributes_apply(card, game_state);
        let mut data = vec![];

        {
            //lock the mutex - this needs to happen after the application to not double lock
            let Ok(guard) = ACTIVE_ATTS.lock() else {
                panic!();
            };

            // take all the data and get how its applicable for this user/card
            for att in guard.iter() {
                // get only cards that apply to this entity

                let contains_entity = att.0.entities.contains(user_id);
                if !contains_entity {
                    continue;
                }

                //push to cur data
                data.push(att.2.clone());
            }
        }

        // remove any of the attributes that were only applied for this play
        CardParser::active_attributes_clear(AttributeClearFlag::Play);

        // flatten all the data into one
        CardData::flatten(&data)
    }
}

#[derive(Clone)]
pub struct CardData {
    pub range: i32,
    pub cost: i32,
    pub energy: i32,
}
impl CardData {
    pub fn new() {}

    pub fn flatten(stack: &Vec<CardData>) -> CardData {
        let mut r = 0;
        let mut c = 0;
        let mut e = 0;
        for x in stack {
            r += x.range;
            c += x.cost;
            e += x.energy;
        }

        CardData { range: r, cost: c, energy: e }
    }
}

#[derive(Clone)]
pub enum CardAttributes {
    EditEnergy(i32, TargetTypesPlayers),
    EditPlaysRange(i32, TargetTypesPlayers),
    EditPlaysCost(i32, TargetTypesCards),
}

impl CardAttributes {
    pub fn get_card_data(&self, game_state: &GameState, card_targeting: &AttributeTargets) -> CardData {
        match self {
            CardAttributes::EditEnergy(delta, _) => CardData { range: 0, cost: 0, energy: *delta },
            CardAttributes::EditPlaysRange(delta, _) => CardData { range: *delta, cost: 0, energy: 0 },
            CardAttributes::EditPlaysCost(delta, _) => CardData { range: 0, cost: *delta, energy: 0 },
        }
    }
}
impl CardAttributes {
    pub fn get_data_dependencies(&self) -> Vec<DataDepsEmpty> {
        match self {
            CardAttributes::EditEnergy(_, target_type) => vec![DataDepsEmpty::Players(target_type.clone())],
            CardAttributes::EditPlaysRange(_, target_type) => vec![DataDepsEmpty::Players(target_type.clone())],
            CardAttributes::EditPlaysCost(_, target_type) => vec![DataDepsEmpty::Cards(target_type.clone())],
        }
    }
}

#[derive(Clone)]
pub enum TargetTypesPlayers {
    User,
    Select,
    Random,
    Opponent,
}
#[derive(Clone)]
pub enum TargetTypesEntities {
    User,
    Select,
    RandomAny,
    RandomOpponent,
}
#[derive(Clone)]
pub enum TargetTypesCards {
    SelectUser,
    SelectOpponent,
    RandomUser,
    RandomOpponent,
}
#[derive(Clone)]
pub enum TargetTypesTiles {
    Select,
    RandomAny,
    RandomOnTeamUser,
    RandomOnTeamOpponent,
}
#[derive(Clone)]
pub enum DataDepsEmpty {
    Players(TargetTypesPlayers),
    Entities(TargetTypesEntities),
    Cards(TargetTypesCards),
    Tiles(TargetTypesTiles),
}
#[derive(Clone, Serialize, Deserialize)]
pub enum DataDepsFilled {
    Players(Vec<i32>),
    Entities(Vec<i32>),
    Cards(Vec<(i32, i32)>),
    Tiles(Vec<Vector2Int>),
}
#[derive(Clone)]
pub enum CardEvents {
    DrawCards(i32, TargetTypesPlayers),
    DiscardCards(TargetTypesCards),
    MoveEntity(TargetTypesEntities, TargetTypesTiles),
    MoveBallForward(i32),
    GainEnergy(i32, TargetTypesEntities),
}
impl CardEvents {
    pub fn get_data_dependencies(&self) -> Vec<DataDepsEmpty> {
        match self {
            CardEvents::DrawCards(_, target_type) => vec![DataDepsEmpty::Players(target_type.clone())],
            CardEvents::DiscardCards(card_target_type) => vec![DataDepsEmpty::Cards(card_target_type.clone())],
            CardEvents::MoveEntity(entity_target_type, tile_target_type) => vec![DataDepsEmpty::Entities(entity_target_type.clone()), DataDepsEmpty::Tiles(tile_target_type.clone())],
            CardEvents::MoveBallForward(_) => vec![],
            CardEvents::GainEnergy(_, t) => vec![DataDepsEmpty::Entities(t.clone())],
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum AttributeClearFlag {
    Play,
    Turn,
    Game,
}
