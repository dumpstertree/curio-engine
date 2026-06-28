use serde::{Deserialize, Serialize};

/// Representation of a Facet to be stored as part of a composition.
#[derive(Serialize, Deserialize)]
pub struct CompositionFacet {
    pub r#type: String,
    pub fields: Vec<String>,
}
