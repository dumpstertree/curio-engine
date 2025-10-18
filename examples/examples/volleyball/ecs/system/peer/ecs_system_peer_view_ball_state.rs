use built_in::component::{
    component_renderer_text::{AligmentHorizontal, AligmentVertical, ComponentRendererText},
    component_transform::Transform,
};
use built_in_state::state_camera::CameraState;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState, quaternion::Quaternion, vector3::Vector3},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_system::global_ecs_system;
use hecs::World;

use crate::{
    ecs::components::{component_ui_ball_state::ComponentUIBallState, component_ui_score::ComponentUIScoreState, component_ui_turn::ComponentUITurnState},
    state::{
        state_ball_mode::StateBallMode,
        state_score::StateScore,
        state_teams::{StateTeamAssignments, Teams},
        state_turn::StateTurn,
    },
};

#[global_ecs_system]
pub struct ECSSytem {}
impl ECSSystemEventless for ECSSytem {
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn enable(&mut self, _: &mut GameState, world: &mut World, _: &mut EventQueue) {
        world.spawn((Transform::default(), ComponentRendererText::default(), ComponentUIBallState::default()));
        world.spawn((Transform::default(), ComponentRendererText::default(), ComponentUITurnState::default()));
        world.spawn((Transform::default(), ComponentRendererText::default(), ComponentUIScoreState::default()));
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut World, events: &mut EventQueue) {
        let camera_state = game_state.get_value2::<CameraState>();
        for (_, (transform, renderer, _)) in world
            .query::<(&mut Transform, &mut ComponentRendererText, &ComponentUIBallState)>()
            .iter()
        {
            renderer.set_horizontal_alignment(AligmentHorizontal::Center);
            renderer.set_vertical_alignment(AligmentVertical::Center);
            renderer.set_font_size(0.055);

            transform.position = camera_state.cameras.position + camera_state.cameras.rotation * Vector3::new(0.0, 0.4, 1.0);
            transform.rotation = camera_state.cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
            match game_state.get_value2::<StateBallMode>().mode {
                crate::state::state_ball_mode::BallModes::Serve => renderer.set_contents("SERVE"),
                crate::state::state_ball_mode::BallModes::Bump => renderer.set_contents("BUMP"),
                crate::state::state_ball_mode::BallModes::Set => renderer.set_contents("SET"),
                crate::state::state_ball_mode::BallModes::Spike => renderer.set_contents("SPIKE"),
                crate::state::state_ball_mode::BallModes::Scored => renderer.set_contents("SCORED"),
            };
        }

        for (_, (transform, renderer, _)) in world
            .query::<(&mut Transform, &mut ComponentRendererText, &ComponentUIScoreState)>()
            .iter()
        {
            renderer.set_horizontal_alignment(AligmentHorizontal::Center);
            renderer.set_vertical_alignment(AligmentVertical::Center);
            renderer.set_font_size(0.075);

            transform.position = camera_state.cameras.position + camera_state.cameras.rotation * Vector3::new(0.0, 0.5, 1.0);
            transform.rotation = camera_state.cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
            let state_scores = game_state.get_value2::<StateScore>();
            let score_red = state_scores.all_scores.get(&Teams::Red).unwrap_or(&0);
            let score_blue = state_scores.all_scores.get(&Teams::Blue).unwrap_or(&0);
            renderer.set_contents(&format!("{} - {}", score_red, score_blue));
        }

        let state_turn = game_state.get_value2::<StateTurn>();

        let state_team = game_state.get_value2::<StateTeamAssignments>();
        for (_, (transform, renderer, _)) in world
            .query::<(&mut Transform, &mut ComponentRendererText, &ComponentUITurnState)>()
            .iter()
        {
            renderer.set_horizontal_alignment(AligmentHorizontal::Left);
            renderer.set_vertical_alignment(AligmentVertical::Center);
            renderer.set_font_size(0.05);

            transform.position = camera_state.cameras.position + camera_state.cameras.rotation * Vector3::new(0.5, 0.5, 1.0);
            transform.rotation = camera_state.cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));

            let Some(team) = state_team.team_for(&state_turn.active_instance_id) else {
                continue;
            };

            match team {
                crate::state::state_teams::Teams::Red => renderer.set_contents("Red Team"),
                crate::state::state_teams::Teams::Blue => renderer.set_contents("Blue Team"),
            };
        }
    }
}
