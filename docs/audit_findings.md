# Curio Engine Audit Findings — tracking list

Audit scope: `curio_core` crate (80 `.rs` files read, Nov 2026). Companion crate
`curio_core_tests` is used to pin behavior with tests (groups 1-7 plan below).
Status flags: `[open]` needs a decision/fix · `[fixed <commit>]` done and pushed ·
`[by design]` intentional, kept as documentation · `[untestable]` cannot be exercised
from outside the crate · `[unverifiable]` no Rust toolchain in audit env, user must build.

---

## A. Bugs — open (tests document them, no fix applied yet)

1. **`Ledger::read::<T>()` panics for unregistered records** — `[by design]`, flag only
   - `ledger.rs` ~L113-129 via `get_entry`/`get_entry_mut` `panic!` incl. guid, type_name, id.
   - Panic-based API kept; test asserts it (`unregistered_record_read_panics`). Only "resolve"
     if the API is ever moved to `Option`.

---

## B. Quirks — intentional / documented, probably NOT bugs

- **Scope asymmetry:** `RecordNetworkCapabilities::has_write_privilege(Instance) == true`
  (instance-owned records are *always* written), but `ImpulseNetworkCapabilities::has_write_privilege(Instance) == false`
  (instance-owned impulses never need sending). Both matrices fully tested
  (`record_network_capabilities.rs`, `impulse_network_capabilities.rs`). Keep as doc.
- `Ledger::write` with insufficient privilege: logs a `Warning` and *silently drops* the write
  (`ledger.rs` ~L144-151). By design; covered by `peer_write_on_host_record_is_rejected`.
- `Nerve::enqueue_event` pushes to the local cache *before* the privilege check
  (`if !has_write_privilege { return; }`, nerve.rs ~L80-90) — event stays local, no sync. Tested
  (`peer_privilege_caches_event_but_drops_sync`).
- `Ledger`/`Nerve` entries are a **sparse vec indexed by global record id**
  (`entries[id as usize]`): high sequential UIDs waste memory. Not fixed; note for a future
  HashMap-based ledger.
- Empty trait-default bodies and marker structs (`File`, `Assets`, `Random`, `Logger`,
  `CurioCabinet`) judged intentional idiom, not gaps.
- `to_remove/` dir and `[unused]` assets treated as dead code, excluded from audit.
- `FieldState::new` does `serde_json::to_value(value).unwrap()` (`plugin_group_state.rs` ~L45-51)
  — panics for non-serializable field values; low risk, left alone.

---

## C. Environment / test-infrastructure constraints (needed when resuming)

- **No cargo/rustc in the audit environment** — everything reviewed only; user builds. `[unverifiable]`
- **Run engine tests with `RUST_TEST_THREADS=1` / `--test-threads=1`:** the global `Logger`
  (unsafe `Send`/`Sync`, `io/logger.rs` L88-89) is mutated through the `Services` singleton
  (`AtomicPtr` + assert-if-unset, `engine/services.rs`) by all tests; registries
  `GlobalRecords`/`GlobalImpulses` are process-global. Parallel cargo test races on the Logger.
- **Services shim:** `curio_core_tests/src/engine/services.rs :: ensure_services()` heap-leaks a
  `Services { logger, assets(AssetPipeline::new(AssetCache::new(10), AssetDatabase::new())), gpu(nulls) }`
  once via `LazyLock`; every engine test that can reach a logging path calls it first. The
  pre-init assert itself is order-dependent → deliberately untestable (comment in file).
- **serde must be pinned identically** in `curio_core_tests/Cargo.toml`: `serde = { version = "1.0.104", features = ["derive"] }`
  so `Serialize`/`DeserializeOwned` trait identities unify with `GlobalRecords::register_serializable` bounds.
- **Registry ids kept unique across test files** (registration is a global `HashMap<i32, …>`):
  | id | type | scope | file |
  |---|---|---|---|
  | Record 9501 | EntryProbe | (default) | ledger_entry.rs |
  | Record 9001 | LedgerProbeLocal (default v=7) | Instance | ledger.rs |
  | Record 9002 | LedgerProbeHost (default v=1) | Host | ledger.rs |
  | Record 9003 | RecordSyncProbe | Host | record_synchronizer.rs |
  | Record 9004 | RecordSyncNoSerde — `register()` w/o serde fns | Instance | record_synchronizer.rs |
  | Record 9005 | CapProbe | Instance | record_network_capabilities.rs |
  | Record 9990/9992 | Unregistered probes — never registered | — | ledger.rs / record_synchronizer.rs |
  | Impulse 9101 | NerveProbe | All | nerve.rs |
  | Impulse 9102 | NerveProbeLocal | Instance | nerve.rs |
  | Impulse 9103 | CapImpulse | All | impulse_network_capabilities.rs |
  | Impulse 9201 | SyncProbe | ConnectedHost | impulse_synchronizer.rs |
  | Impulse 9991 | SyncProbeUnregistered — never registered | — | impulse_synchronizer.rs |
- **`Curio::new` is `pub(crate)`** (`engine/curio.rs` ~L28): a real `Curio` cannot be constructed
  from `curio_core_tests`, so `update()`/`serializable()`/window hooks/plugin-order tests are
  **not** reachable — skipped in group 1. If Curio-level tests are ever wanted: export a test
  constructor or move those tests inside `curio_core`'s own `#[cfg(test)]`.
- Test-crate conventions mirrored: one file per module, `#[cfg(test)] mod tests {}`,
  `pub mod engine { pub mod <file>; }` in `curio_core_tests/src/lib.rs`.

---

## D. Already fixed and pushed (context for later passes)

- `[fixed ae59b43]` `Curio::update` command handlers (`Fullscreen`/`Resizable`/`Cursor` now set
  the new `pub portal: Portal` field) + window hooks `window_focused`/`window_occluded`
  implemented + debug `println!` removals + real power-of-2 check (`nerve.rs`, `texture_asset.rs`).
- `[fixed 51ea929]` Built-in shaders: standalone `assets/built_in/shader_module/vegetation.wgsl`
  + `particle.wgsl` written and wired into `shaders.rs` (replaced `panic!()` `LazyLock`s).
  Left untouched: the old multi-file fragment pair `shader_vegetation.wgsl` + `helper_wind.wgsl`
  (still WIP; drop-in compatible with the mesh pipeline is the new modules).
- `[fixed 56eb8c2]` **bug-fixes branch** (branched off origin/main tip `0cd5dcc`), audit items
  A1/A2/A4/A5 + G1/G2, all re-pinned by `curio_core_tests` (suite: 220 tests, 0 failed):
  - A2: `Nerve::enqueue_event_delayed` now stores the requested delay per entry
    (`(Instant, f32, i32, Box, sync)` tuple) and `update_timed_events` compares `elapsed() > delay`.
    Test renamed `delayed_event_ignores_requested_delay` → `delayed_event_honors_requested_delay`.
  - A1: `Ledger::new_custom` seeds a placeholder `CurioNetworkParticipant(0, LocalHost)`
    instead of an empty network, so `log`/`serializable` have a valid `me()`. Test renamed
    `new_custom_panics_on_empty_network` → `new_custom_constructs_with_placeholder_network`
    (no longer `#[should_panic]`).
  - A5: `RecordSynchronizer::payload` and `ImpulseSynchronizer::payload` are now `pub`
    (roundtrippable anyway), making the deserialize-failure paths externally constructible.
  - A4: `DeserializerFn` (record + impulse aliases) now returns `Option<Box<…>>`;
    global registrations map a bincode `Err` to `None` instead of `unwrap()`-panicking,
    and both synchronizers propagate that `None`. New tests:
    `corrupt_payload_deserializes_to_none` (record + impulse),
    `unregistered_id_deserializes_to_none`, `registration_without_deserialize_fn_deserializes_to_none`.
    Kept as-is: the serialize-side `downcast`/`unwrap` (programming-error path, the caller owns
    the typed value — not network-reachable).
  - G1: `pub use crate::collections::any_map::GetError;` added at the crate root;
    `any_map.rs :: get_result_separates_ok_from_err` now pins `Err(GetError::EmptyKey)` vs
    `Err(GetError::MismatchedType)` via `matches!`.
  - G2: `EventRunner::post_and_drain` walks the queue with a head cursor and clears at the end
    (was O(n²) via `remove(0)`; linear now, same FIFO semantics).
- `[fixed 56eb8c2]` decision, not a code change: A3 `CurioNetwork::me()` kept panic-based
  and documented ("callers must supply a valid seat"); the `empty_network_has_no_me` /
  `me_index_out_of_bounds_panics` tests still pin the panic. A1's fix removes the only
  engine path that built such an invalid network.
- H3 (`InputMapped::get_button_or_default` doc says "returns NONE"): verified already corrected
  in the pushed tree (both `_or_default` doc comments read "returns a default version");
  nothing to fix.
- API-affecting changes from fixes: `Curio::portal` is now a public field (only breaking surface);
  synchronizer `payload` fields are now public; `GetError` gains a root re-export.

## E. Test-plan groups (1-3 committed in `560a4a6`, 4-7 pending)

1. `[done, uncommitted]` engine state & sync layer — 34 tests across 9 files
   (ledger, ledger_entry, nerve, record/impulse synchronizers, both capabilities, curio_network,
   services shim).
2. `[done, uncommitted]` collections — 19 tests across 3 files (`any_queue`, `event_runner`,
   `any_map`); findings moved to section G.
3. `[done, uncommitted]` input — 33 tests across 7 files (`button_state`, `axis_state`,
   `button_pressed`, `button_code`, `axis_code`, `input_raw`, `input_mapped`); findings in section H.
4. static data / identity / UIDs — `sequential_record_uids`, `sequential_impulse_uids`,
   `identity` (note: `Identity::new` calls `Random::guid(6)`; invalid() = instance −1).
5. metadata & commands serde — `portal`, `version`, `commands`, `formation`, `seat`,
   `graphics_mapping`, `input_mapping`, `identity`.
6. math — `matrix4x4`, `frustrum`, `color`, `random`, `vector2_int`/`vector3_int`
   (vectors/quaternion already covered).
7. io — `asset_database_listing`, `cache_metadata`, `asset_pipeline`
   (`asset_cache`, `asset_database`, `file` already covered).
- GPU/host-bound modules (`gpu.rs` raw-pointer struct, winit/window side) excluded throughout.

## G. Group 2 findings — collections (`any_queue`, `event_runner`, `any_map`)

1. **`EventRunner` has no cycle guard** — `[open]` (design)
   - A receiver that always returns >= 1 event makes `post_and_drain` loop forever
     (`while queue.len() > 0` walk). Cannot be tested without hanging the binary.
   - Fix direction: optional event budget / re-entrance depth limit in `post_and_drain`.
     (The old O(n²) `remove(0)` issue behind it was fixed in `56eb8c2`, see D.)

2. **Pinned semantics (by design, no fix):**
   - every receiver sees every event, receivers run in registration order, events FIFO
     (`event_runner.rs :: every_receiver_gets_every_event_in_registration_order`,
     `generated_events_are_processed_fifo_until_exhausted`);
   - `enqueue` clones the event (`enqueue_clones_the_event`);
   - `AnyMap::get` conflates missing-key with type-mismatch → `None`
     (`any_map.rs :: get_returns_none_for_both_missing_key_and_type_mismatch`);
   - `AnyMap` has no remove/clear API — entries can be overwritten but never deleted
     (comment in `any_map.rs`).

3. **`AnyQueue`: no bugs found** — all 7 tests pass-as-spec (typed drain keeps other
   types, FIFO order, `get_all` views without draining, `push_boxed` interops with `drain<T>`).

---

## H. Group 3 findings — input state machines (`button_*`, `axis_*`, `input_raw`, `input_mapped`)

1. **No functional bugs found in group 3** — `ButtonState::update` / `AxisState::update`
   transition math is correct (flags computed from the OLD state), and `InputRaw`/`InputMapped`
   containers, defaults and `new()` seeding all pass-as-spec (33 tests).

2. **`InputRaw` vs `InputMapped` disagree on absent codes** — `[by design]`, pinned
   - `InputRaw::update` iterates the *poll* map: codes absent from a poll keep their previous
     state (`input_raw.rs :: codes_absent_from_a_poll_keep_their_last_state`).
   - `InputMapped::update` iterates the *mapping* and forces absent raw codes to
     `false`/`Vector2::zero()` (`input_mapped.rs :: mapped_uid_forces_absent_codes_to_released`,
     `absent_axis_is_forced_to_zero` — note the forced zero shows up as a large delta).
   - Consumer code must know which container it is polling; only `InputMapped` is safe for
     "current logical control" reads.

3. **Enum order is a wire-format dependency** — `[by design]`, partially pinned
   - `ButtonCode`/`AxisCode` are `#[non_exhaustive]` serde-derived enums: their *declaration
     order* is the serialized tag. Inserting/reordering variants silently breaks saved
     `InputMapping`s and network payloads. Order is pinned only for the codes the tests use
     (`declaration_order_is_stable` in both enum test files), not the whole enum.

4. **`button_code.rs` winit mappings are untested** — `[untestable]` cheaply
   - `from_winit_mousebutton` / `from_winit_physical_key` / `from_winit_keycode`
     (L399-592, ~570 `=>` arms) take winit event types; testing them requires adding winit to
     `curio_core_tests` at the *exact* same version (type identity). Coverage there is manual
     review only: all arms map `KeyCode::X => ButtonCode::X` one-to-one with a `_ => None`
     fallback; no arm duplicates a KeyCode, but full 1:1 coverage is unverified.

5. **Pinned semantics (by design, no fix):**
   - `ButtonPressed` defaults to `Down` (= resting); a resting button reads `Down`
     (`button_pressed.rs :: default_is_down`);
   - `ButtonState`/`AxisState` are `Copy`: polling a stored copy never moves the original, so
     the System Component must keep one owner (`update_is_by_copy_not_reference` ×2);
   - unknown raw codes read a shared default view, never a stale entry
     (`codes_never_seen_read_default_not_stale`, `default_fallback_stays_untouched_while_other_codes_poll`);
   - unmapped raw codes can never press a mapped uid (`unmapped_raw_codes_are_ignored`);
   - `InputMapped::new` seeds one fresh state per mapped uid, and duplicate uids in a mapping
     collapse to one HashMap entry updated by every code that references it (untested edge
     case — aliasing a uid to two codes makes the *last mapping entry* win each poll).

---

## F. Git state snapshot

- `origin/main` tip: `0cd5dcc` (PR #1 `feature/taffy-ui-overlay` merged, includes groups 1-3
  test commit `560a4a6`, findings sync `9440d35`, `d143bc7` test typo fixes, and the taffy
  ui_overlay work `d7781ee`..`d4f0a0a`). Local `main` was left at the stale `560a4a6`.
- **`bug-fixes` branch**: branched from `0cd5dcc`; `56eb8c2` = audit bug fixes (A1/A2/A4/A5,
  G1/G2, A3 decision, H3 verification) + re-pinned tests, 17 files. Findings-file update
  committed immediately after on the same branch.
- Previous tips: `560a4a6` = **groups 1-3 test files + this findings file + `lib.rs`/`Cargo.toml`**
  (22 files, 2192 insertions), `51ea929` shaders, `ae59b43` window cmds/hooks + command-handler
  fixes. Author `dumpstertree <dumpstertree@gmail.com>`, remote `git@github.com:dumpstertree/curio-engine.git`.
- Tests still need a real build (`RUST_TEST_THREADS=1`); failures land here as `[open]` items
  rather than reverting the commit.
