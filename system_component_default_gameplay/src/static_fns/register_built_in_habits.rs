use crate::{
    built_in::habit::{
        system_camera_update_state, system_debug_gui_screen, system_debug_gui_time, system_debug_toggle,
        system_renderer_update_light_state::{self},
        system_renderer_update_state, update_animator_position_sin, update_animator_rotation_sin, update_animator_scale_sin,
    },
    static_data::global_ecs::register_global_ecs,
};
pub fn register_built_in_ecs() {
    register_global_ecs::<system_debug_toggle::Instance>();
    register_global_ecs::<system_debug_gui_time::Instance>();
    register_global_ecs::<system_debug_gui_screen::Instance>();
    register_global_ecs::<system_camera_update_state::Instance>();
    register_global_ecs::<system_renderer_update_state::Instance>();
    register_global_ecs::<system_renderer_update_light_state::Instance>();
    register_global_ecs::<update_animator_scale_sin::Instance>();
    register_global_ecs::<update_animator_rotation_sin::Instance>();
    register_global_ecs::<update_animator_position_sin::Instance>();
}
