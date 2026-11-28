//! Penpot RPC API -> Curio composition import, beside the Figma one (`crate::figma`).
//! Shapes are parsed into the same `FigmaNode` tree, so `figma::convert` writes the comps and
//! both importers produce the same ui_overlay output. Runs on a job thread like the Figma import.

pub mod http;
pub mod parse;
pub mod parse_ctx;
pub mod penpot_state;

pub use penpot_state::PenpotState;

use crate::figma::{convert, slugify, FigmaEvent};
use crate::fs_ops::{get_or_create_meta, import_texture, read_file_bytes, resolve_conflict, write_file_text};
use crate::penpot::parse::{field, parse_page, str_of};
use crate::penpot::parse_ctx::ParseCtx;
use crate::prefab_resolver::dump_raw;

use serde_json::Value;

use std::collections::HashMap;
use std::sync::mpsc::Sender;

const DEFAULT_HOST: &str = "https://design.penpot.app";

pub fn run_import(tx: Sender<FigmaEvent>, token: String, url: String, dir: String) {
    let Some(file_id) = extract_file_id(&url) else {
        let _ = tx.send(FigmaEvent::Error("Paste a Penpot workspace URL (it contains file-id=...) or a file id".to_string()));
        return;
    };
    let host = extract_host(&url);
    let page_id = query_param(&url, "page-id");

    // Without the `fdata/*` client features the backend inlines pointer and objects maps itself.
    let file_url = format!("{host}/api/rpc/command/get-file?id={file_id}");
    let doc_path = format!("{dir}/.penpot_file.json");
    if let Err(msg) = http::get(&file_url, &token, "application/json", &doc_path) {
        let _ = tx.send(FigmaEvent::Error(msg));
        return;
    }
    let doc = read_json_file(&doc_path);
    let _ = std::fs::remove_file(&doc_path);
    let doc = match doc {
        Ok(doc) => doc,
        Err(e) => {
            let _ = tx.send(FigmaEvent::Error(format!("could not read file JSON: {e}")));
            return;
        }
    };
    run_import_with_file(tx, token, host, file_id, page_id, dir, &doc);
}

fn read_json_file(path: &str) -> Result<Value, String> {
    let bytes = read_file_bytes(path)?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}

fn run_import_with_file(tx: Sender<FigmaEvent>, token: String, host: String, file_id: String, page_id: Option<String>, dir: String, doc: &Value) {
    let Some(data) = field(doc, "data") else {
        let keys: Vec<String> = doc
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        let _ = tx.send(FigmaEvent::Error(format!("Penpot response has no 'data' (top-level keys: {})", keys.join(", "))));
        return;
    };
    let Some(index) = field(data, "pages-index") else {
        let _ = tx.send(FigmaEvent::Error("Penpot file data has no pages index".to_string()));
        return;
    };

    // Page order comes from `pages`; a page-id in the URL narrows the import to that page.
    let mut page_ids: Vec<String> = match field(data, "pages") {
        Some(Value::Array(ids)) => ids
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => index
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default(),
    };
    if let Some(pinned) = page_id {
        if index.get(&pinned).is_none() {
            let _ = tx.send(FigmaEvent::Error("page-id from the URL is not in this file (drop it to import every page)".to_string()));
            return;
        }
        page_ids = vec![pinned];
    }

    let no_objects = Value::Null;
    let mut ctx = ParseCtx {
        objects: &no_objects,
        components: field(data, "components"),
        file_id: &file_id,
        images: Vec::new(),
        warnings: Vec::new(),
    };
    let mut roots = Vec::new();
    for id in &page_ids {
        let Some(objects) = index.get(id).and_then(|page| field(page, "objects")) else { continue };
        ctx.objects = objects;
        roots.extend(parse_page(&mut ctx));
    }
    if roots.is_empty() {
        let _ = tx.send(FigmaEvent::Error("no top-level shapes to import on the selected page(s)".to_string()));
        return;
    }
    let file_name = str_of(doc, "name").unwrap_or_default();
    let _ = tx.send(FigmaEvent::Progress(format!("fetched '{file_name}': {} top-level shape(s) on {} page(s)", roots.len(), page_ids.len())));

    let textures = write_images(&tx, &token, &host, &dir, &ctx.images);
    let written_images = textures.len();

    let mut warnings = ctx.warnings;
    let mut written_comps = 0;
    for (order, root) in roots.into_iter().enumerate() {
        let file = format!("{}.comp", slugify(&root.name));
        let comp = convert(root, order as i32, &textures, &mut warnings);
        match dump_raw(&comp) {
            Err(e) => warnings.push(format!("yaml serialize failed for '{}': {e}", comp.name)),
            Ok(yaml) => {
                let path = resolve_conflict(&dir, &file);
                if let Err(e) = write_file_text(&path, &yaml) {
                    warnings.push(format!("could not write {path}: {e}"));
                    continue;
                }
                get_or_create_meta(&path);
                written_comps += 1;
            }
        }
    }

    let warn_count = warnings.len();
    for warning in warnings {
        let _ = tx.send(FigmaEvent::Progress(format!("warning: {warning}")));
    }
    let _ = tx.send(FigmaEvent::Done(format!("Imported {written_comps} comp(s), {written_images} image(s) into {dir}; {warn_count} warning(s)")));
}

/// Downloads each referenced media object once as a texture, named after the first shape using it.
/// These are the uploaded source images, not renders of the shapes. Returns shape id -> texture uid.
fn write_images(tx: &Sender<FigmaEvent>, token: &str, host: &str, dir: &str, images: &[(String, String, String, String)]) -> HashMap<String, i16> {
    // media id -> its uid, None once it failed, so a shared image is fetched and reported once
    let mut media: HashMap<String, Option<i16>> = HashMap::new();
    let mut textures = HashMap::new();
    for (shape_id, name, media_id, mtype) in images {
        if let Some(done) = media.get(media_id) {
            if let Some(uid) = done {
                textures.insert(shape_id.clone(), *uid);
            }
            continue;
        }
        media.insert(media_id.clone(), None);
        let path = resolve_conflict(dir, &format!("{}.{}", slugify(name), extension(mtype)));
        let url = format!("{host}/assets/by-file-media-id/{media_id}");
        if let Err(e) = http::get(&url, token, "*/*", &path) {
            let _ = tx.send(FigmaEvent::Progress(format!("warning: image for '{name}': {e}")));
            continue;
        }
        match import_texture(&path) {
            Ok((_, uid)) => {
                media.insert(media_id.clone(), Some(uid));
                textures.insert(shape_id.clone(), uid);
            }
            Err(e) => {
                let _ = tx.send(FigmaEvent::Progress(format!("warning: image for '{name}': {e}")));
            }
        }
    }
    textures
}

fn extension(mtype: &str) -> String {
    match mtype {
        "image/jpeg" => "jpg".to_string(),
        "image/svg+xml" => "svg".to_string(),
        "" => "png".to_string(),
        other => other
            .rsplit('/')
            .next()
            .unwrap_or("png")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect(),
    }
}

/// `https://host/#/workspace?...` -> `https://host`; no scheme means the hosted design.penpot.app.
pub fn extract_host(url: &str) -> String {
    let Some(scheme_end) = url.find("://") else { return DEFAULT_HOST.to_string() };
    let rest = &url[scheme_end + 3..];
    let host_len = rest.find(['/', '#', '?']).unwrap_or(rest.len());
    if host_len == 0 {
        return DEFAULT_HOST.to_string();
    }
    url[..scheme_end + 3 + host_len].to_string()
}

/// The file uuid: `file-id=` (current query routes), else the last uuid after `workspace/`
/// (older `#/workspace/<project>/<file>` routes), else the input itself when it is a bare uuid.
pub fn extract_file_id(url: &str) -> Option<String> {
    if let Some(id) = query_param(url, "file-id") {
        return Some(id);
    }
    if let Some(at) = url.find("workspace/") {
        let path = url[at + 10..].split(['?', '#']).next().unwrap_or("");
        if let Some(id) = path.split('/').rfind(|part| is_uuid(part)) {
            return Some(id.to_string());
        }
    }
    let bare = url.trim();
    if is_uuid(bare) {
        return Some(bare.to_string());
    }
    None
}

/// Value of `key=` anywhere in the URL (Penpot puts its route params after the `#`), if a uuid.
pub fn query_param(url: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=");
    let mut search = url;
    while let Some(at) = search.find(&needle) {
        let before = search[..at].chars().last();
        let tail = &search[at + needle.len()..];
        if before.is_none() || before == Some('?') || before == Some('&') {
            let value = tail.split(['&', '#', '/']).next().unwrap_or("");
            if is_uuid(value) {
                return Some(value.to_string());
            }
        }
        search = tail;
    }
    None
}

fn is_uuid(s: &str) -> bool {
    s.len() == 36
        && s.chars()
            .enumerate()
            .all(|(i, c)| if i == 8 || i == 13 || i == 18 || i == 23 { c == '-' } else { c.is_ascii_hexdigit() })
}
