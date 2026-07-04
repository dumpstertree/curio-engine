use std::collections::HashMap;

use serde::Serialize;

#[derive(Default, Clone, Serialize)]
pub struct PluginGroupState {
    //populates the left menu. ids are used for the dropdown and value are all the tabs in the tabgroup
    pub id_for_tabs: HashMap<String, Vec<PluginState>>,
}

#[derive(Default, Clone, Serialize)]
pub struct PluginState {
    // name of the tab
    pub tab_name: String,
    // all the objects to display vertically
    pub objects: Vec<ObjectState>,
}

#[derive(Default, Clone, Serialize)]
pub struct ObjectState {
    // name of object
    pub object_name: String,
    // objects can be recusive but dont have to be
    pub children: Vec<ObjectState>,
    // when clicked this data is populated into the inspector
    pub components: Vec<ComponentState>,
}

#[derive(Default, Clone, Serialize)]
pub struct ComponentState {
    // name of component
    pub component_name: String,
    // all the actual data in the component
    pub fields: Vec<FieldState>,
}
#[derive(Default, Clone, Serialize)]
pub struct FieldState {
    // name of the field
    pub field_name: String,
    // serialized data in the field
    pub data: serde_json::Value,
}
impl FieldState {
    pub fn new<T: Serialize>(field_name: &str, value: T) -> FieldState {
        FieldState {
            field_name: field_name.to_string(),
            data: serde_json::to_value(value).unwrap(),
        }
    }
}
