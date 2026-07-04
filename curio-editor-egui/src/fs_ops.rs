//! Direct port of the file-system, meta-file, manifest, and facet sections of
//! `commands.rs`. These were `#[tauri::command]`s that just wrapped
//! `std::fs` calls — called directly here, no change in behavior.
//!
//! One real change: the original hardcoded
//! `/home/dumpstertree/Git/Rust/system_test` as `PROJECT_ROOT`/`ASSETS_ROOT`
//! in `rebuild_manifest`/`get_facets`, and `paths.ts` had the same string as
//! a fallback — clearly a developer's local path, not something meant to
//! ship. Everything here takes `project_root: &str` instead and the caller
//! (`EditorState`) passes `self.project_path`, which was already being
//! loaded from `test.proj` correctly everywhere else.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaFile {
    pub id: i16,
    pub included: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub id: i16,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub uri: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetManifest {
    #[serde(default)]
    pub manifest: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum EntryType {
    Asset(String),
    Float,
    Int,
    Bool,
    Vector2,
    Vector3,
    Vector4,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FacetField {
    pub name: String,
    pub data: EntryType,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FacetComponent {
    pub name: String,
    pub data: Vec<FacetField>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FacetManifest {
    #[serde(default)]
    pub manifest: Vec<FacetComponent>,
}

pub fn assets_root(project_root: &str) -> String {
    format!("{project_root}/assets")
}

fn is_meta(name: &str) -> bool {
    name.ends_with(".meta")
}

pub fn file_ext(name: &str) -> String {
    match name.rfind('.') {
        Some(i) => name[i..].to_lowercase(),
        None => String::new(),
    }
}

pub const SUPPORTED_EXTS: &[&str] = &[".png", ".glb", ".anim", ".comp"];

// ─────────────────────────────────────────────────────────────────────────────
// Directory listing — .meta files hidden, dirs first then alpha, same as
// AssetFileTree.tsx's `entries.filter(e => !isMeta(e.name))` + Rust-side sort
// ─────────────────────────────────────────────────────────────────────────────

pub fn list_dir(path: &str) -> Result<Vec<DirEntry>, String> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(path).map_err(|e| e.to_string())?.flatten() {
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if is_meta(&name) {
            continue;
        }
        let full_path = entry.path().to_string_lossy().to_string();
        entries.push(DirEntry { name, path: full_path, is_dir: metadata.is_dir() });
    }
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    Ok(entries)
}

// ─────────────────────────────────────────────────────────────────────────────
// Basic file ops
// ─────────────────────────────────────────────────────────────────────────────

pub fn create_folder(path: &str) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| e.to_string())
}

pub fn read_file_bytes(path: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| e.to_string())
}

pub fn write_file_text(path: &str, contents: &str) -> Result<(), String> {
    std::fs::write(path, contents).map_err(|e| e.to_string())
}

pub fn copy_file(src: &str, dst: &str) -> Result<(), String> {
    std::fs::copy(src, dst).map(|_| ()).map_err(|e| e.to_string())
}

pub fn create_comp_file(path: &str) -> Result<(), String> {
    const CONTENTS: &str = "enabled: true\nname: \"New GameObject\"\ncomponents: []\nchildren: []\n";
    std::fs::write(path, CONTENTS).map_err(|e| e.to_string())
}

pub fn delete_path(path: &str) -> Result<(), String> {
    let p = Path::new(path);
    if p.is_dir() {
        std::fs::remove_dir_all(p).map_err(|e| e.to_string())
    } else {
        std::fs::remove_file(p).map_err(|e| e.to_string())
    }
}

pub fn rename_path(old_path: &str, new_path: &str) -> Result<(), String> {
    std::fs::rename(old_path, new_path).map_err(|e| e.to_string())
}

pub fn move_path(src: &str, dst: &str) -> Result<(), String> {
    if std::fs::rename(src, dst).is_ok() {
        return Ok(());
    }
    let sp = Path::new(src);
    if sp.is_dir() {
        copy_dir_all(sp, Path::new(dst))?;
        std::fs::remove_dir_all(sp).map_err(|e| e.to_string())
    } else {
        std::fs::copy(src, dst).map_err(|e| e.to_string())?;
        std::fs::remove_file(src).map_err(|e| e.to_string())
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name())).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Tries `dir/name`, then `dir/name_1`, `dir/name_2`, ... until a path that
/// doesn't exist (as file or dir) is found. Port of `resolveConflict` from
/// `AssetFileTree.tsx`.
pub fn resolve_conflict(dir: &str, name: &str) -> String {
    let ext = file_ext(name);
    let base = if ext.is_empty() { name.to_string() } else { name[..name.len() - ext.len()].to_string() };
    let mut i = 1;
    let mut candidate = format!("{dir}/{name}");
    while Path::new(&candidate).exists() {
        candidate = format!("{dir}/{base}_{i}{ext}");
        i += 1;
    }
    candidate
}

// ─────────────────────────────────────────────────────────────────────────────
// Meta files — YAML sidecar per asset (`foo.png.meta`)
// ─────────────────────────────────────────────────────────────────────────────

pub fn read_meta(asset_path: &str) -> Option<MetaFile> {
    let text = std::fs::read_to_string(format!("{asset_path}.meta")).ok()?;
    serde_yaml::from_str(&text).ok()
}

pub fn write_meta(asset_path: &str, meta: &MetaFile) -> Result<(), String> {
    let yaml = serde_yaml::to_string(meta).map_err(|e| e.to_string())?;
    std::fs::write(format!("{asset_path}.meta"), yaml).map_err(|e| e.to_string())
}

/// Port of `getOrCreateMeta` — reads the sidecar if present, otherwise mints
/// a random id (1..=32767, matching the original's `Math.floor(Math.random()
/// * 32767) + 1`) and writes a fresh one.
pub fn get_or_create_meta(asset_path: &str) -> MetaFile {
    if let Some(existing) = read_meta(asset_path) {
        return existing;
    }
    let id = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(1) % 32767 + 1) as i16;
    let meta = MetaFile { id, included: true };
    let _ = write_meta(asset_path, &meta);
    meta
}

// ─────────────────────────────────────────────────────────────────────────────
// Asset manifest — rebuild walks assets/, reads each .meta, writes
// asset.manifest; read just parses it back
// ─────────────────────────────────────────────────────────────────────────────

pub fn rebuild_manifest(project_root: &str) -> Result<(), String> {
    let assets_root = assets_root(project_root);

    fn collect(dir: &Path, root: &Path, entries: &mut Vec<ManifestEntry>) -> Result<(), String> {
        for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.ends_with(".meta") {
                continue;
            }
            if path.is_dir() {
                collect(&path, root, entries)?;
                continue;
            }

            let meta_path = PathBuf::from(format!("{}.meta", path.to_string_lossy()));
            let Ok(meta_text) = std::fs::read_to_string(&meta_path) else { continue };
            let Ok(meta) = serde_yaml::from_str::<MetaFile>(&meta_text) else { continue };
            if !meta.included {
                continue;
            }

            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(&name).to_string();
            let uri = path.strip_prefix(root).map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| name.clone());

            entries.push(ManifestEntry { id: meta.id, name: stem, kind: "Embedded".to_string(), uri });
        }
        Ok(())
    }

    let mut entries = Vec::new();
    collect(Path::new(&assets_root), Path::new(project_root), &mut entries)?;

    let mut yaml = String::from("manifest:\n");
    for e in &entries {
        yaml.push_str(&format!("  - id: {}\n    name: \"{}\"\n    type: {}\n    uri: \"{}\"\n", e.id, e.name, e.kind, e.uri));
    }

    std::fs::write(format!("{project_root}/asset.manifest"), yaml).map_err(|e| e.to_string())
}

pub fn read_manifest(project_root: &str) -> Vec<ManifestEntry> {
    let Ok(text) = std::fs::read_to_string(format!("{project_root}/asset.manifest")) else { return Vec::new() };
    serde_yaml::from_str::<AssetManifest>(&text).map(|m| m.manifest).unwrap_or_default()
}

// ─────────────────────────────────────────────────────────────────────────────
// Facets
// ─────────────────────────────────────────────────────────────────────────────

pub fn get_facets(project_root: &str) -> FacetManifest {
    let Ok(bytes) = std::fs::read(format!("{project_root}/facet.manifest")) else { return FacetManifest::default() };
    serde_yaml::from_slice(&bytes).unwrap_or_default()
}

// ─────────────────────────────────────────────────────────────────────────────
// File picker — native dialog, replaces the Tauri dialog plugin used by
// `api.pickFile`
// ─────────────────────────────────────────────────────────────────────────────

pub fn pick_file() -> Option<String> {
    rfd::FileDialog::new().pick_file().map(|p| p.to_string_lossy().to_string())
}
