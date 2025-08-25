use crate::collections::game_state::StateOwnerships;

pub trait IState: Clone + Default {
    // fn default() -> T;
    fn id() -> i32;
    fn ownership() -> StateOwnerships {
        StateOwnerships::Instance
    }
    // fn deserialize(bytes: Vec<u8>) -> T {
    // let decoded: T = bincode::decode_from_slice(&bytes.as_slice(), bincode::config::standard()).unwrap();
    // decoded
    // }
}
use serde::{de::DeserializeOwned, Serialize};
pub fn to_bytes<T>(value: &T) -> Vec<u8>
where
    T: Serialize + DeserializeOwned,
{
    bincode::serialize(value).unwrap()
}

pub fn from_bytes<T>(bytes: &[u8]) -> T
where
    T: Serialize + DeserializeOwned,
{
    bincode::deserialize(bytes).unwrap()
}
