use crate::{AssetCommon, CompositionFacet};
use serde::{Deserialize, Serialize};

/// Representation of a Form hierachy including Facets to be spawned during runtime
#[derive(Serialize, Deserialize)]
pub struct Composition {
    pub enabled: bool,
    pub base: String,
    pub name: String,
    pub components: Vec<CompositionFacet>,
    pub children: Vec<Composition>,
}
impl AssetCommon<Composition> for Composition {
    fn from_bits(bits: &Vec<u8>) -> Composition {
        let string = String::from_utf8(bits.to_vec()).unwrap();
        return serde_yaml::from_str::<Composition>(&string).unwrap();
    }
}
