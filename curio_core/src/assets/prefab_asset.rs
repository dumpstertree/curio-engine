use serde::{Deserialize, Serialize};

use crate::{assets::asset::AssetCommonFromBits, AssetCommon};

#[derive(Serialize, Deserialize)]
pub struct PrefabGameObject {
    pub name: String,
    pub components: Vec<PrefabComponent>,
    pub children: Vec<PrefabGameObject>,
}
impl AssetCommon for PrefabGameObject {}
impl AssetCommonFromBits<PrefabGameObject> for PrefabGameObject {
    fn from_bits(bits: &Vec<u8>) -> PrefabGameObject {
        let string = String::from_utf8(bits.to_vec()).unwrap();
        return serde_yaml::from_str::<PrefabGameObject>(&string).unwrap();
    }
}
#[derive(Serialize, Deserialize)]
pub struct PrefabComponent {
    pub r#type: String,
    pub fields: Vec<String>,
}
