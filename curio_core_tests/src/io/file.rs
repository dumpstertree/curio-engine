#[cfg(test)]
mod tests {
    use curio_core::io::file::File;
    use tempfile::tempdir;

    #[test]
    fn join_path_joins_paths() {
        let joined = File::join_path("assets", "image.png");
        assert!(joined.ends_with("assets/image.png"));
    }

    #[test]
    fn join_path_handles_extra_slashes() {
        let joined = File::join_path("assets/", "/image.png");
        assert!(joined.ends_with("assets/image.png"));
    }

    #[test]
    fn join_path_trims_whitespace() {
        let joined = File::join_path(" assets ", " image.png ");
        assert!(joined.ends_with("assets/image.png"));
    }

    #[test]
    fn write_read_exists_delete_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.bin");
        let path = path.to_str().unwrap();

        let data = vec![1, 2, 3, 4, 5];

        assert!(File::write(path, &data));
        assert!(File::file_exists(path));

        let read = File::read(path);
        assert_eq!(read, data);

        assert!(File::delete(path));
        assert!(!File::file_exists(path));
    }

    #[test]
    fn write_creates_parent_directories() {
        let dir = tempdir().unwrap();

        let nested = dir.path().join("a").join("b").join("c").join("file.txt");

        let path = nested.to_str().unwrap();

        assert!(File::write(path, b"hello"));

        assert!(File::folder_exists(dir.path().join("a/b/c").to_str().unwrap()));

        assert_eq!(File::read(path), b"hello");
    }

    #[test]
    fn read_missing_file_returns_empty_vec() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing.txt");

        let bytes = File::read(missing.to_str().unwrap());

        assert!(bytes.is_empty());
    }

    #[test]
    fn delete_missing_file_returns_false() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing.txt");

        assert!(!File::delete(missing.to_str().unwrap()));
    }

    #[test]
    fn folder_exists_returns_true_for_directory() {
        let dir = tempdir().unwrap();

        assert!(File::folder_exists(dir.path().to_str().unwrap()));
    }

    #[test]
    fn metadata_exists_after_write() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("meta.txt");
        let path = path.to_str().unwrap();

        assert!(File::write(path, b"metadata"));

        assert!(File::get_meta_modified(path).is_some());

        // These may not be available on every platform/filesystem,
        // so don't require them.
        let _ = File::get_meta_created(path);
        let _ = File::get_meta_accessed(path);
    }

    #[test]
    fn metadata_missing_file_returns_none() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.txt");
        let path = path.to_str().unwrap();

        assert_eq!(File::get_meta_created(path), None);
        assert_eq!(File::get_meta_accessed(path), None);
        assert_eq!(File::get_meta_modified(path), None);
    }

    #[test]
    fn built_in_asset_path_is_not_empty() {
        assert_eq!(File::get_built_in_asset_path(), "assets/");
    }

    #[test]
    fn save_and_cache_paths_are_consistent() {
        let save = File::get_save_path();
        let cache = File::get_cache_path();

        match std::env::consts::OS {
            "linux" => {
                assert!(save.ends_with("/.local/share/curio/save/"));
                assert!(cache.ends_with("/.local/share/curio/cache/"));
            }
            _ => {
                // Current implementation returns empty strings on non-Linux.
                assert!(save.is_empty());
                assert!(cache.is_empty());
            }
        }
    }
}
