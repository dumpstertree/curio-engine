use curio_core::{
    collections::network_modes::NetworkModes,
    collections::{event_queue::EventQueue, ledger::Ledger},
    random::Random,
};
use gameplay::{
    context_3d::Context3D,
    traits::{impulse::Impulse, scope::Scope},
};
use impulse::impulse;

use crate::{
    Assets,
    game_events::GameEvents,
    listeners::listener_initialize_encounter::{Encounter, Participant, TeamController},
    state::{
        host::{
            state_exploration::StateExploration,
            state_shop::{Shop, Stock, StockItems},
        },
        state_score::StateScore,
        state_teams::Teams,
    },
};

#[derive(Default)]
#[impulse(GameEvents)]
pub struct Listener {}

// Impl - Instance
impl Scope for Listener {
    fn is_enabled(&mut self, _ledger: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
// Impl - Listener
impl Impulse<GameEvents> for Listener {
    fn dequeue_event(&mut self, ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::InitializeExploration(exploration) => {
                // log
                println!("Exploration Initialized");

                // assign and start the exploration
                ledger.write::<StateExploration>(|x| {
                    x.exploration = exploration.clone();
                    x.exploration.start();
                });

                // get the state
                let state_exploration = ledger.read::<StateExploration>();

                // get the exploration from the state
                let cur_exploration = &state_exploration.exploration;

                // set the healthpoint total
                ledger.write::<StateScore>(|x| {
                    x.all_scores.insert(Teams::Red, 10);
                });

                // enter the new room
                event_queue.enqueue_event(GameEvents::ExplorationRoomEnter(cur_exploration.get_cur_room()));

                // completion event
                event_queue.enqueue_event(GameEvents::DidInitializeExploration(cur_exploration.clone()));
            }
            _ => {}
        }
    }
}

pub struct EncounterLibrary {}

impl EncounterLibrary {
    pub fn random() -> Encounter {
        match Random::range_int(0, 3) {
            0 => Self::get_encounter_0(),
            1 => Self::get_encounter_1(),
            2 => Self::get_encounter_2(),
            _ => {
                panic!("Bad Roll")
            }
        }
    }
    fn get_encounter_0() -> Encounter {
        Encounter {
            server: Teams::Red,
            team_red: TeamController::Player,
            team_blue: TeamController::Ai(vec![
                //
                OpponentLibrary::get_opponent_grunt(),
                OpponentLibrary::get_opponent_grunt(),
            ]),
        }
    }
    fn get_encounter_1() -> Encounter {
        Encounter {
            server: Teams::Red,
            team_red: TeamController::Player,
            team_blue: TeamController::Ai(vec![
                //
                OpponentLibrary::get_opponent_crab(),
                OpponentLibrary::get_opponent_crab(),
            ]),
        }
    }
    fn get_encounter_2() -> Encounter {
        Encounter {
            server: Teams::Red,
            team_red: TeamController::Player,
            team_blue: TeamController::Ai(vec![
                //
                OpponentLibrary::get_opponent_grunt(),
            ]),
        }
    }
}

pub struct ShopLibrary {}
impl ShopLibrary {
    pub fn random() -> Shop {
        match Random::range_int(0, 3) {
            0 => Self::get_shop_0(),
            1 => Self::get_shop_1(),
            2 => Self::get_shop_2(),
            _ => {
                panic!("Bad Roll")
            }
        }
    }

    pub fn get_shop_0() -> Shop {
        Shop::new(vec![
            Stock::new(StockItems::Card(String::from("bump")), 50, 1), //
            Stock::new(StockItems::Card(String::from("set")), 25, 1),
            Stock::new(StockItems::Card(String::from("spike")), 75, 1),
        ])
    }
    pub fn get_shop_1() -> Shop {
        Shop::new(vec![
            Stock::new(StockItems::Card(String::from("popsicle")), 25, 1), //
            Stock::new(StockItems::Card(String::from("popsicle")), 25, 1),
            Stock::new(StockItems::Card(String::from("popsicle")), 25, 1),
        ])
    }
    pub fn get_shop_2() -> Shop {
        Shop::new(vec![
            Stock::new(StockItems::Card(String::from("deep_breath")), 100, 1), //
            Stock::new(StockItems::Card(String::from("extra_oomph")), 75, 1),
            Stock::new(StockItems::Card(String::from("blessing")), 100, 1),
        ])
    }
}

pub struct OpponentLibrary {}
impl OpponentLibrary {
    pub fn get_opponent_grunt() -> Participant {
        Participant {
            deck_id: "wild".to_string(),
            visual: Assets::CharGrunt,
            energy: 3,
            health: 1,
        }
    }
    pub fn get_opponent_crab() -> Participant {
        Participant {
            deck_id: "crab".to_string(),
            visual: Assets::CharCrab,
            energy: 6,
            health: 1,
        }
    }
    pub fn get_opponent_human() -> Participant {
        Participant {
            deck_id: "".to_string(),
            visual: Assets::Goblin,
            energy: 4,
            health: 3,
        }
    }
}
