use crate::{system::record_id::RecordId, Color, RecordCommon, TextureAsset};
use std::sync::{Arc, OnceLock};

static SYS_RECORD_ID: OnceLock<i32> = OnceLock::new();

#[derive(Default, Hash, PartialEq, Eq, Clone)]
pub struct SysRecordSkybox {
    pub skybox: SkyboxTypes,
}
impl SysRecordSkybox {}
impl RecordCommon for SysRecordSkybox {
    fn id() -> i32 {
        *SYS_RECORD_ID.get_or_init(|| RecordId::of::<SysRecordSkybox>())
    }
}

#[derive(Default, Clone, Hash, PartialEq, Eq)]
pub enum SkyboxTypes {
    #[default]
    Defualt,
    Color(Color),
    CubeMap(Arc<TextureAsset>),
}
