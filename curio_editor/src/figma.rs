//! Figma API -> Curio composition import - first working slice of the plan in
//! `docs/figma-import-plan.md` (stages 1-3, M1+M2).
//!
//! Entry point is the "Assets" right-click context menu (`panels/asset_tree.rs`'s
//! `creation_menu` -> `TreeAction::FigmaDialog`), which opens the dialog in
//! `panels/figma_tab.rs`. Fetch + translate + asset writes run on a background
//! thread reporting through an `mpsc` channel polled each frame (`AssetState::poll_figma`) - the
//! same pattern `AssetState::pending_import` uses for the blocking file picker,
//! since blocking HTTP can't touch the UI thread any more than the picker could.
//!
//! HTTP goes through `curl` as a subprocess (the same `Command`-spawn pattern
//! `state.rs` uses for `cargo`): Figma's REST needs the `X-Figma-Token` header,
//! and the engine's own runtime `fetch_remote_asset` can't set headers at all -
//! which is a big part of why this feature is build-time/editor-side.
//!
//! Mapped so far (output is a ui_overlay screen: forms carry `uinode` facets
//! that `update_ui_layout` lays out with taffy in viewport pixels, and loads
//! through `spawn_prefab_recursive_internal` as-is):
//! - `FRAME`/`GROUP`/`INSTANCE` -> `uinode` panels; auto-layout frames float in
//!   taffy row-flow with `gap` from `itemSpacing`, free-form frames (no
//!   auto-layout) get `absolute: true` + a pixel `transform3d` offset, so they
//!   hang at Figma's own parent-relative position inside the nearest UI
//!   ancestor (the habit's one new authored field - the rest of §7b stays open)
//! - `TEXT` -> `uinode` kind `text` with `text`, `font_size` (px), fill `color`
//! - `RECTANGLE`/shapes -> `uinode` kind `block` flat quads with `color`
//! - image fills -> `uinode` kind `image` with `image` set to the uid of its PNG,
//!   exported next to the `.comp` as a power-of-two texture asset
//! - `INSTANCE` -> `base:` prefab reference named after the Figma component
//! - hidden layers (`visible:false`) -> `enabled: false` (kept, not dropped)
//!
//! Registry note: the component key is `type_name`'s last segment lowercased
//! (`UiNode` -> `uinode`), so the importer emits `kind: "uinode"`, not `ui_node`.
//!
//! Coordinates: leaf placement is a `transform3d` position in Figma's own
//! parent-relative pixels with y DOWN - `ui_overlay`'s UiRect grows down like
//! Figma, so no y-flip. Its z is the paint layer, taken from the layer order.
//!
//! Still parked (plan 7b): flex direction/justify/align fields, authored
//! absolute placement for frames, gradients/strokes/
//! crop modes, constraints. Figma `rendererimage`/`renderer_text` 3D-scene
//! emission was retired with this retarget.

use crate::fs_ops::{get_or_create_meta, import_texture, read_file_bytes, resolve_conflict, write_file_text};
use crate::prefab_resolver::dump_raw;
use crate::prefab_types::{format_tuple, join_field, PrefabComponentRaw, PrefabGameObjectRaw};

use serde_json::{self, Value};

use std::collections::HashMap;
use std::process::{self, Command};
use std::sync::mpsc::{self, Receiver, Sender};

// ─────────────────────────────────────────────────────────────────────────────
// Job events (background thread -> drained by AssetState::poll_figma)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum FigmaEvent {
    Progress(String),
    Done(String),
    Error(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// UI/job state - owned by AssetState so the Asset tab is this feature's whole
// home (dialog, inputs, in-flight job, reported progress)
// ─────────────────────────────────────────────────────────────────────────────

pub struct FigmaState {
    pub dialog_open: bool,
    /// Where opened `.comp`+PNGs land - from whichever row the context menu
    /// fired (mirrors `TreeAction::Import`'s target dir convention).
    pub target_dir: String,
    /// Persistent inputs: egui's `TextEdit::singleline` writes straight back
    /// into these each frame, so they double as the "remember last import"
    /// values when the dialog reopens.
    pub url: String,
    pub token: String,

    /// `(target dir, events)` while a job thread is in flight; cleared by
    /// `poll_figma` once the thread reports `Done`/`Error` or dies.
    pub job: Option<(String, Receiver<FigmaEvent>)>,
    pub progress: Vec<String>,
    pub summary: Option<String>,
    pub error: Option<String>,
}

impl FigmaState {
    pub fn new() -> Self {
        Self {
            dialog_open: false,
            target_dir: String::new(),
            url: String::new(),
            token: String::new(),
            job: None,
            progress: Vec::new(),
            summary: None,
            error: None,
        }
    }

    /// "Fetch + Import" clicked with both inputs filled and no job running.
    pub fn start(&mut self) {
        if self.job.is_some() || self.url.trim().is_empty() || self.token.trim().is_empty() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let token = self.token.trim().to_string();
        let url = self.url.trim().to_string();
        let dir = self.target_dir.clone();
        let job_dir = self.target_dir.clone();
        std::thread::spawn(move || {
            run_import(tx, token, url, dir);
        });
        self.job = Some((job_dir, rx));
        self.progress = Vec::new();
        self.summary = None;
        self.error = None;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Intermediate representation (plan stage 2 - deliberately minimal: only what
// the M2 mapping reads; `width`/`height` carry the absolute box for M3's
// bounds/crop work)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FigmaNode {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub visible: bool,
    // relativeTransform decomposition: translation + rotation about z.
    pub tx: f32,
    pub ty: f32,
    pub roll_deg: f32,
    // absoluteBoundingBox size (now authored straight into UiNode width/height).
    pub width: f32,
    pub height: f32,
    // auto-layout: "HORIZONTAL"/"VERTICAL"/"" + itemSpacing -> UiNode gap.
    pub layout_mode: String,
    pub item_spacing: f32,
    // first SOLID fill as "#rrggbb" (ui_overlay quads take a Color, parsed from hex).
    pub fill_hex: String,
    // TEXT style fontSize in px (0 -> UiNode default 12).
    pub font_size: f32,
    pub image_fill: bool,
    pub text: Option<String>,
    pub base: Option<String>, // INSTANCE -> Figma component name
    pub children: Vec<FigmaNode>,
}

// serde_json member helpers. Destructuring style copied from
// `panels/inspector.rs`'s `json_to_value` - the repo's serde_json `Value`
// exposes Object/Array constructors; helpers return owned copies so the parse
// pass never fights the `Value` borrow checker.

fn member<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    if let Value::Object(map) = v {
        for (k, val) in map.iter() {
            if *k == key {
                return Some(val);
            }
        }
    }
    None
}

fn get_str(v: &Value, key: &str) -> String {
    let Some(inner) = member(v, key) else { return String::new() };
    match inner {
        Value::String(s) => s.to_string(),
        _ => String::new(),
    }
}

fn get_opt_str(v: &Value, key: &str) -> Option<String> {
    let Some(inner) = member(v, key) else { return None };
    match inner {
        Value::String(s) => Some(s.to_string()),
        _ => None,
    }
}

fn get_bool(v: &Value, key: &str, default: bool) -> bool {
    let Some(inner) = member(v, key) else { return default };
    match inner {
        Value::Bool(b) => *b,
        _ => default,
    }
}

/// `v[key]` as a number - for `itemSpacing` and other top-level numeric fields.
fn get_num_flat(v: &Value, key: &str, default: f64) -> f64 {
    let Some(inner) = member(v, key) else { return default };
    if let Value::Number(n) = inner {
        return n.as_f64().unwrap_or(default);
    }
    default
}

/// f32 -> the shortest plain decimal UiNode `apply()` parses (integers lose the
/// `.0`; anything else keeps two places, which taffy is fine with).
pub fn numstr(value: f32) -> String {
    if value == (value as i64 as f32) {
        return (value as i64).to_string();
    }
    let neg = value < 0.0;
    let abs = if neg { 0.0 - value } else { value };
    let scaled = abs * 100.0 + 0.5;
    let whole = (scaled / 100.0).floor();
    let frac = (scaled - whole * 100.0) as i64;
    let mut out = String::new();
    if neg {
        out.push('-');
    }
    out.push_str(&whole.to_string());
    out.push('.');
    if frac < 10 {
        out.push('0');
    }
    out.push_str(&frac.to_string());
    out
}

/// 0..1 channel -> two lowercase hex digits (`Color::parse` reads "#rrggbb").
fn hex_pair(v: f64) -> String {
    let mut n = (v * 255.0 + 0.5).floor();
    if n < 0.0 {
        n = 0.0;
    }
    if n > 255.0 {
        n = 255.0;
    }
    let ni = n as i32;
    let mut out = String::new();
    out.push(digit_char(ni / 16));
    out.push(digit_char(ni % 16));
    out
}

fn digit_char(i: i32) -> char {
    if i < 10 {
        return ((48 + i) as u8) as char;
    }
    ((87 + i) as u8) as char
}

/// `v[outer][inner]` as a number - for `relativeTransform`'s matrix entries
/// and `absoluteBoundingBox`'s size.
fn get_num(v: &Value, outer: &str, inner: &str, default: f64) -> f64 {
    let Some(outside) = member(v, outer) else { return default };
    if let Value::Object(map) = outside {
        for (k, val) in map.iter() {
            if *k == inner {
                if let Value::Number(n) = val {
                    return n.as_f64().unwrap_or(default);
                }
            }
        }
    }
    default
}

pub fn parse_node(v: &Value) -> FigmaNode {
    let kind = get_str(v, "type");
    let name_raw = get_str(v, "name");
    let name = if name_raw.is_empty() { "unnamed".to_string() } else { name_raw };

    let visible = get_bool(v, "visible", true);

    // Figma's REST `relativeTransform` is an affine matrix. The docs show
    // "xx/yx/tx/ty"; responses have also shipped "x/y" column vectors with
    // "i/j" translation - read both spellings so the import survives either.
    let mut xx = get_num(v, "relativeTransform", "xx", 0.0);
    let mut yx = get_num(v, "relativeTransform", "yx", 0.0);
    let txv = get_num(v, "relativeTransform", "tx", get_num(v, "relativeTransform", "i", 0.0));
    let tyv = get_num(v, "relativeTransform", "ty", get_num(v, "relativeTransform", "j", 0.0));
    if xx == 0.0 {
        xx = get_num(v, "relativeTransform", "x", 1.0);
    }
    if yx == 0.0 && get_num(v, "relativeTransform", "y", 0.0) != 0.0 {
        yx = get_num(v, "relativeTransform", "y", 0.0);
    }
    let roll_deg = if xx == 1.0 && yx == 0.0 { 0.0 } else { yx.atan2(xx) * (180.0 / 3.141592653589793) };

    let width = get_num(v, "absoluteBoundingBox", "width", 0.0);
    let height = get_num(v, "absoluteBoundingBox", "height", 0.0);

    let mut image_fill = false;
    if let Some(fills) = member(v, "fills") {
        if let Value::Array(items) = fills {
            for fill in items.iter() {
                if get_str(fill, "type") == "IMAGE" {
                    image_fill = true;
                }
            }
        }
    }

    // first visible SOLID fill -> "#rrggbb" (image fills paint as quads, so
    // the solid pass skips them).
    let mut fill_hex = String::new();
    if !image_fill {
        if let Some(fills) = member(v, "fills") {
            if let Value::Array(items) = fills {
                for fill in items.iter() {
                    if get_str(fill, "type") != "SOLID" {
                        continue;
                    }
                    if !get_bool(fill, "visible", true) {
                        continue;
                    }
                    let r = get_num(fill, "color", "r", 0.0);
                    let g = get_num(fill, "color", "g", 0.0);
                    let b = get_num(fill, "color", "b", 0.0);
                    fill_hex = "#".to_string() + &hex_pair(r) + &hex_pair(g) + &hex_pair(b);
                    break;
                }
            }
        }
    }

    // auto-layout -> panel gap (flow direction is a 7b gap: taffy style is row-
    // only today, VERTICAL frames keep child order and warn).
    let layout_mode = get_str(v, "layoutMode");
    let item_spacing = get_num_flat(v, "itemSpacing", 0.0);

    // TEXT style fontSize in px; UiNode's own default is the atlas's 12.
    let font_size = get_num(v, "style", "fontSize", 0.0);

    let text = get_opt_str(v, "characters");

    let base = get_opt_str(v, "componentId");

    let mut children = Vec::new();
    if let Some(kids) = member(v, "children") {
        if let Value::Array(items) = kids {
            for kid in items.iter() {
                children.push(parse_node(kid));
            }
        }
    }

    FigmaNode {
        id: get_str(v, "id"),
        name: name,
        kind: kind,
        visible: visible,
        tx: txv as f32,
        ty: tyv as f32,
        roll_deg: roll_deg as f32,
        width: width as f32,
        height: height as f32,
        layout_mode: layout_mode,
        item_spacing: item_spacing as f32,
        fill_hex: fill_hex,
        font_size: font_size as f32,
        image_fill: image_fill,
        text: text,
        base: base,
        children: children,
    }
}

/// Layer/document name -> filesystem stem. Slugs keep components' own names
/// intact ("Hero Button" -> `hero_button`) so the `base:` prefab reference
/// reads like a filename; `resolve_conflict` at write time is deliberate:
/// hand-edited comps with the same slug are kept, not clobbered.
pub fn slugify(name: &str) -> String {
    let mut out = String::new();
    for c in name.to_lowercase().chars() {
        out.push(if c.is_alphanumeric() { c } else { '_' });
    }
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    let mut start = 0;
    let mut end = out.len();
    while start < end && out.as_bytes()[start] == b'_' {
        start += 1;
    }
    while end > start && out.as_bytes()[end - 1] == b'_' {
        end -= 1;
    }
    out = out[start..end].to_string();
    if out.is_empty() {
        return "node".to_string();
    }
    out
}

// Tiny template helpers (positional `{0}`/`{1}`) used instead of format
// strings for messages built on the job thread.
fn subst1(templ: &str, a: String) -> String {
    templ.replace("{0}", &a)
}

fn subst2(templ: &str, a: String, b: String) -> String {
    subst1(templ, a).replace("{1}", &b)
}

fn subst4(templ: &str, a: String, b: String, c: String, d: String) -> String {
    subst2(&subst2(templ, a, b), c, d)
}

// ─────────────────────────────────────────────────────────────────────────────
// IR -> the editor's own `.comp` model (`prefab_types`), so what the import
// writes is openable/inspectable by the prefab tab immediately
// ─────────────────────────────────────────────────────────────────────────────

/// IR -> ui_overlay comp: frames/groups/instances become `uinode` panels,
/// leaves carry a `uinode` plus a `transform3d` whose position is Figma's own
/// parent-relative pixels (y down - UiRect grows down like Figma, no flip).
/// Panels float in taffy flow (placement is a plan 7b gap); leaves are exact.
/// `textures` maps an image node's id to the uid of its exported texture.
pub fn convert(node: FigmaNode, order: i32, textures: &HashMap<String, i16>, warnings: &mut Vec<String>) -> PrefabGameObjectRaw {
    let mut components = Vec::new();

    let is_panel = node.kind == "FRAME" || node.kind == "GROUP" || node.kind == "CANVAS" || node.kind == "INSTANCE";

    if is_panel {
        let mut fields = Vec::new();
        fields.push(join_field("kind", "panel"));
        if node.width > 0.0 {
            fields.push(join_field("width", &numstr(node.width)));
        }
        if node.height > 0.0 {
            fields.push(join_field("height", &numstr(node.height)));
        }
        if node.item_spacing > 0.0 {
            fields.push(join_field("gap", &numstr(node.item_spacing)));
        }
        if node.layout_mode == "VERTICAL" {
            warnings.push(subst1("'{0}' uses vertical auto-layout - flow is row-only today, child order kept", node.name.clone()));
        }
        // free-form frames (no auto-layout) keep their Figma placement: the
        // `absolute` flag hangs the panel at its own Transform3D offset inside
        // the nearest UI ancestor, mirroring Figma's parent-relative x/y
        let free_form = node.layout_mode.is_empty() || node.layout_mode == "NONE";
        if free_form {
            fields.push(join_field("absolute", "true"));
        }
        components.push(PrefabComponentRaw { kind: "uinode".to_string(), fields: fields });
        if free_form {
            let mut tf = Vec::new();
            tf.push(join_field("position", &format_tuple(&[node.tx, node.ty, 0.0])));
            components.push(PrefabComponentRaw { kind: "transform3d".to_string(), fields: tf });
        }
    } else if node.kind == "TEXT" {
        let mut fields = Vec::new();
        fields.push(join_field("kind", "text"));
        if let Some(text) = node.text.clone() {
            if text.contains(":") {
                warnings.push(subst1("label in '{0}' contains ':' - the field parser splits on it, text gets cut", node.name.clone()));
            }
            fields.push(join_field("text", &text));
        }
        if node.font_size > 0.0 {
            fields.push(join_field("font_size", &numstr(node.font_size)));
        }
        if !node.fill_hex.is_empty() {
            fields.push(join_field("color", &node.fill_hex));
        }
        components.push(PrefabComponentRaw { kind: "uinode".to_string(), fields: fields });
        let mut tf = Vec::new();
        tf.push(join_field("position", &format_tuple(&[node.tx, node.ty, order as f32])));
        components.push(PrefabComponentRaw { kind: "transform3d".to_string(), fields: tf });
        if node.roll_deg.abs() > 0.001 {
            warnings.push(subst1("'{0}' is rotated - not imported yet: Figma turns about the corner, ui_overlay about the centre", node.name.clone()));
        }
    } else if node.image_fill {
        let mut fields = Vec::new();
        fields.push(join_field("kind", "image"));
        if node.width > 0.0 {
            fields.push(join_field("width", &numstr(node.width)));
        }
        if node.height > 0.0 {
            fields.push(join_field("height", &numstr(node.height)));
        }
        match textures.get(&node.id) {
            Some(uid) => fields.push(join_field("image", &uid.to_string())),
            None => warnings.push(subst1("image node '{0}' paints as a flat quad: its image was not exported as a texture", node.name.clone())),
        }
        components.push(PrefabComponentRaw { kind: "uinode".to_string(), fields: fields });
        let mut tf = Vec::new();
        tf.push(join_field("position", &format_tuple(&[node.tx, node.ty, order as f32])));
        components.push(PrefabComponentRaw { kind: "transform3d".to_string(), fields: tf });
    } else if !node.fill_hex.is_empty() {
        let mut fields = Vec::new();
        fields.push(join_field("kind", "block"));
        if node.width > 0.0 {
            fields.push(join_field("width", &numstr(node.width)));
        }
        if node.height > 0.0 {
            fields.push(join_field("height", &numstr(node.height)));
        }
        fields.push(join_field("color", &node.fill_hex));
        components.push(PrefabComponentRaw { kind: "uinode".to_string(), fields: fields });
        let mut tf = Vec::new();
        tf.push(join_field("position", &format_tuple(&[node.tx, node.ty, order as f32])));
        components.push(PrefabComponentRaw { kind: "transform3d".to_string(), fields: tf });
        if node.roll_deg.abs() > 0.001 {
            warnings.push(subst1("'{0}' is rotated - not imported yet: Figma turns about the corner, ui_overlay about the centre", node.name.clone()));
        }
    } else if node.children.is_empty() {
        warnings.push(subst2("'{0}' ({1}) got no ui facet - bake it in Figma or extend the importer", node.name.clone(), node.kind.clone()));
    }

    let mut children_out = Vec::new();
    let mut child_order = 0;
    for child in node.children {
        children_out.push(convert(child, child_order, textures, warnings));
        child_order += 1;
    }

    let comp_base = if node.base.is_none() { None } else { Some(subst1("{0}.comp", slugify(&node.base.unwrap()))) };

    PrefabGameObjectRaw {
        enabled: node.visible,
        name: node.name,
        base: comp_base,
        components: components,
        children: children_out,
    }
}

/// Depth-first search for a pinned `node-id` ("1:23") anywhere in the tree.
pub fn find_node(nodes: &[FigmaNode], id: &str) -> Option<FigmaNode> {
    for node in nodes {
        if node.id == id {
            return Some(node.clone());
        }
        if let Some(found) = find_node(&node.children, id) {
            return Some(found);
        }
    }
    None
}

pub fn collect_image_nodes(nodes: &[FigmaNode], out: &mut Vec<FigmaNode>) {
    for node in nodes {
        if node.image_fill && node.visible {
            out.push(node.clone());
        }
        collect_image_nodes(&node.children, out);
    }
}

/// INSTANCE nodes carry the component's *id*; the document's "components" map
/// resolves id -> name so the emitted `base:` reads like a prefab filename.
pub fn resolve_instance_bases(nodes: &mut Vec<FigmaNode>, meta: Option<&Value>) {
    for node in nodes {
        if let Some(id) = node.base.clone() {
            if let Some(map) = meta {
                if let Some(entry) = member(map, &id) {
                    if let Some(name) = get_opt_str(entry, "name") {
                        node.base = Some(name);
                    }
                }
            }
        }
        resolve_instance_bases(&mut node.children, meta);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// HTTP (curl subprocess, same `Command`-spawn pattern as `state.rs`'s cargo
// spawn) + full run pipeline - background thread ONLY (blocking)
// ─────────────────────────────────────────────────────────────────────────────

/// Digits of the first `HTTP/<ver> <code>` line of a curl `-i` response
/// file - this toolchain's curl has no `--write-out-to`, so the status
/// travels in the header block itself. "000" = no header block at all.
fn header_status(bytes: &Vec<u8>) -> String {
    let mut limit = bytes.len();
    if limit > 120 {
        limit = 120;
    }
    let mut i = 0;
    while i + 4 < limit {
        if bytes[i] == 72 && bytes[i + 1] == 84 && bytes[i + 2] == 84 && bytes[i + 3] == 80 && bytes[i + 4] == 47 {
            let mut j = i + 5;
            while j < limit && bytes[j] != 32 {
                j += 1;
            }
            j += 1;
            let mut code = String::new();
            while j < limit && code.len() < 3 && bytes[j] >= 48 && bytes[j] <= 57 {
                code.push(bytes[j] as char);
                j += 1;
            }
            if code.len() == 3 {
                return code;
            }
            return "000".to_string();
        }
        i += 1;
    }
    "000".to_string()
}

/// Where a curl `-i` body starts: just past the header block terminator
/// (CRLFCRLF, else LFLF). None means nothing to strip.
fn after_headers(bytes: &Vec<u8>) -> Option<usize> {
    let mut limit = bytes.len();
    if limit > 4096 {
        limit = 4096;
    }
    if bytes.len() < 4 {
        return None;
    }
    let mut i = 0;
    while i + 3 < limit {
        if bytes[i] == 13 && bytes[i + 1] == 10 && bytes[i + 2] == 13 && bytes[i + 3] == 10 {
            return Some(i + 4);
        }
        i += 1;
    }
    i = 0;
    while i + 1 < limit {
        if bytes[i] == 10 && bytes[i + 1] == 10 {
            return Some(i + 2);
        }
        i += 1;
    }
    None
}

/// Printable-ASCII glimpse of a body (control chars skipped) - enough of
/// Figma's JSON error without dumping the lot.
fn byte_snip(bytes: &Vec<u8>, max: usize) -> String {
    let mut out = String::new();
    for b in bytes.iter() {
        if out.len() >= max {
            break;
        }
        if *b >= 32 && *b < 127 {
            out.push(*b as char);
        }
    }
    out
}

fn file_snip(path: String, max: usize) -> String {
    match read_file_bytes(&path) {
        Err(_e) => String::new(),
        Ok(bytes) => byte_snip(&bytes, max),
    }
}

/// Value of response header `key` (lowercase ASCII) inside a curl `-i`
/// header block, matched case-insensitively at line starts. Figma's 429
/// carries Retry-After / X-Figma-Plan-Tier / X-Figma-Rate-Limit-Type /
/// X-Figma-Upgrade-Link, which tell the user whether to wait a minute or
/// whether their seat simply has no budget. Empty when the header is absent.
fn header_value(bytes: &Vec<u8>, key: &String) -> String {
    let kbs = key.as_bytes();
    let mut limit = bytes.len();
    if limit > 1200 {
        limit = 1200;
    }
    let mut p = 0;
    while p + kbs.len() < limit {
        let at_start = p == 0 || bytes[p - 1] == 10;
        if at_start {
            let mut k = 0;
            let mut ok = true;
            while k < kbs.len() {
                let mut b = bytes[p + k];
                if b >= 65 && b <= 90 {
                    b += 32;
                }
                if b != kbs[k] {
                    ok = false;
                }
                k += 1;
            }
            if ok && bytes[p + kbs.len()] == 58 {
                let mut v = p + kbs.len() + 1;
                while v < limit && (bytes[v] == 32 || bytes[v] == 9) {
                    v += 1;
                }
                let mut out = String::new();
                while v < limit && bytes[v] != 13 && bytes[v] != 10 {
                    if bytes[v] >= 32 && bytes[v] < 127 {
                        out.push(bytes[v] as char);
                    }
                    v += 1;
                }
                return out;
            }
        }
        p += 1;
    }
    String::new()
}

/// GET `url`, body lands in `out_path` via curl's `-o` (piping stdout back
/// through the harness was fragile; files are the repo's own idiom -
/// `fs_ops.rs` shuffles everything through paths). No `-f`, and no
/// `-w/--write-out-to` sidecar either - this toolchain's curl rejects that
/// option outright, which used to masquerade as "offline?". Instead `-i`
/// prefixes the header block to `out_path` itself: the REAL status comes
/// from its first line ("HTTP/2 403") and is stripped off before Ok(), so
/// callers still see a pure body. `--stderr` keeps curl's own complaint
/// (unknown option, connect failure) for the no-response case.
fn curl_get(url: String, token: String, out_path: String) -> Result<(), String> {
    let err_path = subst1("{0}.curl_err", out_path.clone());
    let mut command = Command::new("curl");
    command
        .arg("-s")
        .arg("--max-time")
        .arg("30")
        .arg("-i")
        .arg("--stderr")
        .arg(err_path.clone())
        .arg("-o")
        .arg(out_path.clone());
    if !token.is_empty() {
        command.arg("-H").arg(subst1("X-Figma-Token: {0}", token));
    }
    command.arg(&url);

    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = std::fs::remove_file(err_path.clone());
            return Err(subst1("curl failed to start: {0}", e.to_string()));
        }
    };
    let status = match child.wait() {
        Ok(s) => s,
        Err(e) => {
            let _ = std::fs::remove_file(err_path.clone());
            return Err(subst1("curl failed: {0}", e.to_string()));
        }
    };

    let complaint = file_snip(err_path.clone(), 160);
    let _ = std::fs::remove_file(err_path);

    match read_file_bytes(&out_path) {
        Err(_e) => {
            // curl died before writing any response (offline, bad option)
            if complaint.is_empty() {
                return Err(subst1("could not reach {0} (offline?)", url));
            }
            Err(subst2("curl could not fetch {0}: {1}", url, complaint))
        }
        Ok(bytes) => {
            let http_code = header_status(&bytes);
            if status.success() && http_code == "200" {
                // hand the caller back a pure body: strip curl's header block
                match after_headers(&bytes) {
                    None => (),
                    Some(start) => match std::fs::write(out_path, bytes[start..].to_vec()) {
                        Err(e) => return Err(subst1("could not store response body: {0}", e.to_string())),
                        Ok(_w) => (),
                    },
                }
                return Ok(());
            }
            // failure: quote Figma's JSON body (header block stripped), then
            // drop the junk file
            let mut detail = byte_snip(&bytes, 220);
            match after_headers(&bytes) {
                None => (),
                Some(start) => {
                    let tail = bytes[start..].to_vec();
                    let body = byte_snip(&tail, 220);
                    if !body.is_empty() {
                        detail = body;
                    }
                }
            }
            let _ = std::fs::remove_file(out_path);
            if http_code == "000" {
                if complaint.is_empty() {
                    return Err(subst1("could not reach {0} (offline?)", url));
                }
                return Err(subst2("curl could not fetch {0}: {1}", url, complaint));
            }
            if http_code == "200" {
                // 200 headers but curl died mid-transfer (timeout/reset)
                return Err(subst1("curl died mid-transfer for {0} (http was 200)", url));
            }
            if http_code == "429" {
                // Figma's rate limiter tripped (per-seat/plan budget, not
                // a token problem): surface Retry-After + the diagnostic
                // headers the 429 carries, per Figma's own rate-limit docs.
                let retry = header_value(&bytes, &"retry-after".to_string());
                let tier = header_value(&bytes, &"x-figma-plan-tier".to_string());
                let ltype = header_value(&bytes, &"x-figma-rate-limit-type".to_string());
                let upgrade = header_value(&bytes, &"x-figma-upgrade-link".to_string());
                let mut why = "wait a minute and try again".to_string();
                if !retry.is_empty() {
                    why = subst1("Figma says retry after {0}s", retry);
                }
                if !tier.is_empty() {
                    why = subst2("{0}, file's plan tier {1}", why, tier);
                }
                if !ltype.is_empty() {
                    why = subst2("{0}, your seat's limit type {1}", why, ltype);
                }
                if !upgrade.is_empty() {
                    why = subst2("{0} (upgrade link: {1})", why, upgrade);
                }
                let mut msg = subst2("Figma rate-limited this request for {0}: {1}", url, why);
                if !detail.is_empty() {
                    msg = subst2("{0} {1}", msg, detail);
                }
                return Err(msg);
            }
            if detail.is_empty() {
                return Err(subst2("request for {0} returned http {1}", url, http_code));
            }
            let head = subst2("request for {0} returned http {1}:", url, http_code);
            Err(subst2("{0} {1}", head, detail))
        }
    }
}

/// The Images endpoint's render URLs are absolute and pre-signed - no token
/// header needed (they expire ~7 days out, plenty for one import).
fn curl_download(url: &str, out_path: &str) -> Result<(), String> {
    curl_get(url.to_string(), String::new(), out_path.to_string())
}

pub fn run_import(tx: Sender<FigmaEvent>, token: String, url: String, dir: String) {
    let Some(file_key) = extract_file_key(&url) else {
        let _ = tx.send(FigmaEvent::Error("Paste a Figma file URL (figma.com/design/key/Name)".to_string()));
        return;
    };
    let pinned = extract_node_id(&url);

    let doc_url = subst1("https://api.figma.com/v1/files/{0}", file_key.clone());
    let doc_path = subst1("{0}/.figma_document.json", dir.clone());
    if let Err(msg) = curl_get(doc_url, token.clone(), doc_path.clone()) {
        let _ = tx.send(FigmaEvent::Error(msg));
        return;
    }

    let doc = match read_json_file(doc_path.clone()) {
        Err(inner) => {
            let _ = tx.send(FigmaEvent::Error(subst1("could not read document JSON: {0}", inner)));
            return;
        }
        Ok(doc) => doc,
    };
    let _ = std::fs::remove_file(doc_path);

    run_import_with_document(tx, token, file_key, pinned, dir, &doc);
}

/// Read a whole file as serde_json `Value` (curl wrote it to disk).
fn read_json_file(path: String) -> Result<Value, String> {
    let bytes = read_file_bytes(&path)?;
    let text = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn run_import_with_document(tx: Sender<FigmaEvent>, token: String, file_key: String, pinned: Option<String>, dir: String, doc: &Value) {
    let Some(document) = member(doc, "document") else {
        let _ = tx.send(FigmaEvent::Error("Figma response has no 'document' subtree (empty file?)".to_string()));
        return;
    };

    // document -> CANVAS pages -> top-level frames. Each frame becomes its own
    // `.comp` unless the URL pins a subtree.
    let mut roots = Vec::new();
    if let Some(canvas) = member(document, "children") {
        if let Value::Array(pages) = canvas {
            for page in pages.iter() {
                if let Some(children) = member(page, "children") {
                    if let Value::Array(frames) = children {
                        for frame in frames.iter() {
                            roots.push(parse_node(frame));
                        }
                    }
                }
            }
        }
    }
    if let Some(id) = pinned {
        let Some(found) = find_node(&roots, &id) else {
            let _ = tx.send(FigmaEvent::Error("pinned node-id not found in document (try importing the whole file)".to_string()));
            return;
        };
        roots.clear();
        roots.push(found);
    }
    if roots.is_empty() {
        let _ = tx.send(FigmaEvent::Error("document has no top-level frames to import".to_string()));
        return;
    }

    let _ = tx.send(FigmaEvent::Progress(subst1("fetched document: {0} top-level frame(s)", roots.len().to_string())));

    // INSTANCE nodes' componentIds -> component names via the doc's map.
    resolve_instance_bases(&mut roots, member(doc, "components"));

    // Image fills first: the PNG export is one batched request.
    let mut image_nodes = Vec::new();
    collect_image_nodes(&roots, &mut image_nodes);
    let textures = write_images(tx.clone(), token, file_key, dir.clone(), &mut image_nodes);
    let written_pngs = textures.len();

    let mut warnings = Vec::new();
    let mut written_comps = 0;
    let mut root_order = 0;
    for root in roots {
        let comp = convert(root.clone(), root_order, &textures, &mut warnings);
        root_order += 1;
        match dump_raw(&comp) {
            Err(e) => warnings.push(subst2("yaml serialize failed for '{0}': {1}", comp.name, e)),
            Ok(yaml) => {
                let name = subst1("{0}.comp", slugify(&root.name));
                let path = resolve_conflict(&dir, &name);
                if let Err(e) = write_file_text(&path, &yaml) {
                    warnings.push(subst2("could not write {0}: {1}", path, e));
                    continue;
                }
                get_or_create_meta(&path);
                written_comps += 1;
            }
        }
    }

    let warn_count = warnings.len();
    for warning in warnings {
        let _ = tx.send(FigmaEvent::Progress(subst1("warning: {0}", warning)));
    }
    let summary = subst4("Imported {0} comp(s), {1} PNG(s) into {2}; {3} warning(s)", written_comps.to_string(), written_pngs.to_string(), dir, warn_count.to_string());
    let _ = tx.send(FigmaEvent::Done(summary));
}

/// One batched Images-endpoint request for every visible image fill, then one
/// texture PNG per node written next to the comps. Returns node id -> texture uid.
fn write_images(tx: Sender<FigmaEvent>, token: String, file_key: String, dir: String, nodes: &mut Vec<FigmaNode>) -> HashMap<String, i16> {
    let mut textures = HashMap::new();
    if nodes.is_empty() {
        return textures;
    }
    let mut id_parts = Vec::new();
    for node in &*nodes {
        id_parts.push(node.id.clone());
    }
    let ids = id_parts.join(",");
    let img_url = subst2("https://api.figma.com/v1/images/{0}?ids={1}&format=png&scale=1", file_key, ids);
    let json_path = subst1("{0}/.figma_images.json", dir.clone());
    if let Err(e) = curl_get(img_url, token, json_path.clone()) {
        let _ = tx.send(FigmaEvent::Progress(subst1("warning: image export request failed: {0}", e)));
        return textures;
    }

    let images_json = match read_json_file(json_path.clone()) {
        Err(e) => {
            let _ = tx.send(FigmaEvent::Progress(subst1("warning: could not read image export: {0}", e)));
            return textures;
        }
        Ok(json) => json,
    };
    let _ = std::fs::remove_file(json_path);

    let images = member(&images_json, "images");
    for node in &*nodes {
        let Some(map) = images else {
            let _ = tx.send(FigmaEvent::Progress(subst1("warning: no render urls returned for '{0}'", node.name.clone())));
            continue;
        };
        let Some(link) = member(map, &node.id) else {
            let _ = tx.send(FigmaEvent::Progress(subst1("warning: no render url for '{0}'", node.name.clone())));
            continue;
        };
        let Some(link_url) = (match link {
            Value::String(s) => Some(s.to_string()),
            _ => None,
        }) else {
            let _ = tx.send(FigmaEvent::Progress(subst1("warning: render url not a string for '{0}'", node.name.clone())));
            continue;
        };
        let name = subst1("{0}.png", slugify(&node.name));
        let path = resolve_conflict(&dir, &name);
        if let Err(e) = curl_download(&link_url, &path) {
            let _ = tx.send(FigmaEvent::Progress(subst2("warning: '{0}': {1}", node.name.clone(), e)));
            continue;
        }
        match import_texture(&path) {
            Ok((_, uid)) => {
                textures.insert(node.id.clone(), uid);
            }
            Err(e) => {
                let _ = tx.send(FigmaEvent::Progress(subst2("warning: '{0}': {1}", node.name.clone(), e)));
            }
        }
    }
    textures
}

/// `figma.com/design/KEY/Name` | `/file/KEY` | `/proto/KEY` | `/board/KEY`
/// -> the file key the REST endpoints want.
fn extract_file_key(url: &str) -> Option<String> {
    let Some(at) = url.find("figma.com/") else { return None };
    let rest = &url[at + 10..];
    for kind in ["design/", "file/", "proto/", "board/"].iter() {
        if let Some(off) = rest.find(kind) {
            let tail = &rest[off + kind.len()..];
            let key = tail.split('/').next().unwrap_or("");
            if !key.is_empty() {
                return Some(key.to_string());
            }
        }
    }
    None
}

/// `?node-id=1-23` -> "1:23" (URLs dash what the API wants colons for).
fn extract_node_id(url: &str) -> Option<String> {
    let Some(at) = url.find("node-id=") else { return None };
    let tail = &url[at + 8..];
    let val = tail.split('&').next().unwrap_or("");
    if val.is_empty() {
        return None;
    }
    Some(val.replace("-", ":"))
}

// Mapping regression net (plan stage 7): IR -> ui_overlay comp. One frame with
// auto-layout gap, one TEXT leaf, one image RECTANGLE - pins the `uinode`
// registry key, pixel (y-down) placement, and the image-node flat-quad warn.
#[test]
fn figma_import_emits_ui_overlay_panels_and_leaves() {
    let title = FigmaNode {
        id: "1:2".to_string(),
        name: "title".to_string(),
        kind: "TEXT".to_string(),
        visible: true,
        tx: 20.0,
        ty: 12.0,
        roll_deg: 0.0,
        width: 100.0,
        height: 16.0,
        layout_mode: String::new(),
        item_spacing: 0.0,
        fill_hex: "#ff0000".to_string(),
        font_size: 16.0,
        image_fill: false,
        text: Some("Buy now".to_string()),
        base: None,
        children: Vec::new(),
    };
    let art = FigmaNode {
        id: "1:3".to_string(),
        name: "icon".to_string(),
        kind: "RECTANGLE".to_string(),
        visible: true,
        tx: 40.0,
        ty: 20.0,
        roll_deg: 0.0,
        width: 64.0,
        height: 64.0,
        layout_mode: String::new(),
        item_spacing: 0.0,
        fill_hex: String::new(),
        font_size: 0.0,
        image_fill: true,
        text: None,
        base: None,
        children: Vec::new(),
    };
    let card = FigmaNode {
        id: "1:4".to_string(),
        name: "card".to_string(),
        kind: "FRAME".to_string(),
        visible: true,
        tx: 60.0,
        ty: 30.0,
        roll_deg: 0.0,
        width: 120.0,
        height: 40.0,
        layout_mode: "NONE".to_string(),
        item_spacing: 0.0,
        fill_hex: String::new(),
        font_size: 0.0,
        image_fill: false,
        text: None,
        base: None,
        children: Vec::new(),
    };
    let doc = FigmaNode {
        id: "1:1".to_string(),
        name: "shop".to_string(),
        kind: "FRAME".to_string(),
        visible: true,
        tx: 0.0,
        ty: 0.0,
        roll_deg: 0.0,
        width: 320.0,
        height: 240.0,
        layout_mode: "HORIZONTAL".to_string(),
        item_spacing: 8.0,
        fill_hex: String::new(),
        font_size: 0.0,
        image_fill: false,
        text: None,
        base: None,
        children: vec![title.clone(), art, card],
    };

    let mut warnings = Vec::new();
    let comp = convert(doc, 0, &HashMap::new(), &mut warnings);
    let yaml = dump_raw(&comp).unwrap();

    // panel: uinode key + authored box + auto-layout gap
    let mut panel = false;
    for c in comp.components {
        if c.kind == "uinode" {
            panel = c.fields.contains(&join_field("kind", "panel")) && c.fields.contains(&join_field("width", &numstr(320.0))) && c.fields.contains(&join_field("gap", &numstr(8.0)));
        }
    }
    assert!(panel);

    // text leaf: kind text + label + px size + colour, y-down pixel offset
    let mut text_ok = false;
    for c in comp.children.first().unwrap().components.iter() {
        if c.kind == "uinode" {
            text_ok = c.fields.contains(&join_field("kind", "text")) && c.fields.contains(&join_field("text", "Buy now")) && c.fields.contains(&join_field("font_size", &numstr(16.0))) && c.fields.contains(&join_field("color", "#ff0000"));
        }
    }
    assert!(text_ok);
    assert!(yaml.contains("position: (20.0,12.0,0.0)"));
    assert!(!yaml.contains("(20.0,-12.0"));
    assert!(yaml.contains("transform3d") && !yaml.contains("transform2d") && !yaml.contains("z_index"));

    // image leaf: kind image + a flat-quad warning (no texture sampling yet)
    let mut image_ok = false;
    for c in comp.children.get(1).unwrap().components.iter() {
        if c.kind == "uinode" {
            image_ok = c.fields.contains(&join_field("kind", "image"));
        }
    }
    assert!(image_ok);
    let warned = warnings.iter().any(|w| w.contains("flat quad"));
    assert!(warned);

    // free-form nested frame: absolute flag + Figma pixel offset kept
    let mut card_ok = false;
    for c in comp.children.get(2).unwrap().components.iter() {
        if c.kind == "uinode" {
            card_ok = c.fields.contains(&join_field("kind", "panel")) && c.fields.contains(&join_field("absolute", "true"));
        }
    }
    assert!(card_ok);
    assert!(yaml.contains("position: (60.0,30.0,0.0)"));

    // the whole doc speaks the ui_overlay registry key, not the old 3D facets
    assert!(yaml.contains("uinode"));
    assert!(!yaml.contains("renderer_text") && !yaml.contains("rendererimage"));
}

#[test]
fn curl_get_reports_http_status_and_diagnostics() {
    // a curl -i response, exactly as -o stores it: header block + body
    let blob = "HTTP/2 429 \r\nRetry-After: 60\r\nX-Figma-Plan-Tier: starter\r\nX-Figma-Rate-Limit-Type: low\r\nContent-Type: application/json; charset=utf-8\r\n\r\n{\"errors\":[\"Too Many Requests\"],\"rationale\":\"\"}".to_string();
    let bytes = blob.as_bytes().to_vec();
    assert_eq!(header_status(&bytes), "429".to_string());
    assert_eq!(header_value(&bytes, &"retry-after".to_string()), "60".to_string());
    assert_eq!(header_value(&bytes, &"x-figma-plan-tier".to_string()), "starter".to_string());
    assert_eq!(header_value(&bytes, &"x-figma-rate-limit-type".to_string()), "low".to_string());
    assert_eq!(header_value(&bytes, &"x-figma-upgrade-link".to_string()), String::new());
    match after_headers(&bytes) {
        None => assert!(false),
        Some(start) => {
            let tail = bytes[start..].to_vec();
            assert_eq!(byte_snip(&tail, 64), "{\"errors\":[\"Too Many Requests\"],\"rationale\":\"\"}");
        }
    }
    // 200s strip the block too, so callers still see a pure body
    let ok = "HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\n{}".to_string();
    let ok_bytes = ok.as_bytes().to_vec();
    assert_eq!(header_status(&ok_bytes), "200".to_string());
    match after_headers(&ok_bytes) {
        None => assert!(false),
        Some(start) => {
            let tail = ok_bytes[start..].to_vec();
            assert_eq!(byte_snip(&tail, 8), "{}");
        }
    }
}
