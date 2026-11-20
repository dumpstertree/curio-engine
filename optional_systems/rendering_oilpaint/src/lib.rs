pub use feature::render_feature_draw_dabs::RenderFeatureDrawDabs;
pub use feature::render_feature_draw_dabs::OilPaintFeature;
pub use feature::render_feature_draw_dabs::Dab;
pub use data::mesh_sampler::sample_mesh_surface;
pub use facet::paint_oil::PaintOil;
pub use record::sys_record_oil_paint::SysRecordOilPaint;

pub(crate) mod data {
    pub(crate) mod mesh_sampler;
}
pub(crate) mod feature {
    pub(crate) mod render_feature_draw_dabs;
}
pub(crate) mod facet {
    pub(crate) mod paint_oil;
}
pub(crate) mod record {
    pub(crate) mod sys_record_oil_paint;
}
pub(crate) mod habit {
    pub(crate) mod paint_dabs;
}
