use curio_core::Color;

/// What a UI node paints.
///
/// A Panel is a flex container: taffy sizes it and arranges its children in
/// flow. Every other kind is a leaf — its `Transform2D.position` places it in
/// pixels (relative to the nearest UI ancestor) and it paints one quad.
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
        }.to_owned()
    }
}

/// RGBA in 0..1.
///
/// `curio_core::Color` has no `Default` (its constructors are all explicit),
/// but every facet field needs one — so UI stores this plain struct and
/// converts to/from `Color` at the edges.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct UiColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Default for UiColor {
    // an authored box that never got a colour should still be visible
    fn default() -> UiColor {
        UiColor::white()
    }
}
impl UiColor {
    pub const fn new_01(r: f32, g: f32, b: f32, a: f32) -> UiColor {
        UiColor { r, g, b, a }
    }
    pub const fn white() -> UiColor {
        UiColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }
    }
    pub const fn clear() -> UiColor {
        UiColor { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
    }
    pub fn from_color(color: Color) -> UiColor {
        UiColor::new_01(color.as_r_01(), color.as_g_01(), color.as_b_01(), color.as_a_01())
    }
    pub fn to_color(&self) -> Color {
        Color::new_01(self.r, self.g, self.b, self.a)
    }
    // hover/click feedback: scale the rgb channels, keep the alpha
    pub fn brighten(&self, amount: f32) -> UiColor {
        UiColor::new_01(clamp_01(self.r * amount), clamp_01(self.g * amount), clamp_01(self.b * amount), self.a)
    }
}

fn clamp_01(value: f32) -> f32 {
    if value < 0.0 {
        0.0
    } else if value > 1.0 {
        1.0
    } else {
        value
    }
}

/// A resolved box in screen pixels. x/y is the top-left corner, y grows down.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct UiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl UiRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> UiRect {
        UiRect { x, y, width, height }
    }
    pub const fn zero() -> UiRect {
        UiRect::new(0.0, 0.0, 0.0, 0.0)
    }
    pub fn area(&self) -> f32 {
        self.width * self.height
    }
    /// Is a screen-space point inside this box? Used for hover/click routing.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}
