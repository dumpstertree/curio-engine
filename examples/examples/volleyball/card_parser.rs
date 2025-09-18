use serde::{Deserialize, Serialize};

use crate::{
    cards::{card_master::CardMaster, card_modifier::CardModifier},
    game_events::GameEvents,
    state::{state_deck::AttributeTargets, state_teams::StateTeamAssignments, state_turn::StateTurn},
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

static ACTIVE_ATTS: Mutex<Vec<(AttributeTargets, AttributeClearFlag, CardModifier)>> = Mutex::new(vec![]);

// pub struct CardParser {}
// impl CardParser {
//     pub fn active_attributes_apply(card: Arc<CardMaster>, game_state: &GameState) {
//         // println!("add state {}", game_state.instance_id);
//         // let Ok(mut guard) = ACTIVE_ATTS.lock() else {
//         //     return;
//         // };

//         // for x in &card.attributes {
//         //     let t = x.0(game_state);
//         //     let d = x.2.get_card_data(game_state, &t);
//         //     guard.push((t, x.1.clone(), d));
//         // }
//     }
//     pub fn active_attributes_clear(flag: AttributeClearFlag) {
//         // let Ok(mut guard) = ACTIVE_ATTS.lock() else {
//         //     return;
//         // };

//         // for i in (0..guard.len()).rev() {
//         //     if guard[i].1 == flag {
//         //         guard.remove(i);
//         //     }
//         // }
//     }
//     // pub fn test_card(user_id: &i32, card: Arc<Card>) -> CardData {
//     //     //lock the mutex - this needs to happen after the application to not double lock
//     //     let Ok(guard) = ACTIVE_ATTS.lock() else {
//     //         panic!();
//     //     };

//     //     // take all the data and get how its applicable for this user/card
//     //     let mut data = vec![];
//     //     // for att in card.attributes.iter() {
//     //     //     let includes_entity = att.0
//     //     //     data.push(att.1.get_card_data());
//     //     // }
//     //     // for att in guard.iter() {
//     //     //     data.push(att.1.get_card_data());
//     //     // }

//     //     // flatten all the data into one
//     //     CardData::flatten(&data)
//     // }
//     pub fn play_card(game_state: &GameState, user_id: &i32, card: Arc<CardMaster>) -> CardModifier {
//         // add attributes
//         CardParser::active_attributes_apply(card, game_state);
//         let mut data = vec![];

//         {
//             //lock the mutex - this needs to happen after the application to not double lock
//             let Ok(guard) = ACTIVE_ATTS.lock() else {
//                 panic!();
//             };

//             // take all the data and get how its applicable for this user/card
//             for att in guard.iter() {
//                 // get only cards that apply to this entity

//                 let contains_entity = att.0.entities.contains(user_id);
//                 if !contains_entity {
//                     continue;
//                 }

//                 //push to cur data
//                 data.push(att.2.clone());
//             }
//         }

//         // remove any of the attributes that were only applied for this play
//         CardParser::active_attributes_clear(AttributeClearFlag::Play);

//         // flatten all the data into one
//         CardModifier::flatten(&data)
//     }
// }

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeClearFlag {
    Play,
    Turn,
    Game,
}
