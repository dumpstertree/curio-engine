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
//! Mapped so far (output loads through `spawn_prefab_recursive_internal` as-is):
//! - `FRAME`/`GROUP` -> nested comp nodes; each canvas child becomes its own
//!   `.comp` (or one file when the URL pins a `node-id` subtree)
//! - hidden layers (`visible:false`) -> `enabled: false` (kept, not dropped)
//! - image fills -> `rendererimage` facet + PNG exported through the Images
//!   endpoint and written next to the `.comp`
//! - `TEXT` -> `renderer_text` `contents` only (no font/style mapping yet)
//! - `INSTANCE` -> `base:` prefab reference named after the Figma component
//!
//! Parked on purpose (plan section 7b - needs new facet surface, not importer
//! logic): auto-layout as `UiNode` panels, solid fills, gradients, strokes,
//! image-fill crop modes, `RendererText` metrics, constraints. Those nodes
//! still get a transform, just no paint facet, and add a warning line.
//!
//! Coordinates: positions are parent-relative pixels (Figma's own `relativeTransform`)
//! with the y axis flipped, since the engine's y grows up. Calibrating the
//! whole import against `Transform2D::get_matrix`'s frustum remap is M3 - TODO.

use crate::fs_ops::{get_or_create_meta, read_file_bytes, resolve_conflict, write_file_text};
use crate::prefab_resolver::dump_raw;
use crate::prefab_types::{PrefabComponentRaw, PrefabGameObjectRaw, format_tuple, join_field};

use serde_json::{self, Value};

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
    pub job: Option < (String, Receiver < FigmaEvent > ) > ,
    pub progress: Vec < String > ,
    pub summary: Option < String > ,
    pub error: Option < String > ,
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
        let dir = self.target_dir.clone(); let job_dir = self.target_dir.clone();
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
    // absoluteBoundingBox size (kept for M3's bounds/crop work, unused now).
    pub width: f32,
    pub height: f32,
    pub image_fill: bool,
    pub text: Option < String > ,
    pub base: Option < String > , // INSTANCE -> Figma component name
    pub children: Vec < FigmaNode > ,
}

// serde_json member helpers. Destructuring style copied from
// `panels/inspector.rs`'s `json_to_value` - the repo's serde_json `Value`
// exposes Object/Array constructors; helpers return owned copies so the parse
// pass never fights the `Value` borrow checker.

fn member<'a>(v: &'a Value, key: &str) -> Option < &'a Value > {
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

fn get_opt_str(v: &Value, key: &str) -> Option < String > {
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
fn slugify(name: &str) -> String {
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

pub fn convert(node: FigmaNode, warnings: &mut Vec < String > ) -> PrefabGameObjectRaw {
    let mut components = Vec::new();

    // transform2d: parent-relative pixels with Figma's y flipped (engine y
    // grows up). Rotation only when nonzero - the runtime default is already
    // the identity quaternion.
    let mut fields = Vec::new();
    fields.push(join_field("position", &format_tuple(&[node.tx, -node.ty])));
    if node.roll_deg.abs() > 0.001 {
        fields.push(join_field("rotation", &format_tuple(&[0.0, 0.0, node.roll_deg])));
    }
    components.push(PrefabComponentRaw { kind: "transform2d".to_string(), fields: fields });

    if node.image_fill {
        let mut image_fields = Vec::new();
        image_fields.push(join_field("asset", &subst1("{0}.png", slugify(&node.name))));
        components.push(PrefabComponentRaw { kind: "rendererimage".to_string(), fields: image_fields });
    }
    if let Some(text) = node.text.clone() {
        let mut text_fields = Vec::new();
        text_fields.push(join_field("contents", &text));
        components.push(PrefabComponentRaw { kind: "renderer_text".to_string(), fields: text_fields });
    }

    let known_kind = node.kind == "FRAME" || node.kind == "GROUP" || node.kind == "CANVAS"
        || node.kind == "INSTANCE" || node.kind == "TEXT" || node.kind == "RECTANGLE"
        || node.kind == "VECTOR";
    if !known_kind && !node.image_fill && node.text.is_none() {
        warnings.push(subst2(
            "'{0}' ({1}) has no paint facet yet - bake it in Figma or extend the importer",
            node.name.clone(),
            node.kind.clone(),
        ));
    }

    let mut children_out = Vec::new();
    for child in node.children {
        children_out.push(convert(child, warnings));
    }

    let comp_base = if node.base.is_none() {
        None
    } else {
        Some(subst1("{0}.comp", slugify(&node.base.unwrap())))
    };

    PrefabGameObjectRaw {
        enabled: node.visible,
        name: node.name,
        base: comp_base,
        components: components,
        children: children_out,
    }
}

/// Depth-first search for a pinned `node-id` ("1:23") anywhere in the tree.
pub fn find_node(nodes: &[FigmaNode], id: &str) -> Option < FigmaNode > {
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

pub fn collect_image_nodes(nodes: &[FigmaNode], out: &mut Vec < FigmaNode > ) {
    for node in nodes {
        if node.image_fill && node.visible {
            out.push(node.clone());
        }
        collect_image_nodes(&node.children, out);
    }
}

/// INSTANCE nodes carry the component's *id*; the document's "components" map
/// resolves id -> name so the emitted `base:` reads like a prefab filename.
pub fn resolve_instance_bases(nodes: &mut Vec < FigmaNode > , meta: Option < &Value > ) {
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

/// GET `url`, body lands in `out_path` via curl's `-o` (piping stdout back
/// through the harness was fragile; files are the repo's own idiom -
/// `fs_ops.rs` shuffles everything through paths). `-f` makes curl exit
/// nonzero on 4xx/5xx so `wait()`/`success()` sees bad token/key/expiry.
fn curl_get(url: String, token: String, out_path: String) -> Result < (), String > {
    let mut command = Command::new("curl");
    command
        .arg("-s")
        .arg("-f")
        .arg("--max-time").arg("30")
        .arg("-o").arg(out_path);
    if !token.is_empty() {
        command.arg("-H").arg(subst1("X-Figma-Token: {0}", token));
    }
    command.arg(&url);

    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => return Err(subst1("curl failed to start: {0}", e.to_string())),
    };
    let status = match child.wait() {
        Ok(s) => s,
        Err(e) => return Err(subst1("curl failed: {0}", e.to_string())),
    };
    if !status.success() {
        return Err(subst1("request for {0} failed (offline? bad token 403? bad file key 404?)", url));
    }
    Ok(())
}

/// The Images endpoint's render URLs are absolute and pre-signed - no token
/// header needed (they expire ~7 days out, plenty for one import).
fn curl_download(url: &str, out_path: &str) -> Result < (), String > {
    curl_get(url.to_string(), String::new(), out_path.to_string())
}

pub fn run_import(tx: Sender < FigmaEvent > , token: String, url: String, dir: String) {
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
fn read_json_file(path: String) -> Result < Value, String > {
    let bytes = read_file_bytes(&path)?;
    let text = String::from_utf8(bytes).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn run_import_with_document(
    tx: Sender < FigmaEvent > ,
    token: String,
    file_key: String,
    pinned: Option < String > ,
    dir: String,
    doc: &Value,
) {
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

    let _ = tx.send(FigmaEvent::Progress(
        subst1("fetched document: {0} top-level frame(s)", roots.len().to_string())
    ));

    // INSTANCE nodes' componentIds -> component names via the doc's map.
    resolve_instance_bases(&mut roots, member(doc, "components"));

    // Image fills first: the PNG export is one batched request.
    let mut image_nodes = Vec::new();
    collect_image_nodes(&roots, &mut image_nodes);
    let written_pngs = write_images(tx.clone(), token, file_key, dir.clone(), &mut image_nodes);

    let mut warnings = Vec::new();
    let mut written_comps = 0;
    for root in roots {
        let comp = convert(root.clone(), &mut warnings);
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

    let warn_count = warnings.len(); for warning in warnings {
        let _ = tx.send(FigmaEvent::Progress(subst1("warning: {0}", warning)));
    }
    let summary = subst4(
        "Imported {0} comp(s), {1} PNG(s) into {2}; {3} warning(s)",
        written_comps.to_string(),
        written_pngs.to_string(),
        dir,
        warn_count.to_string(),
    );
    let _ = tx.send(FigmaEvent::Done(summary));
}

/// One batched Images-endpoint request for every visible image fill, then one
/// PNG per node written next to the comps (M3 will route these to
/// assets/texture/ + real name lookups). Returns how many PNGs landed.
fn write_images(
    tx: Sender < FigmaEvent > ,
    token: String,
    file_key: String,
    dir: String,
    nodes: &mut Vec < FigmaNode > ,
) -> usize {
    if nodes.is_empty() {
        return 0;
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
        return 0;
    }

    let images_json = match read_json_file(json_path.clone()) {
        Err(e) => {
            let _ = tx.send(FigmaEvent::Progress(subst1("warning: could not read image export: {0}", e)));
            return 0;
        }
        Ok(json) => json,
    };
    let _ = std::fs::remove_file(json_path);

    let images = member(&images_json, "images");
    let mut count = 0;
    for node in &*nodes {
        let Some(map) = images else {
            let _ = tx.send(FigmaEvent::Progress(
                subst1("warning: no render urls returned for '{0}'", node.name.clone())
            ));
            continue;
        };
        let Some(link) = member(map, &node.id) else {
            let _ = tx.send(FigmaEvent::Progress(
                subst1("warning: no render url for '{0}'", node.name.clone())
            ));
            continue;
        };
        let Some(link_url) = (match link {
            Value::String(s) => Some(s.to_string()),
            _ => None,
        }) else {
            let _ = tx.send(FigmaEvent::Progress(
                subst1("warning: render url not a string for '{0}'", node.name.clone())
            ));
            continue;
        };
        let name = subst1("{0}.png", slugify(&node.name));
        let path = resolve_conflict(&dir, &name);
        if let Err(e) = curl_download(&link_url, &path) {
            let _ = tx.send(FigmaEvent::Progress(
                subst2("warning: '{0}': {1}", node.name.clone(), e)
            ));
            continue;
        }
        get_or_create_meta(&path);
        count += 1;
    }
    count
}

/// `figma.com/design/KEY/Name` | `/file/KEY` | `/proto/KEY` | `/board/KEY`
/// -> the file key the REST endpoints want.
fn extract_file_key(url: &str) -> Option < String > {
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
fn extract_node_id(url: &str) -> Option < String > {
    let Some(at) = url.find("node-id=") else { return None };
    let tail = &url[at + 8..];
    let val = tail.split('&').next().unwrap_or("");
    if val.is_empty() {
        return None;
    }
    Some(val.replace("-", ":"))
}
