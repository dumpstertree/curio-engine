use crate::{
    cards::{
        card_attributes::{card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers, card_attribute_requirement::CardAttributeRequirement},
        card_attributes_targets::{attribute_target_type_cards::AttributeTargetTypesCards, attribute_target_type_entities::AttribtuteTargetTypesEntities, attribute_target_type_tiles::AttributeTargetTypesTiles},
        card_master::CardMaster,
        card_statement::CardStatement,
        enums::attribute_clear_flag::ModifierClearFlag,
    },
    game_board::Directions,
    state::{
        state_ball_mode::BallModes,
        state_deck::{CardAttributeLifecycle, CardTypes},
    },
};
use core::collections::vector2_int::Vector2Int;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

static LIBRARY: Mutex<Option<HashMap<String, Arc<CardMaster>>>> = Mutex::new(None);
pub struct CardLibrary {}
impl CardLibrary {
    fn init() -> HashMap<String, Arc<CardMaster>> {
        let mut hashmap: HashMap<String, Arc<CardMaster>> = HashMap::new();

        hashmap.insert(
            String::from("move_forward_standard"),
            Arc::new(CardMaster::new(
                "move_forward_standard",
                "",
                CardTypes::Move,
                vec![CardStatement::new(
                    1, //
                    vec![CardAttributeRequirement::RequireCanMove(Directions::Forward)],
                    vec![],
                    vec![CardAttributeEvents::MoveEntity(AttribtuteTargetTypesEntities::User, AttributeTargetTypesTiles::RandomInRangeLocalToUser(Vector2Int::new(0, 1), Vector2Int::new(0, 1)))],
                )],
                vec![CardAttributeLifecycle::Quick, CardAttributeLifecycle::Linger, CardAttributeLifecycle::Persistant, CardAttributeLifecycle::Light],
            )),
        );
        hashmap.insert(
            String::from("move_back_standard"),
            Arc::new(CardMaster::new(
                "move_back_standard",
                "",
                CardTypes::Move,
                vec![CardStatement::new(
                    1, //
                    vec![CardAttributeRequirement::RequireCanMove(Directions::Back)],
                    vec![],
                    vec![CardAttributeEvents::MoveEntity(AttribtuteTargetTypesEntities::User, AttributeTargetTypesTiles::RandomInRangeLocalToUser(Vector2Int::new(0, -1), Vector2Int::new(0, -1)))],
                )],
                vec![CardAttributeLifecycle::Quick, CardAttributeLifecycle::Linger, CardAttributeLifecycle::Persistant, CardAttributeLifecycle::Light],
            )),
        );
        hashmap.insert(
            String::from("move_left_standard"),
            Arc::new(CardMaster::new(
                "move_left_standard",
                "",
                CardTypes::Move,
                vec![CardStatement::new(
                    1, //
                    vec![CardAttributeRequirement::RequireCanMove(Directions::Left)],
                    vec![],
                    vec![CardAttributeEvents::MoveEntity(AttribtuteTargetTypesEntities::User, AttributeTargetTypesTiles::RandomInRangeLocalToUser(Vector2Int::new(-1, 0), Vector2Int::new(-1, 0)))],
                )],
                vec![CardAttributeLifecycle::Quick, CardAttributeLifecycle::Linger, CardAttributeLifecycle::Persistant, CardAttributeLifecycle::Light],
            )),
        );
        hashmap.insert(
            String::from("move_right_standard"),
            Arc::new(CardMaster::new(
                "move_right_standard",
                "",
                CardTypes::Move,
                vec![CardStatement::new(
                    1, //
                    vec![CardAttributeRequirement::RequireCanMove(Directions::Right)],
                    vec![],
                    vec![CardAttributeEvents::MoveEntity(AttribtuteTargetTypesEntities::User, AttributeTargetTypesTiles::RandomInRangeLocalToUser(Vector2Int::new(1, 0), Vector2Int::new(1, 0)))],
                )],
                vec![CardAttributeLifecycle::Quick, CardAttributeLifecycle::Linger, CardAttributeLifecycle::Persistant, CardAttributeLifecycle::Light],
            )),
        );
        // rest
        hashmap.insert(
            String::from("heat"),
            Arc::new(CardMaster::new(
                "HEAT",
                "Draw 5 cards. Drain Heat",
                CardTypes::Rest,
                vec![CardStatement::new(
                    0, //
                    vec![CardAttributeRequirement::RequireHeatGreaterEqual(30)],
                    vec![],
                    vec![CardAttributeEvents::DrawCards(5, AttribtuteTargetTypesEntities::User), CardAttributeEvents::DrainHeat(AttribtuteTargetTypesEntities::User)],
                )],
                vec![CardAttributeLifecycle::Quick, CardAttributeLifecycle::Linger, CardAttributeLifecycle::Persistant, CardAttributeLifecycle::Reliable(2), CardAttributeLifecycle::Light],
            )),
        );
        hashmap.insert(
            String::from("rest"),
            Arc::new(CardMaster::new(
                "rest",
                "do a rest",
                CardTypes::Rest,
                vec![CardStatement::new(
                    1, //
                    vec![],
                    vec![CardAttributeModifiers::EditEnergyForEntities(ModifierClearFlag::Game, AttribtuteTargetTypesEntities::User, -1)],
                    vec![CardAttributeEvents::DiscardCards(AttributeTargetTypesCards::AllUser), CardAttributeEvents::DrawCards(7, AttribtuteTargetTypesEntities::User)],
                )],
                vec![CardAttributeLifecycle::Quick, CardAttributeLifecycle::Linger, CardAttributeLifecycle::Persistant, CardAttributeLifecycle::Reliable(1), CardAttributeLifecycle::Light],
            )),
        );
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
                vec![CardAttributeLifecycle::Quick, CardAttributeLifecycle::Exhuast, CardAttributeLifecycle::Reliable(0), CardAttributeLifecycle::Light],
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
                            AttributeTargetTypesTiles::RandomInRangeLocalToBall(
                                //
                                Vector2Int::new(-1, 1),
                                Vector2Int::new(1, 1),
                            ),
                        ),
                    ],
                )],
                vec![],
            )),
        );
        hashmap.insert(
            String::from("popsicle"),
            Arc::new(CardMaster::new(
                "Popsicle",
                "Eat it up yum. +1 Energy, Draw a card",
                CardTypes::Food,
                vec![CardStatement::new(
                    1, //
                    vec![],
                    vec![],
                    vec![
                        //
                        CardAttributeEvents::GainEnergy(1, AttribtuteTargetTypesEntities::User),
                        CardAttributeEvents::DrawCards(1, AttribtuteTargetTypesEntities::User),
                    ],
                )],
                vec![
                    //
                    CardAttributeLifecycle::Quick,
                    CardAttributeLifecycle::Linger,
                    CardAttributeLifecycle::Light,
                    CardAttributeLifecycle::Consume,
                    CardAttributeLifecycle::Light,
                ],
            )),
        );
        hashmap.insert(
            String::from("set"),
            Arc::new(CardMaster::new(
                "Set",
                "Sets the ball",
                CardTypes::Set,
                vec![CardStatement::new(
                    1, //
                    vec![
                        CardAttributeRequirement::RequireNotBallMode(BallModes::Serve), //
                        CardAttributeRequirement::BallRangeLessEqual(0),
                    ],
                    vec![],
                    vec![
                        CardAttributeEvents::SetBallMode(BallModes::Set),
                        CardAttributeEvents::MoveBall(
                            //
                            AttributeTargetTypesTiles::RandomInRangeLocalToBall(
                                //
                                Vector2Int::new(0, 0), //
                                Vector2Int::new(0, 0),
                            ),
                        ),
                    ],
                )],
                vec![],
            )),
        );
        hashmap.insert(
            String::from("wild_card"),
            Arc::new(CardMaster::new(
                "Wild Card",
                "Bump the ball to a random position",
                CardTypes::Bump,
                vec![CardStatement::new(
                    0, //
                    vec![
                        CardAttributeRequirement::RequireNotBallMode(BallModes::Serve), //
                        CardAttributeRequirement::BallRangeLessEqual(0),
                    ],
                    vec![],
                    vec![
                        CardAttributeEvents::SetBallMode(BallModes::Bump),
                        CardAttributeEvents::MoveBall(
                            //
                            AttributeTargetTypesTiles::RandomInRangeGlobal(
                                //
                                Vector2Int::new(0, 0), //
                                Vector2Int::new(3, 3),
                            ),
                        ),
                    ],
                )],
                vec![],
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
                                AttributeTargetTypesTiles::RandomInRangeLocalToBall(
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
                                AttributeTargetTypesTiles::RandomInRangeLocalToBall(
                                    //
                                    Vector2Int::new(0, 2), //
                                    Vector2Int::new(0, 2),
                                ),
                            ),
                        ],
                    ),
                ],
                vec![],
            )),
        );
        hashmap.insert(
            String::from("counter_spike"),
            Arc::new(CardMaster::new(
                "Counter Spike!",
                "Forward +2. Cost 0 if ball is Spiked",
                CardTypes::Spike,
                vec![
                    CardStatement::new(
                        0, //
                        vec![
                            CardAttributeRequirement::RequireNotBallMode(BallModes::Serve), //
                            CardAttributeRequirement::RequireBallMode(BallModes::Spike),
                            CardAttributeRequirement::BallRangeLessEqual(0),
                        ],
                        vec![],
                        vec![
                            CardAttributeEvents::SetBallMode(BallModes::Spike),
                            CardAttributeEvents::MoveBall(
                                //
                                AttributeTargetTypesTiles::RandomInRangeLocalToBall(
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
                                AttributeTargetTypesTiles::RandomInRangeLocalToBall(
                                    //
                                    Vector2Int::new(0, 2), //
                                    Vector2Int::new(0, 2),
                                ),
                            ),
                        ],
                    ),
                ],
                vec![],
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
                vec![],
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
                vec![],
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
                vec![],
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
                    vec![CardAttributeModifiers::EditRangeForEntities(ModifierClearFlag::Turn, AttribtuteTargetTypesEntities::User, -1)],
                    vec![],
                )],
                vec![],
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
                    vec![(CardAttributeModifiers::EditRangeForEntities(ModifierClearFlag::Turn, AttribtuteTargetTypesEntities::User, 1))],
                    vec![],
                )],
                vec![],
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
