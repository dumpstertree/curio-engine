pub mod component {
    pub mod component_camera;
    pub mod component_camera_index;
    pub mod component_input_index;
    pub mod component_renderer;
    pub mod component_transform;
    pub mod component_colliders {
        pub mod component_collider_box;
        pub mod component_collider_sphere;
    }
}
pub mod system {
    pub mod system_camera_update_state;
    pub mod system_collider_box_update_state;
    pub mod system_collider_sphere_update_state;
    pub mod system_debug_camera;
    pub mod system_debug_gui_colliders;
    pub mod system_debug_gui_collision;
    pub mod system_debug_gui_entity;
    pub mod system_debug_gui_screen;
    pub mod system_debug_gui_time;
    pub mod system_renderer_update_state;
}

pub fn main() {}
