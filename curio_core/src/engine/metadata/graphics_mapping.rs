use crate::Vector2;

/// A mapping of viewport anchors for one user
#[derive(Clone)]
pub struct GraphicsMapping {
    pub viewport_min: Vector2,
    pub viewport_max: Vector2,
}
impl GraphicsMapping {
    pub fn new(viewport_min: Vector2, viewport_max: Vector2) -> GraphicsMapping {
        GraphicsMapping { viewport_min, viewport_max }
    }
}
