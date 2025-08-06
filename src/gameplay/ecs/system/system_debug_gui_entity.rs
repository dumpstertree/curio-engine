use crate::{
    gameplay::ecs::{component::component_transform::Transform, traits::ecs_system::ECSSystemEventless},
    system::system_game_states::{
        state_camera::CameraState,
        state_gizmos::GizmosState,
        state_gui::{GUIState, GuiElement, GuiWindow},
        state_input,
    },
    Collections::{event_queue::EventQueue2, game_state::GameState, gizmo::Gizmo, matrix4x4::Matrix4x4, vector3::Vector3, Color::Color},
};
use ecs_system::ECSSystem;
use hecs::{Entity, World};

#[ECSSystem]
pub struct SystemDebugGuiEntity {}
impl SystemDebugGuiEntity {}
impl SystemDebugGuiEntity {
    pub fn new() -> Box<SystemDebugGuiEntity> {
        Box::new(SystemDebugGuiEntity {})
    }
}
impl ECSSystemEventless for SystemDebugGuiEntity {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }

    fn debug(&mut self, game_state: &mut GameState, world: &mut World, system_event_queue: &mut EventQueue2) {
        let state_camera = game_state.get_value2::<CameraState>();
        let state_input = game_state.get_value2::<state_input::InputState>();

        // let mut closest: Option<Entity> = None;
        let mut entityy: Option<Entity> = None;
        let mut distance = 99999999.0;
        let mut screen_pos = Vector3::zero();
        let mut matrix = Matrix4x4::default();
        for (entity, (transform)) in world.query::<(&Transform)>().iter() {
            let Some(p) = state_camera.world_to_screen(transform.position) else {
                continue;
            };

            let d = (Vector3::new(p.0, p.1, 0.0) - state_input.cursor.position).magnitude();
            let is_less = d < distance;
            if !is_less {
                continue;
            }

            // closest = Some(entity);
            distance = d;
            screen_pos = Vector3::new(p.0, p.1, 0.0);
            matrix = transform.get_matrix();
            entityy = Some(entity);
        }

        game_state.edit::<GUIState>(|x| {
            let mut w = GuiWindow::new(String::from("cur"), screen_pos.clone(), Vector3::zero());
            w.add(GuiElement::new_label(
                String::from("HERE"),
                20.0,
                crate::Collections::Color::Color::get_red(),
            ));

            x.guis.push(w);
        });

        game_state.edit::<GizmosState>(|x| {
            x.draw_calls
                .push(Gizmo::cube(matrix, Vector3::one() * 3.0, Color::get_blue()));
        });
    }
}
