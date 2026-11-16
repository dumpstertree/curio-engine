use crate::UiKind;
use crate::UiRect;
use curio_core::{Color, FieldState};
use facet::facet;
use gameplay::traits::field_override::FieldOverride;

/// One node of a screen's UI tree, authored on a form and laid out by
/// `update_ui_layout`.
///
/// The facet holds what the author asked for (kind, requested box, flex
/// knobs, colour, label); the layout habit writes back what taffy resolved —
/// `rect` in screen pixels plus `hovered`/`clicked` from the cursor — every
/// tick. A Panel only arranges children; every other kind paints one quad,
/// and a label paints glyphs from the shared font atlas instead of a box.
///
/// Children are not attached by the builder: the author sets `set_parent` on
/// the child's form, and a leaf places itself with a `Transform2D` facet, in
/// pixels below-right of its nearest UI ancestor.
#[facet]
pub struct UiNode {
    pub kind: UiKind,
    /// painter's order within the screen. The author's `z_index` wins, taffy's
    /// own paint order breaks ties
    pub z_index: i32,
    /// requested box. 0 means "let taffy decide" (flex/auto sizing)
    pub width: f32,
    pub height: f32,
    pub min_width: f32,
    pub min_height: f32,
    /// spacing between this panel's children, in pixels
    pub gap: f32,
    /// how much a panel child claim's leftover space in its row
    pub flex_grow: f32,
    /// paint colour. `Color`'s own default is fully clear, so the layout
    /// habit paints white when this is left unset
    pub color: Color,
    /// label painted from the shared font atlas inside this box. Empty means
    /// no text; layout is monospace, one advance per character
    pub text: String,
    /// glyph height in pixels — the shared atlas's native cells are 12 px.
    /// 0 means "don't paint text"
    pub font_size: f32,
    /// filled in by update_ui_layout each tick
    pub rect: UiRect,
    pub hovered: bool,
    pub clicked: bool,
}

impl FieldOverride for UiNode {
    fn apply(&mut self, field: &str, value: &str) {
        let value = value.trim();
        match field {
            "kind" => {
                if let Some(kind) = UiKind::parse(value) {
                    self.kind = kind;
                }
            }
            "z_index" => self.z_index = value.parse().unwrap_or(self.z_index),
            // a bad parse keeps the current value - 0 means auto-size and would
            // silently make an authored box collapse
            "width" => self.width = value.parse().unwrap_or(self.width),
            "height" => self.height = value.parse().unwrap_or(self.height),
            "min_width" => self.min_width = value.parse().unwrap_or(self.min_width),
            "min_height" => self.min_height = value.parse().unwrap_or(self.min_height),
            "gap" => self.gap = value.parse().unwrap_or(self.gap),
            "flex_grow" => self.flex_grow = value.parse().unwrap_or(self.flex_grow),
            // Color parses "#rrggbb[aa]" and "(r, g, b[, a])", both 0..1
            "color" => self.color = value.parse().unwrap_or(self.color),
            "text" => self.text = value.to_string(),
            "font_size" => self.font_size = value.parse().unwrap_or(self.font_size),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("kind", UiKind::serialize(&self.kind)),
            FieldState::new("z_index", self.z_index),
            FieldState::new("width", self.width),
            FieldState::new("height", self.height),
            FieldState::new("min_width", self.min_width),
            FieldState::new("min_height", self.min_height),
            FieldState::new("gap", self.gap),
            FieldState::new("flex_grow", self.flex_grow),
            FieldState::new("color", self.color.as_hex()),
            FieldState::new("text", self.text.clone()),
            FieldState::new("font_size", self.font_size),
        ]
    }
}

impl UiNode {
    pub fn builder() -> UiNodeBuilder {
        UiNodeBuilder {
            kind: UiKind::Panel,
            z_index: 0,
            width: 0.0,
            height: 0.0,
            min_width: 0.0,
            min_height: 0.0,
            gap: 0.0,
            flex_grow: 0.0,
            color: Color::white(),
            text: String::new(),
            font_size: 12.0,
        }
    }
    // resolved box for this node - useful for hit-testing from other systems
    pub fn hit_test(&self, x: f32, y: f32) -> bool {
        self.rect.contains(x, y)
    }
}

/// Builder for `UiNode` — `UiNode::builder().kind(...).size(...).build()`.
/// Each call sets one authored field and hands the builder back, so an
/// author writes only what the node needs and every other field keeps the
/// panel defaults: a Panel with no requested box, flexing its children.
pub struct UiNodeBuilder {
    kind: UiKind,
    z_index: i32,
    width: f32,
    height: f32,
    min_width: f32,
    min_height: f32,
    gap: f32,
    flex_grow: f32,
    color: Color,
    text: String,
    font_size: f32,
}
impl UiNodeBuilder {
    pub fn kind(mut self, kind: UiKind) -> Self {
        self.kind = kind;
        self
    }
    pub fn z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
    pub fn min_size(mut self, min_width: f32, min_height: f32) -> Self {
        self.min_width = min_width;
        self.min_height = min_height;
        self
    }
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
    pub fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.flex_grow = flex_grow;
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    pub fn text(mut self, text: &str) -> Self {
        self.text = String::from(text);
        self
    }
    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }
    pub fn build(self) -> UiNode {
        UiNode {
            kind: self.kind,
            z_index: self.z_index,
            width: self.width,
            height: self.height,
            min_width: self.min_width,
            min_height: self.min_height,
            gap: self.gap,
            flex_grow: self.flex_grow,
            color: self.color,
            text: self.text,
            font_size: self.font_size,
            rect: UiRect::zero(),
            hovered: false,
            clicked: false,
            owner: None,
        }
    }
}
