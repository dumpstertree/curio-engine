use crate::{
    card_parser::AttributeClearFlag,
    cards::{
        attribute_target_type_cards::AttributeTargetTypesCards, attribute_target_type_entities::AttribtuteTargetTypesEntities, attribute_target_type_players::AtrributeTargetTypesPlayers, attribute_target_type_tiles::AttributeTargetTypesTiles, card_attribute_events::CardAttributeEvents,
        card_attribute_modifier::CardAttributeModifiers, card_master::CardMaster,
    },
    state::state_deck::CardTypes,
};
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
        hashmap.insert(
            String::from("rest"),
            Arc::new(CardMaster::new(
                "rest",
                "card_bump.glb",
                CardTypes::Rest,
                0,
                vec![CardAttributeModifiers::EditEnergyForEntities(AttributeClearFlag::Game, AttribtuteTargetTypesEntities::User, -1)],
                vec![CardAttributeEvents::DiscardCards(AttributeTargetTypesCards::AllUser), CardAttributeEvents::DrawCards(7, AtrributeTargetTypesPlayers::User)],
            )),
        );

        // basic
        hashmap.insert(
            String::from("set+draw"),
            Arc::new(CardMaster::new("set+draw", "card_set.glb", CardTypes::Set, 1, vec![], vec![CardAttributeEvents::DrawCards(1, AtrributeTargetTypesPlayers::User)])),
        );
        hashmap.insert(
            String::from("set+move"),
            Arc::new(CardMaster::new(
                "set+move",
                "card_set.glb",
                CardTypes::Set,
                1,
                vec![],
                vec![CardAttributeEvents::MoveEntity(AttribtuteTargetTypesEntities::User, AttributeTargetTypesTiles::RandomOnTeamUser)],
            )),
        );
        hashmap.insert(String::from("bump"), Arc::new(CardMaster::new("bump", "card_bump.glb", CardTypes::Bump, 1, vec![], vec![CardAttributeEvents::MoveBallForward(1)])));
        hashmap.insert(String::from("spike"), Arc::new(CardMaster::new("spike", "card_spike.glb", CardTypes::Spike, 1, vec![], vec![CardAttributeEvents::MoveBallForward(2)])));

        // spells
        hashmap.insert(
            String::from("curse"),
            Arc::new(CardMaster::new("curse", "card_spike.glb", CardTypes::Spell, 1, vec![], vec![CardAttributeEvents::DiscardCards(AttributeTargetTypesCards::RandomOpponent)])),
        );
        hashmap.insert(
            String::from("blessing"),
            Arc::new(CardMaster::new("blessing", "card_spike.glb", CardTypes::Spell, 1, vec![], vec![CardAttributeEvents::DrawCards(2, AtrributeTargetTypesPlayers::User)])),
        );
        hashmap.insert(
            String::from("deep_breath"),
            Arc::new(CardMaster::new("deep_breath", "card_spike.glb", CardTypes::Spell, 1, vec![], vec![CardAttributeEvents::GainEnergy(2, AttribtuteTargetTypesEntities::User)])),
        );

        hashmap.insert(
            String::from("hold_back"),
            Arc::new(CardMaster::new(
                "hold_back",
                "card_spike.glb",
                CardTypes::Spell,
                1,
                vec![CardAttributeModifiers::EditRangeForEntities(AttributeClearFlag::Turn, AttribtuteTargetTypesEntities::User, -1)],
                vec![],
            )),
        );
        hashmap.insert(
            String::from("extra_oomph"),
            Arc::new(CardMaster::new(
                "extra_oomph",
                "card_spike.glb",
                CardTypes::Spell,
                1,
                vec![(CardAttributeModifiers::EditRangeForEntities(AttributeClearFlag::Turn, AttribtuteTargetTypesEntities::User, 1))],
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
