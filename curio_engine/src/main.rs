// Prevents an additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use curio_engine::curio_cabinet::CurioCabinet;
use curio_engine::project::Project;

fn main() {
    // A game plugin panicking while running used to abort with nothing but a
    // one-line message lost in wgpu noise — print a banner with the real
    // location + message so crashes are findable (same as the editor).
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!("\n================ CURIO PANIC ================");
        if let Some(loc) = info.location() {
            eprintln!("at {}:{}", loc.file(), loc.line());
        }
        let msg = info
            .payload()
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| info.payload().downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "panic with no message".to_string());
        eprintln!("{msg}");
        eprintln!("(set RUST_BACKTRACE=1 for a backtrace)");
        eprintln!("=============================================\n");
        default_hook(info);
    }));

    // Load ./test.proj relative to the working directory — same convention
    // as the editor's main.rs.
    let project = Project::load_local();

    // Open directly into the application — its own window, no editor UI.
    CurioCabinet::put_on_display(project);
}
