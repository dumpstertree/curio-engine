#[cfg(test)]
mod tests {
    use curio_engine::project::Project;
    use std::fs;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn project_default_is_empty() {
        let project = Project::default();
        assert!(project.name.is_empty());
        assert!(project.project_path.is_empty());
        assert!(project.build_args.is_empty());
    }

    // load_local always reads the one fixed path ./test.proj relative to cwd,
    // and cargo test runs with this package directory as cwd — so the whole
    // contract lives in one sequential test that owns the file's lifecycle.
    #[test]
    fn project_contract_through_cwd_file() {
        // 1. no test.proj present — load_local must panic with the
        //    "No project found" message, contained here so the test survives.
        let survived = catch_unwind(AssertUnwindSafe(|| Project::load_local()));
        assert!(survived.is_err());

        // 2. a full project file parses every field
        let yaml = "name: Runner Test\nproject_path: ./fixtures/game\nbuild_args:\n  - --flag\n";
        assert!(fs::write("./test.proj", yaml.as_bytes()).is_ok());
        let project = Project::load_local();
        assert_eq!(project.name, "Runner Test");
        assert_eq!(project.project_path, "./fixtures/game");
        assert_eq!(project.build_args, vec!["--flag"]);

        // 3. build_args is optional — #[serde(default)]
        let bare = "name: Bare Project\nproject_path: ./bare\n";
        assert!(fs::write("./test.proj", bare.as_bytes()).is_ok());
        let bare_project = Project::load_local();
        assert_eq!(bare_project.name, "Bare Project");
        assert!(bare_project.build_args.is_empty());

        // cleanup — leave no test.proj behind
        assert!(fs::remove_file("./test.proj").is_ok());
    }
}
