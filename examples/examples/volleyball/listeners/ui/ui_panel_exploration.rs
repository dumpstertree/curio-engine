use curio_core::{
    built_in::record::{state_input::InputState, state_time::TimeState},
    collections::{event_queue::EventQueue, game_state::GameState, input_cursor::InputAxisState, key_state::KeyState, vector2::Vector2, vector3::Vector3},
    input::{axis_code::AxisCode, key_code::ButtonCode},
};

use system_component_default_gameplay::{
    built_in::facet::{
        renderer::renderer_text::{AligmentHorizontal, AligmentVertical, RendererText},
        transform::transform2d::Transform2D,
    },
    context_2d::Context2D,
    form::Form,
    traits::ui_panel::UIPanel,
    traits_internal::ui_common::UICommon,
};

use crate::{
    exploration::exploration_path::{Room, RoomTypes},
    game_events::GameEvents,
    state::host::state_exploration::StateExploration,
};

pub struct UIPanelInstance {
    selected_index: i32,
    go_desc: Option<Form>,
    go_opts: Vec<Form>,
    rooms: Vec<Room>,
}
impl UIPanelInstance {
    pub fn new() -> Box<UIPanelInstance> {
        Box::new(UIPanelInstance {
            selected_index: 0,
            go_desc: None,
            go_opts: Vec::new(),
            rooms: Vec::new(),
        })
    }
}
impl UIPanel for UIPanelInstance {
    fn input_button(&mut self, button: ButtonCode, state: KeyState) {}

    fn input_axis(&mut self, axis: AxisCode, state: InputAxisState) {}
}
impl UICommon for UIPanelInstance {
    fn init(&mut self) {}

    fn present(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut Context2D) {
        let mut rend = RendererText::default();
        rend.set_contents("Where to go next?");
        // create obj
        let go_desc = context
            .spawn("text.description", Transform2D::default().set_position_01(Vector2::new(0.5, 0.5)))
            .add_facet(rend);

        let next_rooms = game_state
            .get::<StateExploration>()
            .exploration
            .get_next_room();

        self.rooms = next_rooms.clone();

        for i in 0..next_rooms.len() {
            let x_pos = 0.5 + (-1.0 * (next_rooms.len() as f32 * 0.2) / 2.0) + i as f32 * 0.2;
            let r = &next_rooms[i];
            let rt = match r.room_type {
                RoomTypes::Invalid => "invalid",
                RoomTypes::Combat => "combat",
                RoomTypes::Heal => "heal",
                RoomTypes::Shop => "shop",
                RoomTypes::Boss => "boss",
            };

            let mut rend = RendererText::default();
            rend.set_contents(rt);
            rend.set_horizontal_alignment(AligmentHorizontal::Center);
            rend.set_vertical_alignment(AligmentVertical::Center);
            // rend.set_bounds(Vector2::zero());

            println!("room {} ", rt);
            let go_opt_0 = context
                .spawn("text.option_0", Transform2D::default().set_position_01(Vector2::new(x_pos, 0.4)))
                .add_facet(rend);
            self.go_opts.push(go_opt_0);
        }

        // save
        self.go_desc = Some(go_desc);
    }

    fn dismiss(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut Context2D) {
        self.go_desc.clone().unwrap().destroy();
        for x in &self.go_opts {
            x.destroy();
        }
        self.go_opts.clear();
        // self.go_opts.clone().unwrap().destroy();
        // self.go_opt_1.clone().unwrap().destroy();
    }

    fn tick(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut Context2D) {
        let input_state = game_state.get::<InputState>();
        if input_state.mapped.len() > 0 {
            if input_state.mapped[0]
                .get_button_or_default("move_left")
                .went_up
            {
                self.selected_index += 1;
                if self.selected_index >= self.go_opts.len() as i32 {
                    self.selected_index = 0;
                }
            }
            if input_state.mapped[0]
                .get_button_or_default("move_right")
                .went_up
            {
                self.selected_index -= 1;
                if self.selected_index < 0 {
                    self.selected_index = (self.go_opts.len() - 1) as i32;
                }
            }
            if input_state.mapped[0]
                .get_button_or_default("turn_end")
                .went_up
            {
                let r = self.rooms.get(self.selected_index as usize).unwrap();
                event_queue.enqueue_event(GameEvents::ExplorationPickRoomComplete(r.clone()));
            }
        }

        let sin = f32::sin(game_state.get::<TimeState>().unscaled_time as f32 * 5.0);
        for i in 0..self.go_opts.len() {
            let go = &self.go_opts[i];
            go.edit_facet::<Transform2D>(|x| {
                if i as i32 == self.selected_index {
                    x.scale = Vector3::one() * 0.5 + Vector3::one() * 0.1 * sin;
                    // x.position = Vector2::new(0.5, 0.5);
                } else {
                    x.scale = Vector3::one() * 0.5;
                    // x.position = Vector2::new(0.5, 0.5);
                }
            });
        }
        // if let Some(a) = &self.go_opts {
        //     a.edit_component::<Transform2D>(|x| x.scale = Vector3::one() * 0.5 + Vector3::one() * if self.selected_index == 0 { sin * 0.1 } else { 0.0 });
        // }
    }
}
