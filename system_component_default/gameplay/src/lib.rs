pub mod static_fns {
    pub mod register_built_in_facets;
    pub mod register_built_in_habits;
}
pub mod traits_internal {
    pub mod ui_common;
    pub mod world_context_common;
}
pub mod built_in {
    pub mod impulse {
        pub mod ui_events;
    }
    pub mod habit {
        // pub mod system_camera_update_state;
        pub mod system_debug_gui_screen;
        pub mod system_debug_gui_time;
        pub mod system_debug_toggle;
        pub mod system_renderer_update_light_state;
        // pub mod system_renderer_update_state;
        pub mod update_animator_position_sin;
        pub mod update_animator_rotation_sin;
        pub mod update_animator_scale_sin;
    }
    pub mod facet {
        pub mod animator_common;
        pub mod animator {
            pub mod animator_position_sin;
            pub mod animator_rotation_sin;
            pub mod animator_scale_sin;
        }
        // pub mod collider {
        //     pub mod collider_box;
        //     pub mod collider_sphere;
        // }
        pub mod transform {
            pub mod transform2d;
            pub mod transform3d;
        }
        // pub mod renderer {
        //     pub mod renderer_dynamic;
        //     pub mod renderer_image;
        //     pub mod renderer_static;
        //     pub mod renderer_text;
        // }
        pub mod tween {
            pub mod tween;
        }
        // pub mod camera;
        // pub mod collider_common;
        // pub mod light;
        // pub mod renderer_common;
    }
}
pub mod traits {
    pub mod facet_common;
    pub mod field_override;
    pub mod habit;
    pub mod impulse;
    pub mod scope;
    pub mod ui_dialog;
    pub mod ui_events;
    pub mod ui_hud;
    pub mod ui_panel;
}
pub mod static_data {
    pub mod global_components;
    pub mod global_ecs;
    pub mod global_event_recievers;
}
pub mod context_2d;
pub mod context_3d;
pub mod form;
pub mod form_ref;
pub mod gameplay_instance;
pub mod system_component_default_gameplay;
