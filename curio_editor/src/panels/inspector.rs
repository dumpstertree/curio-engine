use crate::state::EditorState;
use crate::theme;
use curio_core::ComponentState;
use eframe::egui::{self, CollapsingHeader, RichText, Ui};
use std::iter::Peekable;
use std::str::Chars;

pub fn show(ui: &mut Ui, state: &EditorState) {
    ui.vertical(|ui| {
        theme::section_title(ui, "Inspector");
        ui.separator();

        let Some(obj) = state.selected_object() else {
            ui.weak("Select an object");
            return;
        };

        ui.label(RichText::new(&obj.object_name).strong());
        let mut meta = format!("{} component{}", obj.components.len(), if obj.components.len() == 1 { "" } else { "s" });
        if !obj.children.is_empty() {
            meta.push_str(&format!(" · {} children", obj.children.len()));
        }
        ui.label(RichText::new(meta).small().color(theme::TEXT_SECONDARY));
        ui.add_space(6.0);

        if obj.components.is_empty() {
            ui.weak("No components");
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (i, comp) in obj.components.iter().enumerate() {
                    component_block(ui, comp, i);
                }
            });
    });
}

fn component_block(ui: &mut Ui, comp: &ComponentState, idx: usize) {
    CollapsingHeader::new(RichText::new(&comp.component_name).color(theme::GREEN))
        .id_salt(("comp", idx))
        .default_open(true)
        .show(ui, |ui| {
            if comp.fields.is_empty() {
                ui.weak("no fields");
                return;
            }
            for (fi, field) in comp.fields.iter().enumerate() {
                // NOTE: `field.data`'s exact Rust type is still unknown (see
                // README § "Assumptions about curio_core's API"), so this can't
                // `match` over the real enum. Instead `parse_debug_value` parses
                // the *text* of its `Debug` output into a small `Value` tree
                // (numbers/bools/strings/2-4 element vectors/nested structs —
                // with the struct/enum's type name itself discarded, since it
                // doesn't add anything the field name doesn't already say), and
                // `render_value` draws that with real egui widgets (DragValue,
                // Checkbox, TextEdit, labeled X/Y/Z/W rows) placed right next to
                // the field name, rather than one long string underneath it.
                // Every widget below sizes itself to the space it's actually
                // given (wraps, or stops at `ui.available_width()`) instead of
                // demanding more — see `app.rs`'s `inspector_panel` for why
                // that matters.
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&field.field_name)
                            .small()
                            .color(theme::TEXT_SECONDARY),
                    );
                    let value = parse_debug_value(&format!("{:?}", field.data));
                    render_value(ui, &value);
                });
                if fi + 1 < comp.fields.len() {
                    ui.add_space(2.0);
                }
            }
        });
}

// ── Rendering ────────────────────────────────────────────────────────────────
//
// Always called with `ui` already inside a horizontal layout (the field's
// name was just added to its left), so every arm here lays out left-to-right
// starting from wherever that cursor is.

fn render_value(ui: &mut Ui, value: &Value) {
    match value {
        Value::Null => {
            ui.weak("—");
        }
        Value::Bool(b) => {
            let mut b = *b;
            ui.add_enabled(false, egui::Checkbox::new(&mut b, ""));
        }
        Value::Number(n) => {
            let mut n = *n;
            ui.add_enabled(false, egui::DragValue::new(&mut n));
        }
        Value::Str(s) => {
            if s.contains('\n') || s.chars().count() > 60 {
                ui.add(egui::Label::new(RichText::new(s).monospace().color(theme::TEXT_PRIMARY)).wrap());
            } else {
                let mut text = s.clone();
                let width = ui.available_width().max(40.0);
                ui.add_enabled(false, egui::TextEdit::singleline(&mut text).desired_width(width));
            }
        }
        Value::Vector(nums, is_color) => {
            const AXIS_LABELS: [&str; 4] = ["X", "Y", "Z", "W"];
            const COLOR_LABELS: [&str; 4] = ["R", "G", "B", "A"];
            let labels = if *is_color { COLOR_LABELS } else { AXIS_LABELS };
            // Fixed-size + clipped instead of `horizontal_wrapped`: this
            // never drops to a second line — if it doesn't fit, it's cut
            // off rather than wrapped. Allocating the exact width up front
            // (rather than letting content decide) also means an overly
            // long row still can't force the panel wider.
            let width = ui.available_width();
            let height = ui.spacing().interact_size.y;
            ui.allocate_ui_with_layout(egui::vec2(width, height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.set_clip_rect(ui.max_rect());
                for (i, n) in nums.iter().enumerate() {
                    ui.label(
                        RichText::new(labels.get(i).copied().unwrap_or("?"))
                            .small()
                            .color(theme::TEXT_MUTED),
                    );
                    let mut n = *n;
                    ui.add_enabled(false, egui::DragValue::new(&mut n));
                }
            });
        }
        Value::List(items) => {
            ui.vertical(|ui| {
                for (i, item) in items.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("[{i}]"))
                                .small()
                                .color(theme::TEXT_MUTED),
                        );
                        render_value(ui, item);
                    });
                }
            });
        }
        Value::Map(fields) => {
            ui.vertical(|ui| {
                for (k, v) in fields {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(k).small().color(theme::TEXT_MUTED));
                        render_value(ui, v);
                    });
                }
            });
        }
        Value::Raw(s) => {
            ui.add(egui::Label::new(RichText::new(s).monospace().color(theme::BLUE)).wrap());
        }
    }
}

// ── Debug-text value parser ─────────────────────────────────────────────────
//
// `field.data`'s Rust type is unknown (see README), so this parses the
// *text* of its compact `{:?}` output into a small tree — good enough to
// recognize numbers, bools, strings, 2-4 element numeric vectors, lists,
// and nested structs generically, whatever the real type is. Struct/enum
// *names* (`Vector3 { .. }`, `Foo(1, 2)`, `Some(x)`, ...) are deliberately
// dropped wherever they wrap a value we can already render meaningfully —
// the field name already labels it, so keeping "Vector3" or "Some" around
// would just be noise.

enum Value {
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Vector(Vec<f64>, bool), // bool: true if this came from r/g/b/a-named fields (render as a color row)
    List(Vec<Value>),
    Map(Vec<(String, Value)>),
    Raw(String),
}

fn parse_debug_value(s: &str) -> Value {
    let mut p = Parser { chars: s.chars().peekable() };
    p.parse_value()
}

struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Parser<'a> {
    fn skip_ws(&mut self) {
        while matches!(self.chars.peek(), Some(c) if c.is_whitespace()) {
            self.chars.next();
        }
    }

    fn parse_value(&mut self) -> Value {
        self.skip_ws();
        match self.chars.peek().copied() {
            Some('"') => refine_str(&self.parse_quoted_string()),
            Some('[') => self.parse_bracketed(),
            Some('{') => self.parse_braced(),
            Some('(') => self.parse_paren(),
            Some(c) if c.is_ascii_digit() || c == '-' || c == '+' => self.parse_number(),
            Some(c) if c.is_alphabetic() || c == '_' => self.parse_ident_led(),
            _ => Value::Raw(self.consume_raw()),
        }
    }

    fn parse_ident(&mut self) -> String {
        let mut out = String::new();
        while matches!(self.chars.peek(), Some(c) if c.is_alphanumeric() || *c == '_') {
            out.push(self.chars.next().unwrap());
        }
        out
    }

    /// An identifier was just read; its own text is discarded once we know
    /// what follows it — see the module doc comment for why.
    fn parse_ident_led(&mut self) -> Value {
        let ident = self.parse_ident();
        self.skip_ws();
        match self.chars.peek().copied() {
            Some('(') => self.parse_paren(),
            Some('{') => self.parse_braced(),
            Some('[') => self.parse_bracketed(),
            _ => match ident.as_str() {
                "true" => Value::Bool(true),
                "false" => Value::Bool(false),
                "None" => Value::Null,
                _ => ident
                    .parse::<f64>()
                    .map(Value::Number)
                    .unwrap_or(Value::Raw(ident)),
            },
        }
    }

    fn parse_number(&mut self) -> Value {
        let mut out = String::new();
        while matches!(self.chars.peek(), Some(c) if c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E')) {
            out.push(self.chars.next().unwrap());
        }
        out.parse::<f64>()
            .map(Value::Number)
            .unwrap_or(Value::Raw(out))
    }

    fn parse_quoted_string(&mut self) -> String {
        self.chars.next(); // opening quote
        let mut out = String::new();
        while let Some(c) = self.chars.next() {
            match c {
                '"' => break,
                '\\' => match self.chars.next() {
                    Some('n') => out.push('\n'),
                    Some('r') => out.push('\r'),
                    Some('t') => out.push('\t'),
                    Some(other) => out.push(other), // covers \" and \\
                    None => {}
                },
                other => out.push(other),
            }
        }
        out
    }

    /// Consumes a comma-separated list of values up to (and including) `close`.
    fn parse_items(&mut self, close: char) -> Vec<Value> {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.chars.peek() == Some(&close) {
                self.chars.next();
                break;
            }
            if self.chars.peek().is_none() {
                break;
            }
            items.push(self.parse_value());
            self.skip_ws();
            match self.chars.peek() {
                Some(',') => {
                    self.chars.next();
                }
                Some(c) if *c == close => {
                    self.chars.next();
                    break;
                }
                _ => break,
            }
        }
        items
    }

    fn parse_paren(&mut self) -> Value {
        self.chars.next(); // '('
        finish_sequence(self.parse_items(')'))
    }

    fn parse_bracketed(&mut self) -> Value {
        self.chars.next(); // '['
        finish_sequence(self.parse_items(']'))
    }

    /// Parses `{ key: value, ... }` — either named-struct fields or a bare
    /// map (e.g. a `HashMap`'s `Debug` output, which has no leading name).
    fn parse_braced(&mut self) -> Value {
        self.chars.next(); // '{'
        let mut fields = Vec::new();
        loop {
            self.skip_ws();
            if self.chars.peek() == Some(&'}') {
                self.chars.next();
                break;
            }
            if self.chars.peek().is_none() {
                break;
            }
            let key = if self.chars.peek() == Some(&'"') { self.parse_quoted_string() } else { self.parse_ident() };
            self.skip_ws();
            if self.chars.peek() == Some(&':') {
                self.chars.next();
            }
            let value = self.parse_value();
            fields.push((key, value));
            self.skip_ws();
            match self.chars.peek() {
                Some(',') => {
                    self.chars.next();
                }
                Some('}') => {
                    self.chars.next();
                    break;
                }
                _ => break,
            }
        }
        vector_from_fields(&fields)
            .map(|(nums, is_color)| Value::Vector(nums, is_color))
            .unwrap_or(Value::Map(fields))
    }

    /// Fallback for anything that didn't match a recognized shape — just
    /// grabs whatever's left up to the next natural boundary.
    fn consume_raw(&mut self) -> String {
        let mut out = String::new();
        while let Some(c) = self.chars.peek() {
            if matches!(c, ',' | ')' | '}' | ']') {
                break;
            }
            out.push(*c);
            self.chars.next();
        }
        out.trim().to_string()
    }
}

/// A `Some(x)`/`Ok(x)`/tuple-struct-with-one-field wrapper is more useful
/// shown as its inner value directly; a multi-item tuple/array that isn't
/// a 2-4 element numeric vector just becomes a plain list.
fn finish_sequence(items: Vec<Value>) -> Value {
    if let Some(nums) = vector_from_items(&items) {
        return Value::Vector(nums, false);
    }
    if items.len() == 1 {
        return items.into_iter().next().unwrap();
    }
    Value::List(items)
}

fn vector_from_items(items: &[Value]) -> Option<Vec<f64>> {
    if !(2..=4).contains(&items.len()) {
        return None;
    }
    items
        .iter()
        .map(|v| if let Value::Number(n) = v { Some(*n) } else { None })
        .collect()
}

fn vector_from_fields(fields: &[(String, Value)]) -> Option<(Vec<f64>, bool)> {
    let keys: Vec<String> = fields.iter().map(|(k, _)| k.to_lowercase()).collect();
    let key_refs: Vec<&str> = keys.iter().map(String::as_str).collect();
    let is_axis = matches!(key_refs.as_slice(), ["x", "y"] | ["x", "y", "z"] | ["x", "y", "z", "w"]);
    let is_color = matches!(key_refs.as_slice(), ["r", "g", "b"] | ["r", "g", "b", "a"]);
    if !is_axis && !is_color {
        return None;
    }
    let nums: Option<Vec<f64>> = fields
        .iter()
        .map(|(_, v)| if let Value::Number(n) = v { Some(*n) } else { None })
        .collect();
    nums.map(|n| (n, is_color))
}

/// A `Value::Str` might itself be this engine's own encoded-value
/// convention: JSON, or the `"(x,y,z)"` tuple format `prefab_types.rs`
/// uses for on-disk prefab fields. Recognize those and turn them into the
/// same structured `Value`s instead of leaving them as opaque text.
fn refine_str(s: &str) -> Value {
    if let Some(nums) = parse_tuple_string(s) {
        return Value::Vector(nums, false);
    }
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(s) {
        if json.is_object() || json.is_array() {
            return json_to_value(&json);
        }
    }
    Value::Str(s.to_string())
}

fn parse_tuple_string(s: &str) -> Option<Vec<f64>> {
    let inner = s.trim().strip_prefix('(')?.strip_suffix(')')?;
    if inner.trim().is_empty() {
        return None;
    }
    let nums: Vec<f64> = inner
        .split(',')
        .map(|p| p.trim().parse::<f64>())
        .collect::<Result<_, _>>()
        .ok()?;
    if (2..=4).contains(&nums.len()) {
        Some(nums)
    } else {
        None
    }
}

fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => refine_str(s),
        serde_json::Value::Array(items) => {
            let values: Vec<Value> = items.iter().map(json_to_value).collect();
            vector_from_items(&values)
                .map(|nums| Value::Vector(nums, false))
                .unwrap_or(Value::List(values))
        }
        serde_json::Value::Object(map) => {
            let fields: Vec<(String, Value)> = map
                .iter()
                .map(|(k, v)| (k.clone(), json_to_value(v)))
                .collect();
            vector_from_fields(&fields)
                .map(|(nums, is_color)| Value::Vector(nums, is_color))
                .unwrap_or(Value::Map(fields))
        }
    }
}
