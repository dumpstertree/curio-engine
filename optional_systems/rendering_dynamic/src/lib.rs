pub fn main() {}

pub mod asset {
    pub mod model_asset_animated;
}
pub mod habit {
    pub mod habit_update;
}
pub mod facet {
    pub mod renderer_dynamic;
}
// Spine runtime, adopted from rusty_spine (see spine/README.md). It owns the spine-c host
// callbacks (`_spUtil_readFile`, `_spAtlasPage_createTexture`, ...); override them through
// `spine::extension`, not by defining the symbols again.
pub mod spine;

// pub mod assets {
//     pub mod model_asset_animated;
// }
