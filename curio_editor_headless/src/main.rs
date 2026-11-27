// Headless Curio runner for LLM-driven testing — runs a compiled game
// plugin from a YAML scenario without ever opening a window, dumps the
// ledger over time, and renders frames to PNG on demand (needs an adapter,
// software ones like llvmpipe work — no window manager required).

mod scenario;
mod sim;

use sim::Simulation;

fn main() {
    // same panic banner contract as the engine/editor — LLM tooling greps
    // for this to find where a game plugin died
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

    // scenario path from argv, default `scenario.yaml` in the working dir
    let path = std::env::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "scenario.yaml".to_string());

    // structured exit codes for LLM tooling: 0 done · 2 scenario · 3 project ·
    // 4 no plugin · 5 plugin died mid-run (the stdout lines carry the detail)
    let code = Simulation::new(path).run();
    std::process::exit(code);
}
