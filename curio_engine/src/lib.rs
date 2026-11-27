// Library surface of the runner — everything main() does is call into this,
// and the sibling test crate (curio_engine_tests) exercises it directly.

pub mod curio_cabinet;
pub mod plugin_loader;
pub mod project;
