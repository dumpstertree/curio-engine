use std::{
    env, fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

/// A static helper that generates the OS specific locations and has
/// convient helpers for reading/writing to disk
pub struct File {}
impl File {
    /// Join two paths into one path.
    /// Removes double slashses and other common problems
    /// Returns empty String if there was an error encountered
    pub fn join_path(path_a: &str, path_b: &str) -> String {
        // 1. Trim whitespace
        let mut a = path_a.trim().to_string();
        let mut b = path_b.trim().to_string();

        // Ensure `a` ends with '/'
        if !a.ends_with('/') {
            a.push('/');
        }

        // Ensure `b` does NOT start with '/'
        if b.starts_with('/') {
            b = b.trim_start_matches('/').to_string();
        }

        // Join
        let result_path_buf = PathBuf::from(&a).join(&b);

        let Some(result_path_str) = result_path_buf.to_str() else {
            // Failed
            eprintln!("Failed to convert path to str");
            return String::new();
        };

        // Success
        return result_path_str.to_string();
    }
    /// Get a path that contains the location of assets built into the application
    pub fn get_built_in_asset_path() -> String {
        return String::from("assets/");
    }

    /// Get a path that contains the location of save data - OS specific
    pub fn get_save_path() -> String {
        // get home dir
        let Some(home_path_buf) = env::home_dir() else {
            panic!();
        };

        // match for all supported os versions
        let os = std::env::consts::OS;
        if os == "linux" {
            // append path to home
            let path_buf = home_path_buf.join(".local/share/curio/save/");

            // conver to string
            let Some(path_str) = path_buf.to_str() else {
                panic!();
            };

            // return path
            return String::from(path_str);
        } else if os == "windows" {
            return String::from("");
        } else if os == "mac" {
            return String::from("");
        } else {
            eprintln!("Unsupported OS tag '{}' returning default value", os);
            return String::from("");
        }
    }

    /// Get a path that contains the location of cache data - OS specific
    pub fn get_cache_path() -> String {
        // get home dir
        let Some(home_path_buf) = env::home_dir() else {
            panic!();
        };

        // match for all supported os versions
        let os = std::env::consts::OS;
        if os == "linux" {
            // append path to home
            let path_buf = home_path_buf.join(".local/share/curio/cache/");

            // conver to string
            let Some(path_str) = path_buf.to_str() else {
                panic!();
            };

            // return path
            return String::from(path_str);
        } else if os == "windows" {
            return String::from("");
        } else if os == "mac" {
            return String::from("");
        } else {
            eprintln!("Unsupported OS tag '{}' returning default value", os);
            return String::from("");
        }
    }

    /// Reads the entire file into a `Vec<u8>`.
    /// Returns an empty Vec if reading fails.
    pub fn read(path: &str) -> Vec<u8> {
        fs::read(path).unwrap_or_else(|_| vec![])
    }

    /// Writes the given bytes to the file at `path`.
    /// Automatically creates parent directories if they don't exist.
    /// Returns true if successful, false otherwise.
    pub fn write(path: &str, data: &[u8]) -> bool {
        let path_ref = Path::new(path);

        // If the file has a parent directory, create it if needed
        if let Some(parent) = path_ref.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("Failed to create parent directories: {}", e);
                    return false;
                }
            }
        }

        fs::write(path_ref, data).is_ok()
    }

    /// Deletes the file at `path`.
    /// Returns true if successful, false otherwise.
    pub fn delete(path: &str) -> bool {
        fs::remove_file(path).is_ok()
    }

    /// Checks if a file exists at `path`.
    pub fn file_exists(path: &str) -> bool {
        Path::new(path).is_file()
    }

    /// Checks if a folder exists at `path`.
    pub fn folder_exists(path: &str) -> bool {
        Path::new(path).is_dir()
    }

    /// Checks the date created for the file at `path`.
    /// If file or metadata does not exist returns None
    pub fn get_meta_created(path: &str) -> Option<SystemTime> {
        if let Ok(meta) = fs::metadata(path) {
            if let Ok(created) = meta.created() {
                return Some(created);
            }
        }

        return None;
    }
    /// Checks the date accessed for the file at `path`.
    /// If file or metadata does not exist returns None
    pub fn get_meta_accessed(path: &str) -> Option<SystemTime> {
        if let Ok(meta) = fs::metadata(path) {
            if let Ok(accessed) = meta.accessed() {
                return Some(accessed);
            }
        }

        return None;
    }
    /// Checks the date modified for the file at `path`.
    /// If file or metadata does not exist returns None
    pub fn get_meta_modified(path: &str) -> Option<SystemTime> {
        if let Ok(meta) = fs::metadata(path) {
            if let Ok(modified) = meta.modified() {
                return Some(modified);
            }
        }

        return None;
    }
}
