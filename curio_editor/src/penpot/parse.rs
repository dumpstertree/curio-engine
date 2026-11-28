//! Penpot `get-file` JSON -> the importer tree `figma::convert` already maps to ui_overlay comps.
//! Penpot keeps shapes in a flat per-page map with absolute page coordinates; this rebuilds the
//! tree, makes positions parent-relative and renames types to the Figma spellings `convert` reads.

use crate::figma::FigmaNode;
use crate::penpot::parse_ctx::ParseCtx;

use serde_json::Value;

/// `uuid/zero`: the root frame every page's top-level shapes hang from.
pub const ROOT_ID: &str = "00000000-0000-0000-0000-000000000000";

/// `v[key]` by its kebab-case name, or the camelCase spelling the JSON encoder writes.
pub fn field<'a>(v: &'a Value, kebab: &str) -> Option<&'a Value> {
    let map = v.as_object()?;
    if let Some(found) = map.get(kebab) {
        return Some(found);
    }
    map.get(&camel(kebab))
}

fn camel(kebab: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for c in kebab.chars() {
        if c == '-' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// Strings, with a keyword's leading `:` dropped in case one is encoded as `":frame"`.
pub fn str_of(v: &Value, key: &str) -> Option<String> {
    match field(v, key)? {
        Value::String(s) => Some(s.trim_start_matches(':').to_string()),
        _ => None,
    }
}

/// Numbers, also accepting numeric strings (text styles store `font-size` as `"14"`).
fn num_of(v: &Value, key: &str) -> Option<f64> {
    match field(v, key)? {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn bool_of(v: &Value, key: &str) -> bool {
    matches!(field(v, key), Some(Value::Bool(true)))
}

fn items<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    match field(v, key) {
        Some(Value::Array(list)) => list.as_slice(),
        _ => &[],
    }
}

/// `"#RRGGBB"` -> `"#rrggbb"`; anything else (gradients, `#rgb`, empty) -> `""`.
fn hex_color(raw: &str) -> String {
    let lower = raw.trim().to_lowercase();
    if lower.len() == 7 && lower.starts_with('#') && lower[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return lower;
    }
    String::new()
}

/// First visible solid colour of a `fills` list, and the first image fill's media reference.
fn read_fills(fills: &[Value]) -> (String, Option<(String, String)>) {
    let mut hex = String::new();
    let mut image = None;
    for fill in fills {
        if bool_of(fill, "hidden") {
            continue;
        }
        if let Some(img) = field(fill, "fill-image") {
            if image.is_none() {
                if let Some(id) = str_of(img, "id") {
                    image = Some((id, str_of(img, "mtype").unwrap_or_default()));
                }
            }
            continue;
        }
        if hex.is_empty() {
            hex = hex_color(&str_of(fill, "fill-color").unwrap_or_default());
        }
    }
    (hex, image)
}

/// Text `content` tree (root -> paragraph-set -> paragraph -> leaves): the label with paragraphs
/// on their own lines, plus the first leaf's font size and colour.
fn read_text(content: &Value) -> (String, f32, String) {
    let mut paragraphs = Vec::new();
    let mut font_size = 0.0;
    let mut color = String::new();
    collect_text(content, &mut paragraphs, &mut font_size, &mut color);
    (paragraphs.join("\n"), font_size, color)
}

fn collect_text(node: &Value, paragraphs: &mut Vec<String>, font_size: &mut f32, color: &mut String) {
    if str_of(node, "type").as_deref() == Some("paragraph") {
        paragraphs.push(String::new());
    }
    if let Some(text) = str_of(node, "text") {
        if paragraphs.is_empty() {
            paragraphs.push(String::new());
        }
        paragraphs.last_mut().unwrap().push_str(&text);
        if *font_size == 0.0 {
            *font_size = num_of(node, "font-size").unwrap_or(0.0) as f32;
        }
        if color.is_empty() {
            *color = read_fills(items(node, "fills")).0;
        }
        if color.is_empty() {
            *color = hex_color(&str_of(node, "fill-color").unwrap_or_default());
        }
    }
    for child in items(node, "children") {
        collect_text(child, paragraphs, font_size, color);
    }
}

/// Box of a shape: `x/y/width/height`, falling back to `selrect` (paths and bools may leave them nil).
fn read_box(shape: &Value) -> (f64, f64, f64, f64) {
    let sel = field(shape, "selrect");
    let pick = |key: &str| {
        num_of(shape, key)
            .or_else(|| sel.and_then(|s| num_of(s, key)))
            .unwrap_or(0.0)
    };
    (pick("x"), pick("y"), pick("width"), pick("height"))
}

/// Shape `id` and its subtree; `origin` is the parent's absolute top-left.
pub fn parse_shape(ctx: &mut ParseCtx, id: &str, origin: (f64, f64), depth: usize) -> Option<FigmaNode> {
    let shape = ctx.objects.get(id)?;
    let kind_raw = str_of(shape, "type").unwrap_or_default();
    let name = str_of(shape, "name")
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "unnamed".to_string());
    let (x, y, width, height) = read_box(shape);

    let (mut fill_hex, mut image) = read_fills(items(shape, "fills"));
    if kind_raw == "image" {
        if let Some(meta) = field(shape, "metadata") {
            if let Some(media) = str_of(meta, "id") {
                image = Some((media, str_of(meta, "mtype").unwrap_or_default()));
            }
        }
    }

    let mut text = None;
    let mut font_size = 0.0;
    if kind_raw == "text" {
        let (label, size, color) = field(shape, "content").map(read_text).unwrap_or_default();
        text = Some(label);
        font_size = size;
        if !color.is_empty() {
            fill_hex = color;
        }
    }

    // Flex frames flow like Figma auto-layout; the gap is the one along the flow axis.
    let mut layout_mode = String::new();
    let mut item_spacing = 0.0;
    let layout = str_of(shape, "layout").unwrap_or_default();
    let flex_dir = str_of(shape, "layout-flex-dir").unwrap_or_else(|| "row".to_string());
    let column = flex_dir.starts_with("column");
    if layout == "flex" {
        layout_mode = if column { "VERTICAL".to_string() } else { "HORIZONTAL".to_string() };
        let gap = field(shape, "layout-gap");
        item_spacing = gap
            .and_then(|g| num_of(g, if column { "row-gap" } else { "column-gap" }))
            .unwrap_or(0.0) as f32;
    } else if layout == "grid" {
        layout_mode = "HORIZONTAL".to_string();
        ctx.warnings
            .push(format!("'{name}' uses grid layout: imported as row flow, cells and tracks are dropped"));
    }

    // A component copy (not the main instance) becomes a `base:` reference, like a Figma INSTANCE.
    let mut base = None;
    let is_container = kind_raw == "frame" || kind_raw == "group" || kind_raw == "bool";
    if is_container && !bool_of(shape, "main-instance") {
        if let Some(component_id) = str_of(shape, "component-id") {
            let local = str_of(shape, "component-file")
                .map(|f| f == ctx.file_id)
                .unwrap_or(true);
            let entry = ctx.components.and_then(|map| map.get(&component_id));
            match entry.and_then(|e| str_of(e, "name")) {
                Some(component_name) if local => base = Some(component_name),
                _ => ctx
                    .warnings
                    .push(format!("'{name}' is a copy of a component from another library file: imported inline, no base")),
            }
        }
    }

    let kind = match kind_raw.as_str() {
        "frame" | "group" | "bool" if base.is_some() => "INSTANCE".to_string(),
        "frame" => "FRAME".to_string(),
        "group" | "bool" => "GROUP".to_string(),
        other => other.to_uppercase(),
    };

    if let Some((media, mtype)) = image.clone() {
        if !bool_of(shape, "hidden") {
            ctx.images.push((name.clone(), media, mtype));
        }
    }

    let mut children = Vec::new();
    if depth < 256 {
        for child in items(shape, "shapes") {
            let Some(child_id) = child.as_str() else { continue };
            if let Some(node) = parse_shape(ctx, child_id, (x, y), depth + 1) {
                children.push(node);
            }
        }
    }
    // Flex children follow the visual order along the flow axis, whatever their layer order.
    if layout == "flex" {
        children.sort_by(|a, b| {
            let (pa, pb) = if column { (a.ty, b.ty) } else { (a.tx, b.tx) };
            pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    Some(FigmaNode {
        id: id.to_string(),
        name,
        kind,
        visible: !bool_of(shape, "hidden"),
        tx: (x - origin.0) as f32,
        ty: (y - origin.1) as f32,
        roll_deg: num_of(shape, "rotation").unwrap_or(0.0) as f32,
        width: width as f32,
        height: height as f32,
        layout_mode,
        item_spacing,
        fill_hex,
        font_size,
        image_fill: image.is_some(),
        text,
        base,
        children,
    })
}

/// Top-level shapes of one page (children of its root frame), each parsed with its subtree.
pub fn parse_page(ctx: &mut ParseCtx) -> Vec<FigmaNode> {
    let mut roots = Vec::new();
    let Some(root) = ctx.objects.get(ROOT_ID) else { return roots };
    let ids: Vec<String> = items(root, "shapes")
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    for id in ids {
        if let Some(node) = parse_shape(ctx, &id, (0.0, 0.0), 0) {
            roots.push(node);
        }
    }
    roots
}
