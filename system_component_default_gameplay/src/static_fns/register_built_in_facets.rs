use crate::{
    built_in::facet::{
        component_camera::Camera,
        component_light::ComponentLight,
        facet_renderer::{component_renderer_animated::RendererAnimated, component_renderer_static::Renderer, component_renderer_text::ComponentRendererText},
        facet_transform::{component_transform::Transform, component_transform2d::Transform2D},
    },
    static_data::global_components::register_global_component,
};
pub fn register_built_in_component() {
    register_global_component::<Transform>();
    register_global_component::<Transform2D>();
    register_global_component::<Camera>();
    register_global_component::<ComponentLight>();
    register_global_component::<Renderer>();
    register_global_component::<RendererAnimated>();
    register_global_component::<ComponentRendererText>();
}
