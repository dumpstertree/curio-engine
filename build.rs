use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;

fn main() -> Result<()> {
    let _ = add_env_path("/res", "OUT_DIR");
    let _ = add_env_path("/assets/mesh", "ASSET_MESH");
    // let _ = add_env_path("/assets/texture", "ASSET_TEXTURE");

    unsafe {
        env::set_var("ASSET_TEXTURE", "/assets/texture");
    }

    Ok(())
}
fn add_env_path(path: &str, key: &str) -> Result<()> {
    // This tells Cargo to rerun this script if something in /res/ changes.
    println!("cargo:rerun-if-changed=res/*");

    let out_dir = env::var(key)?;
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();
    paths_to_copy.push(path);
    copy_items(&paths_to_copy, out_dir, &copy_options)?;

    Ok(())
}
