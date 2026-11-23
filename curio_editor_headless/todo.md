# curio_core changes needed to make headless testing fully controllable

This crate is the zero-`curio_core`-change cut: it drives the same
`load_curio` → `window_opened()` → `update()`/`render()` contract the
standalone engine (`curio_engine`) uses, forwards scenario input through
`Curio::input_button`/`Curio::input_axis`, dumps `Ledger::serializable()`
as JSON diffs, and reads frames back with the editor's `capture.rs`
pattern. Everything works today — but a few simulation levers still leak
out of the runner's reach. These are the `curio_core` changes that would
close them, roughly in order of how much they matter to LLM game testing.

## 1. Generic (name-based) ledger field writes

`curio_core/src/engine/ledger.rs`

`Ledger::write::<TRecord>` matches on `TRecord::id()` **plus** the
TypeIdentity of `TRecord`. Compiled into the runner that is fine, but a
game that came from a dlopen'd `.so` carries the plugin crate's own copy
of `SysRecordTime` (or any record type) with a different `TypeId` — the
downcast inside `write` panics. `sim.rs::inject_time` has to guard the
whole write with `catch_unwind` and report "time injection skipped".

Wanted: a dyn path that does not need compile-time type identity:

```rust
// LedgerEntry already keeps `RecordOverride` dyn objects around
Ledger::set_field(&mut self, record_name: &str, field: &str, value_json: serde_json::Value) -> bool
// via RecordOverride trait addition:
//   fn set_state(&mut self, json: serde_json::Value)
```

`serializable()` already round-trips fields as `serde_json::Value`
(`FieldState.data`), so the read half exists — only the write half is
missing. With that, scenario files could seed ANY record (time, player
position, health…) at startup, which is the single biggest unlock for
deterministic LLM test scenarios.

## 2. Nerve delayed events on a virtual clock

`curio_core/src/engine/nerve.rs`

`enqueue_event_delayed` stamps `Instant::now()` (wall clock) and
`update_timed_events` fires anything whose `.elapsed() > 1.0` — the
`_delay` parameter is currently ignored outright. Two consequences for
headless runs:

- delayed impulses depend on real wall time, not simulated frames —
  two runs of the same scenario drift apart;
- the fixed 1.0 s check makes every delayed event land one real second
  later regardless of what the scenario asked for.

Wanted: drive `delayed` off the simulation clock (frame count or a
`Nerve::tick(f32 dt)` parameter threaded from `Curio::update`) so
`enqueue_event_delayed(event, 0.25)` means "fire a quarter sim-second
later" reproducibly.

## 3. Time record injection (pairs with item 1)

`system_component_default/time/src/sys_record_time.rs`

`RecordOverride::set_state` for `SysRecordTime` is an empty impl, so
even with the dyn write path above there is nowhere for the JSON to go.
Populate it (write `target_frame_rate` etc. from the `serde_json::Value`)
and the runner's typed-write fallback becomes a clean, non-panicking set.

## 4. Logger sink buffer instead of direct stdout

`curio_core/src/io/logger.rs`

`Logger::log` prints straight out. The runner interleaves that with its
own `HEADLESS:`/`ledger:`/`shot:` protocol lines — an LLM parsing the
run output has to grep around it, and a game that logs heavily per frame
drowns the machine-readable lines.

Wanted: a `Logger::take_lines() -> Vec<String>` drain (or a configurable
sink) so the runner can emit structured
`{frame, logs: […]}` blocks instead of raw interleaving.

## 5. Seeded `Random`

`curio_core/src/math/random.rs`

`Random::guid` and friends pull from an unseeded global source, so two
headless runs of one scenario can pick different network ids/asset
cache keys/etc. A `Random::seed(u64)` call — set once at startup from
the scenario file — makes runs bit-reproducible, which is what a
regression-testing loop needs.

## Note: screenshots need an adapter, not a GPU

The pinned wgpu (24.0.5) has no CPU backend — `Instance::request_adapter`
with `compatible_surface: None` still needs *some* driver (llvmpipe
software-GL counts; no window manager or GPU hardware required). Where
none exists, this crate prints `HEADLESS: no adapter` and continues the
sim with screenshots skipped. If screenshot-less CI becomes common, a
future `curio_core` change could move the `Gpu` contract onto a
software-rasterizable fallback — out of scope for now.
