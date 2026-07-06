use std::{fs, path::Path};

use serde::Deserialize;

#[derive(Deserialize)]
struct Manifest {
    manifest: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    id: i32,
    r#type: EntryType,
    uri: String,
}

#[derive(Deserialize)]
enum EntryType {
    Local,
    Remote,
    Embedded,
}

fn main() {
    let manifest_path = Path::new("/home/dumpstertree/Git/Rust/system_test/asset.manifest");

    let yaml = fs::read_to_string(manifest_path).expect("failed to read asset.manifest");

    let manifest: Manifest = serde_yaml::from_str(&yaml).expect("failed to parse asset.manifest");

    let mut generated = String::new();

    generated.push_str(r#"use curio_core::AssetDatabaseListing;"#);
    generated.push_str(
        r#"
pub fn generated_assets() -> Vec<(String, i16, AssetDatabaseListing)> {
    vec![
"#,
    );

    for entry in manifest.manifest {
        match entry.r#type {
            EntryType::Embedded => {
                println!("cargo:rerun-if-changed={}", entry.uri);

                let path = entry.uri.replace('\\', "/");

                generated.push_str(&format!(
                    r#"
        (
            "{name}".to_string(),
            {id},
            AssetDatabaseListing::Embedded(
                include_bytes!("../{path}")
                .to_vec()
            ),
        ),
"#,
                    name = path,
                    id = entry.id,
                    path = path,
                ));
            }

            EntryType::Local => {
                generated.push_str(&format!(
                    r#"
        (
            "{name}".to_string(),
            {id},
            AssetDatabaseListing::Local(
                "{path}".to_string()
            ),
        ),
"#,
                    name = entry.uri,
                    id = entry.id,
                    path = entry.uri,
                ));
            }

            EntryType::Remote => {
                generated.push_str(&format!(
                    r#"
        (
            "{name}".to_string(),
            {id},
            AssetDatabaseListing::Remote(
                "{uri}".to_string()
            ),
        ),
"#,
                    name = entry.uri,
                    id = entry.id,
                    uri = entry.uri,
                ));
            }
        }
    }

    generated.push_str(
        r#"
    ]
}
"#,
    );

    fs::write("/home/dumpstertree/Git/Rust/system_test/src/generated_assets.rs", generated).expect("failed to write generated_assets.rs");
}
