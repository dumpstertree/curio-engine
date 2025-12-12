use core::{
    collections::{
        event_queue::EventQueue,
        game_state::{self, GameState},
        vector2::Vector2,
        vector3::Vector3,
    },
    gameplay::{
        ecs::component::component_transform2d::Transform2D,
        world_context::{GameObject, WorldContext2D},
    },
    io::{
        asset_database::{self, AssetDatabase, AssetDatabaseListing},
        asset_loader::AssetLoader,
    },
};

use built_in::component::{component_renderer_animated::RendererAnimated, component_renderer_text::ComponentRendererText};
use built_in_state::{state_input::InputState, state_time::TimeState};
use system_component_default_gameplay::{UI, UIPanel};

use crate::{
    AssetMappingUIDs,
    game_events::GameEvents,
    state::{
        state_energy::StateEnergy,
        state_teams::{StateTeamAssignments, Teams},
    },
    ui_hud_encounter_ball_mode,
};

pub struct UIHUD {
    go_energy_0: Vec<GameObject>,
    go_energy_1: Vec<GameObject>,
}
impl UIHUD {
    pub fn new() -> Box<UIHUD> {
        Box::new(UIHUD { go_energy_0: Vec::new(), go_energy_1: Vec::new() })
    }
}
impl UIPanel for UIHUD {
    fn input_button(&mut self, button: core::input::key_code::ButtonCode, state: core::collections::key_state::KeyState) {}
    fn input_axis(&mut self, axis: core::input::axis_code::AxisCode, state: core::collections::input_cursor::InputAxisState) {}
}
impl UI for UIHUD {
    fn init(&mut self) {}

    fn present(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext2D) {
        println!("present hud encounter");
        let asset = AssetLoader::load_model_animated_from_database(AssetMappingUIDs::EnergyToken.uid());
        let x_offset = 0.15;
        let y_start = 0.75;
        let y_spacing = -0.05;
        for i in 0..10 {
            let mut r = RendererAnimated::default();
            r.set_fps(60);
            r.set_asset(Some(asset.clone()));
            r.set_animation("add", false);

            let mut rr = RendererAnimated::default();
            rr.set_fps(60);
            rr.set_asset(Some(asset.clone()));
            rr.set_animation("add", false);
            //create
            let go_0 = context
                .instantiate(
                    &format!("animated.energy_0_{}", i),
                    Transform2D::default()
                        .set_scale(Vector3::one() * 0.05)
                        .set_position_01(Vector2::new(x_offset, y_start + i as f32 * y_spacing)),
                )
                .add_component_value(r);
            let go_1 = context
                .instantiate(
                    &format!("animated.energy_1_{}", i),
                    Transform2D::default()
                        .set_scale(Vector3::one() * 0.05)
                        .set_position_01(Vector2::new(1.0 - x_offset, y_start + i as f32 * y_spacing)),
                )
                .add_component_value(rr);

            //collect
            self.go_energy_0.push(go_0);
            self.go_energy_1.push(go_1);
        }
    }

    fn dismiss(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext2D) {
        for x in &self.go_energy_0 {
            x.destroy();
        }
        for x in &self.go_energy_1 {
            x.destroy();
        }
        self.go_energy_0.clear();
        self.go_energy_1.clear();
    }

    fn tick(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext2D) {
        let state_energy = game_state.get::<StateEnergy>();
        let state_teams = game_state.get::<StateTeamAssignments>();

        for t in state_teams.team_assignments {
            match t.0 {
                Teams::Red => {
                    if let Some(e) = state_energy.all_players.get(&t.1[0]) {
                        for i in 0..self.go_energy_0.len() {
                            let go = &self.go_energy_0[i];
                            let is_enabled = i < e.0.try_into().unwrap();
                            go.edit_component::<RendererAnimated>(|x| {
                                x.set_animation(if is_enabled { "add" } else { "remove" }, false);
                            })
                        }
                    }
                }
                Teams::Blue => {
                    if let Some(e) = state_energy.all_players.get(&t.1[0]) {
                        for i in 0..self.go_energy_1.len() {
                            let go = &self.go_energy_1[i];
                            let is_enabled = i < e.0.try_into().unwrap();
                            go.edit_component::<RendererAnimated>(|x| {
                                x.set_animation(if is_enabled { "add" } else { "remove" }, false);
                            })
                        }
                    }
                }
            }
        }
    }
}
