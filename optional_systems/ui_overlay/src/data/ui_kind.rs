/// What a UI node paints.
///
/// A Panel is a flex container: taffy sizes it and arranges its children in
/// flow. Every other kind is a leaf — its `Transform3D.position` places it in
/// pixels (relative to the nearest UI ancestor), z is its paint layer, its
/// rotation (about z) and scale turn it about its centre, and it paints one quad.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum UiKind {
    #[default]
    Panel,
    Block,
    Image,
    Button,
    Text,
}

// composition fields arrive as strings, so the enum needs its own hooks
impl UiKind {
    pub fn parse(value: &str) -> Option<UiKind> {
        match value.to_ascii_lowercase().as_str() {
            "panel" => Some(UiKind::Panel),
            "block" => Some(UiKind::Block),
            "image" => Some(UiKind::Image),
            "button" => Some(UiKind::Button),
            "text" => Some(UiKind::Text),
            _ => None,
        }
    }
    pub fn serialize(&self) -> String {
        match self {
            UiKind::Panel => "panel",
            UiKind::Block => "block",
            UiKind::Image => "image",
            UiKind::Button => "button",
            UiKind::Text => "text",
        }
        .to_owned()
    }
}
