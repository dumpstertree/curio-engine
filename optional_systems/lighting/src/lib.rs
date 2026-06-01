pub use facet::light::Light;
pub use light_uniform::DrawCallLight;
pub use light_uniform::LightSystem;
pub use light_uniform::LightType;
pub use record::sys_record_lights::SysRecordLights;
pub use record::sys_record_sun::SysRecordSun;

pub(crate) mod record {
    pub(crate) mod sys_record_lights;
    pub(crate) mod sys_record_sun;
}
pub(crate) mod facet {
    pub(crate) mod light;
}

pub(crate) mod habit {
    pub(crate) mod update_lights;
}
pub(crate) mod light_uniform;
