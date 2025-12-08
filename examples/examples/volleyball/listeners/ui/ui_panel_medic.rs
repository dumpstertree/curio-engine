use core::collections::{event_queue::EventQueue, game_state::GameState, quaternion::Quaternion, vector3::Vector3};

use built_in::component::{component_renderer_static::Renderer, component_renderer_text::ComponentRendererText, component_transform::Transform};
use built_in_state::state_camera::{self, CameraState};
use system_component_default_gameplay::{GameObject, UI, UIPanel, WorldContext};

pub struct UIPanelMedic {
    go_desc: Option<GameObject>,
    go_opt_0: Option<GameObject>,
    go_opt_1: Option<GameObject>,
}
impl UIPanelMedic {
    pub fn new() -> Box<UIPanelMedic> {
        Box::new(UIPanelMedic { go_desc: None, go_opt_0: None, go_opt_1: None })
    }
}
impl UIPanel for UIPanelMedic {
    fn input_button(button: core::input::key_code::ButtonCode, state: core::collections::input_button::InputButtonState) {
        todo!()
    }

    fn input_axis(axis: core::input::axis_code::AxisCode, state: core::collections::input_cursor::InputAxisState) {
        todo!()
    }
}
impl UI for UIPanelMedic {
    fn init(&mut self) {}

    fn present(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext) {
        // create obj
        let go_desc = context.instantiate();
        go_desc.add_component_default::<Transform>();
        go_desc.add_component_default::<ComponentRendererText>();

        let go_opt_0 = context.instantiate();
        go_desc.add_component_default::<Transform>();
        go_desc.add_component_default::<ComponentRendererText>();

        let go_opt_1 = context.instantiate();
        go_desc.add_component_default::<Transform>();
        go_desc.add_component_default::<ComponentRendererText>();

        // save
        self.go_desc = Some(go_desc);
        self.go_opt_0 = Some(go_opt_0);
        self.go_opt_1 = Some(go_opt_1);
    }

    fn dismiss(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext) {
        context.clear();
    }

    fn tick(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext) {
        let state_camera = game_state.get::<CameraState>();
        let z = 1.0;
        let x = 0.0;
        let y = 0.0;
        let rotation = game_state.get::<CameraState>().cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
        let position = state_camera.cameras.position + (state_camera.cameras.rotation * Vector3::forward()) * z + state_camera.cameras.rotation * Vector3::down() * y + state_camera.cameras.rotation * Vector3::right() * x;

        // edit a
        if let Some(a) = &self.go_desc {
            a.edit_component::<Transform>(|x| {
                //
                x.position = position;
                x.rotation = rotation;
                x.scale = Vector3::one();
            });
            // edit text renderer
            a.edit_component::<ComponentRendererText>(|x| {
                x.set_contents("setup contents!!");
            });
        }
    }
}
