//! Direct access to the [Spine C Runtime](http://en.esotericsoftware.com/spine-c).
//!
//! Transpiled from spine-c commit
//! [a8a7ba878aacf2109be07ef4dffcf34643f9547b](https://github.com/EsotericSoftware/spine-runtimes/tree/a8a7ba878aacf2109be07ef4dffcf34643f9547b)
//!
//! Transpiled using c2rust commit [0a2b64b4f83b42f08fe13c3d4fbd8b5b167f07a8](https://github.com/immunant/c2rust/tree/0a2b64b4f83b42f08fe13c3d4fbd8b5b167f07a8)
#[macro_use]
mod wasm;

mod spine_c;

pub use self::spine_c::*;

mod environment {
    pub use super::wasm::*;
}
