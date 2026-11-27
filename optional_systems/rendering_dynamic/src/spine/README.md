# spine — Spine runtime (adopted from rusty_spine)

This module is a vendored copy of [rusty_spine](https://github.com/jabuwu/rusty_spine)
0.3.1, taken from the `rusty_spine3.8` checkout at commit `4ff8b72`. That crate is a
Rust wrapper around the official Spine C runtime, transpiled with c2rust (see
`c/mod.rs` for the spine-c and c2rust commits).

It is licensed under the Spine Runtimes License in `LICENSE` next to this file. That
license and its copyright notice must stay with any redistribution.

## Changes from upstream

- It is a module of `rendering_dynamic` (`rendering_dynamic::spine`), not a crate.
  `crate::` paths were rewritten to `crate::spine::`.
- Features: `draw_functions` (`controller`, `draw`) and `mint` are always on.
  `egui_debugger` (egui 0.19) and the `use_libc` allocator switch were dropped.
  spine-c always uses the pure-Rust allocator in `c/wasm.rs`.
- Migrated to edition 2024 (`cargo fix --edition`): `unsafe extern`, `#[unsafe(no_mangle)]`
  and explicit `unsafe {}` blocks in unsafe fns. `static mut` reads go through raw
  pointers (`&raw const` / `&raw mut`), not references.
- Clippy cleanups: elided lifetimes made explicit; `f32::consts::PI` replaces the
  `3.1415927` literal (it is the same f32); c2rust's const-cast `&field as *const T as *mut T`
  writes now derive from `&raw mut`; `_spRandom` no longer transmutes an
  `extern "C"` fn pointer to a Rust-ABI one before calling it.
- Upstream tests that need the sample assets were left out. The allocator/string tests in
  `c/wasm.rs` are kept, minus `snprintf`, which only compiled with the dropped libc path.
- Doc examples are marked `ignore`: they load upstream sample assets that were not
  vendored, so they are kept as documentation only. Paths in them read
  `rendering_dynamic::spine::`.

## Host callbacks

spine-c calls `_spUtil_readFile`, `_spAtlasPage_createTexture` and
`_spAtlasPage_disposeTexture` as global C symbols, and `extension.rs` defines them. Do
not define these symbols anywhere else in the workspace; that duplicate-symbol link error
is why this module was adopted. To change the behaviour, register a callback with
`extension::set_read_file_cb`, `set_create_texture_cb` or `set_dispose_texture_cb`.
Memory handed back to spine must come from spine's allocator; the extension copies into it
for you.
