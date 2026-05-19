use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct ComponentData {
    pub name: String,
    pub fields: serde_json::Value,
}

#[derive(Serialize, Clone)]
pub struct EntityData {
    pub id: u64,
    pub name: String,
    pub children: Vec<EntityData>,
    pub components: Vec<ComponentData>,
}

#[derive(Serialize, Clone)]
pub struct SceneSnapshot {
    pub entities: Vec<EntityData>,
}
