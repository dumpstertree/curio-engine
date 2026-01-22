use crate::{
    built_in::facet::{
        animator::{animator_position_sin::AnimatorPositionSin, animator_rotation_sin::AnimatorRotationSin, animator_scale_sin::AnimatorScaleSin},
        camera::Camera,
        light::Light,
        renderer::{renderer_dynamic::RendererDynamic, renderer_static::RendererStatic, renderer_text::RendererText},
        transform::{transform2d::Transform2D, transform3d::Transform3D},
    },
    static_data::global_components::register_global_component,
};
pub fn register_built_in_component() {
    register_global_component::<Transform3D>();
    register_global_component::<Transform2D>();
    register_global_component::<RendererStatic>();
    register_global_component::<RendererDynamic>();
    register_global_component::<RendererText>();
    register_global_component::<AnimatorScaleSin>();
    register_global_component::<AnimatorPositionSin>();
    register_global_component::<AnimatorRotationSin>();
    register_global_component::<Camera>();
    register_global_component::<Light>();
}
