use serde::{Deserialize, Serialize};

use crate::{assets::asset::AssetCommonFromBits, AssetCommon};

#[derive(Serialize, Deserialize)]
pub struct Composition {
    pub enabled: bool,
    pub base: String,
    pub name: String,
    pub components: Vec<PrefabComponent>,
    pub children: Vec<Composition>,
}
impl AssetCommon for Composition {}
impl AssetCommonFromBits<Composition> for Composition {
    fn from_bits(bits: &Vec<u8>) -> Composition {
        let string = String::from_utf8(bits.to_vec()).unwrap();
        return serde_yaml::from_str::<Composition>(&string).unwrap();
    }
}
#[derive(Serialize, Deserialize)]
pub struct PrefabComponent {
    pub r#type: String,
    pub fields: Vec<String>,
}
