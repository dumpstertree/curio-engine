use crate::{
    card_parser::AttributeClearFlag,
    cards::{
        attribute_target_type_cards::AttributeTargetTypesCards,
        attribute_target_type_entities::AttribtuteTargetTypesEntities,
        // attribute_target_type_players::AtrributeTargetTypesPlayers,
        attribute_target_type_tiles::AttributeTargetTypesTiles,
        card_attribute_events::CardAttributeEvents,
        card_attribute_modifier::CardAttributeModifiers,
        card_attribute_requirement::CardAttributeRequirement,
        card_master::{CardMaster, CardStatement},
    },
    state::{state_ball_mode::BallModes, state_deck::CardTypes},
};
use core::collections::{vector2::Vector2, vector2_int::Vector2Int};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

static LIBRARY: Mutex<Option<HashMap<String, Arc<CardMaster>>>> = Mutex::new(None);
pub struct CardLibrary {}
impl CardLibrary {
    fn init() -> HashMap<String, Arc<CardMaster>> {
        let mut hashmap: HashMap<String, Arc<CardMaster>> = HashMap::new();
        // rest
        // hashmap.insert(
        //     String::from("rest"),
        //     Arc::new(CardMaster::new(
        //         "rest",
        //         "do a rest",
        //         CardTypes::Rest,
        //         vec![CardStatement::new(
        //             0, //
        //             vec![],
        //             vec![CardAttributeModifiers::EditEnergyForEntities(AttributeClearFlag::Game, AttribtuteTargetTypesEntities::User, -1)],
        //             vec![CardAttributeEvents::DiscardCards(AttributeTargetTypesCards::AllUser), CardAttributeEvents::DrawCards(7, AttribtuteTargetTypesEntities::User)],
        //         )],
        //     )),
        // );
        hashmap.insert(
            String::from("serve"),
            Arc::new(CardMaster::new(
                "Serve",
                "Serve the ball to a random position on the opponents side.",
                CardTypes::Serve,
                vec![CardStatement::new(
                    0, //
                    vec![CardAttributeRequirement::RequireBallMode(BallModes::Serve), CardAttributeRequirement::BallRangeLessEqual(0)],
                    vec![],
                    vec![CardAttributeEvents::SetBallMode(BallModes::Bump), CardAttributeEvents::MoveBall(AttributeTargetTypesTiles::RandomOnTeamOpponent)],
                )],
            )),
        );
        hashmap.insert(
            String::from("bump"),
            Arc::new(CardMaster::new(
                "bump",
                "Forward +1",
                CardTypes::Bump,
                vec![CardStatement::new(
                    1, //
                    vec![CardAttributeRequirement::RequireNotBallMode(BallModes::Serve), CardAttributeRequirement::BallRangeLessEqual(0)],
                    vec![],
                    vec![
                        CardAttributeEvents::SetBallMode(
                            //
                            BallModes::Bump,
                        ),
                        CardAttributeEvents::MoveBall(
                            //
                            AttributeTargetTypesTiles::RandomInRangeLocal(
                                //
                                Vector2Int::new(-1, 1),
                                Vector2Int::new(1, 1),
                            ),
                        ),
                    ],
                )],
            )),
        );
        hashmap.insert(
            String::from("set"),
            Arc::new(CardMaster::new(
                "Set",
                "Sets the ball",
                CardTypes::Bump,
                vec![CardStatement::new(
                    1, //
                    vec![
                        CardAttributeRequirement::RequireNotBallMode(BallModes::Serve), //
                        CardAttributeRequirement::BallRangeLessEqual(0),
                    ],
                    vec![],
                    vec![CardAttributeEvents::SetBallMode(BallModes::Set)],
                )],
            )),
        );
        hashmap.insert(
            String::from("spike"),
            Arc::new(CardMaster::new(
                "Spike!",
                "Forward +2. Cost 0 if ball is Set",
                CardTypes::Spike,
                vec![
                    CardStatement::new(
                        0, //
                        vec![
                            CardAttributeRequirement::RequireNotBallMode(BallModes::Serve), //
                            CardAttributeRequirement::RequireBallMode(BallModes::Set),
                            CardAttributeRequirement::BallRangeLessEqual(0),
                        ],
                        vec![],
                        vec![
                            CardAttributeEvents::SetBallMode(BallModes::Spike),
                            CardAttributeEvents::MoveBall(
                                //
                                AttributeTargetTypesTiles::RandomInRangeLocal(
                                    //
                                    Vector2Int::new(0, 2), //
                                    Vector2Int::new(0, 2),
                                ),
                            ),
                        ],
                    ),
                    CardStatement::new(
                        3, //
                        vec![
                            CardAttributeRequirement::RequireNotBallMode(BallModes::Serve), //
                            CardAttributeRequirement::BallRangeLessEqual(0),
                        ],
                        vec![],
                        vec![
                            CardAttributeEvents::SetBallMode(BallModes::Spike),
                            CardAttributeEvents::MoveBall(
                                //
                                AttributeTargetTypesTiles::RandomInRangeLocal(
                                    //
                                    Vector2Int::new(0, 2), //
                                    Vector2Int::new(0, 2),
                                ),
                            ),
                        ],
                    ),
                ],
            )),
        );

        // spells
        hashmap.insert(
            String::from("curse"),
            Arc::new(CardMaster::new(
                "curse",
                "Target DISCARDs a card",
                CardTypes::Spell,
                vec![CardStatement::new(
                    1, //
                    vec![],
                    vec![],
                    vec![CardAttributeEvents::DiscardCards(AttributeTargetTypesCards::RandomOpponent)],
                )],
            )),
        );
        hashmap.insert(
            String::from("blessing"),
            Arc::new(CardMaster::new(
                "blessing",
                "Target DRAWs 2 cards",
                CardTypes::Spell,
                vec![CardStatement::new(
                    1, //
                    vec![],
                    vec![],
                    vec![CardAttributeEvents::DrawCards(2, AttribtuteTargetTypesEntities::User)],
                )],
            )),
        );
        hashmap.insert(
            String::from("deep_breath"),
            Arc::new(CardMaster::new(
                "deep_breath",
                "Energy +2 on until end of turn",
                CardTypes::Spell,
                vec![CardStatement::new(
                    1, //
                    vec![],
                    vec![],
                    vec![CardAttributeEvents::GainEnergy(2, AttribtuteTargetTypesEntities::User)],
                )],
            )),
        );

        hashmap.insert(
            String::from("hold_back"),
            Arc::new(CardMaster::new(
                "hold_back",
                "RANGE -1 on until end of turn",
                CardTypes::Spell,
                vec![CardStatement::new(
                    1, //
                    vec![],
                    vec![CardAttributeModifiers::EditRangeForEntities(AttributeClearFlag::Turn, AttribtuteTargetTypesEntities::User, -1)],
                    vec![],
                )],
            )),
        );
        hashmap.insert(
            String::from("extra_oomph"),
            Arc::new(CardMaster::new(
                "Extra Oomph",
                "RANGE +1 on until end of turn",
                CardTypes::Spell,
                vec![CardStatement::new(
                    1, //
                    vec![],
                    vec![(CardAttributeModifiers::EditRangeForEntities(AttributeClearFlag::Turn, AttribtuteTargetTypesEntities::User, 1))],
                    vec![],
                )],
            )),
        );
        hashmap
    }
    pub fn get_master_card(card_id: &str) -> Arc<CardMaster> {
        let mut guard = LIBRARY.lock().unwrap();

        // Initialize if not done yet
        if guard.is_none() {
            *guard = Some(CardLibrary::init());
        }

        // clone the arc and return the card
        guard.as_ref().unwrap().get(card_id).cloned().unwrap()
    }
}
