pub fn main() {}

use std::rc::Rc;

pub use crate::data::draw_call::DrawCall;
pub use crate::data::material::Material;
pub use crate::data::mesh::Mesh;
// pub use crate::facet::renderer_common::RendererCommon;
pub use crate::record::sys_record_rendering::SysRecordRendering;

pub mod record {
    pub mod sys_record_rendering;
}
pub mod data {
    pub mod draw_call;
    pub mod material;
    pub mod mesh;
}
pub mod facet {
    pub mod renderer_common;
}

pub trait RenderingAccesor {
    fn rendering(&self) -> Rc<SysRecordRendering>;
}
impl RenderingAccesor for curio_core::Ledger {
    fn rendering(&self) -> Rc<SysRecordRendering> {
        self.read::<SysRecordRendering>()
    }
}
