# curio_editor_headless — the headless runner

Same simulation contract as the standalone `curio_engine` runner
(`load_curio` → `window_opened()` → `update()`/`render()` per frame), but
with no winit window in front of it: it opens straight into a
scenario-driven batch run and writes everything out as files +
machine-readable stdout lines. An LLM (or CI) can drive a real game
plugin — read its ledger over time, feed it input, get screenshots —
without any user interaction and without a GPU (software adapters like
llvmpipe are enough; only a window manager is not needed).

Nothing here is a `curio_core` change: this crate reuses the
`curio_engine` library surface (`plugin_loader`, `project`,
`curio_cabinet`) and the public core API (`Curio::input_button`/
`Curio::input_axis`, `Ledger::serializable()`, the `Services`/`Gpu`
contract). See [todo.md](./todo.md) for the core changes that would
close remaining gaps (generic record injection, Nerve virtual clock,
Logger sink, seeded `Random`).

## Running it

```
cargo build -p curio_editor_headless
cd <dir containing ./test.proj>
./target/debug/curio_editor_headless scenario.yaml
```

- **`./test.proj`** — same project descriptor the engine and the egui
  editor read (name + project path). The game plugin is loaded from
  `{project_path}/target/release/` exactly as the engine does, with the
  plugin stem preferred (`lib{package_name_snake}` first) so shared
  build dirs pick the right `.so`.
- **scenario path** — first argv; defaults to `scenario.yaml` in the
  working directory.

`Cargo.toml` already has it as a workspace member
(`cargo run -p curio_editor_headless -- <scenario>` works too).

## Scenario format

One YAML file per experiment — the LLM writes it, the runner reads it:

```yaml
steps: 180                    # frames to simulate (required, nonzero)
dump_every: 60                # ledger dump every N frames (0/omitted = never)
dump_dir: out/ledger          # default "ledger"
capture_dir: out/shots        # default "shots"
capture_width: 640            # screenshot texture size (defaults 640x360)
capture_height: 360
capture_frames: [30, 60, 120] # frames to render + save as PNG
time_target_frame_rate: 30.0  # SysRecordTime startup injection (see todo.md
                              # for the dlopen-mode limitation)
events:
  - frame: 10
    button: MousePrimary      # any ButtonCode variant name
    pressed: Down             # Down or Up (default Down)
  - frame: 40
    button: MousePrimary
    pressed: Up
  - frame: 90
    axis: Cursor              # any AxisCode variant name
    x: 0.5
    y: -0.25
```

Input events are forwarded through exactly the same public core API the
engine's and editor's window handlers call (`Curio::input_button`,
`Curio::input_axis`), bucketed by frame and fired at the frame boundary
before `update()`.

## What a run produces

Everything is written relative to the working directory; parent
folders are auto-created (`File::write`).

```
out/ledger/ledger-000060.json   # full snapshot: ["mode-guid", PluginState tree]
out/ledger/diff-000060.json     # flattened (object path, field value) pairs
                                #   that differ from the previous dump
out/shots/shot-000030.png       # one screenshot per capture_frames entry
```

stdout is a flat line protocol, easy to grep:

```
ledger: out/ledger/ledger-000060.json
diff: out/ledger/diff-000060.json
shot: out/shots/shot-000030.png
HEADLESS DONE: steps=180
```

`out/ledger/…` snapshots come from `Ledger::serializable()` — the same
`PluginState`/`ObjectState`/`ComponentState`/`FieldState` tree the editor
inspects, with each field's `data` as JSON. The diff files are the
cheapest thing for a model to read: only the `(path, value)` pairs that
changed since the last dump.

Screenshots use the egui editor's proven `capture.rs` readback pattern
(offscreen `capture_texture`, ping-pong `MAP_READ` buffers,
`copy_texture_to_buffer`, `map_async` + `poll(Wait)`, 256-byte row
padding stripped). `Services.gpu.capture_texture` stays **null** on
contract — the game's swapchain→capture copy step self-skips and this
runner owns the readback.

A missing adapter (pinned wgpu 24.0.5 has no CPU backend) is not an
error: the run prints `HEADLESS: no adapter — screenshots will be
skipped` and the simulation still runs headless; `capture_frames`
entries become `shot skipped: N (no adapter)` lines.

## Crash contract

A game plugin that dies mid-run doesn't leave bare panic noise:

- the `================ CURIO PANIC ================` banner (file, line,
  message, same as engine/editor) prints as always;
- the frame loop catches it: `HALT: the game plugin panicked during
  frame N — see the CURIO PANIC banner`;
- the process exits with a structured code.

| exit | meaning |
|---|---|
| 0 | run completed |
| 2 | scenario missing / invalid / `steps: 0` |
| 3 | no `./test.proj` in the working directory |
| 4 | no game plugin could be loaded |
| 5 | plugin panicked mid-run |

## What it deliberately does not do

- **No window, no interactivity** — that's what `curio_engine`
  (`curio_engine/src/curio_cabinet.rs`) and the egui editor
  (`curio_editor/`) are for. This is a one-shot batch run.
- **No generic startup injection yet** — `time_target_frame_rate` is a
  typed `Ledger::write::<SysRecordTime>`, which can only write when the
  record's type identity matches (compiled-in game). Against a
  dlopen'd `.so` the downcast panics; the runner guards it and prints
  `HEADLESS: time injection skipped … (see todo.md)`. Item 1 in
  [todo.md](./todo.md) is the small core change that makes arbitrary
  record seeding work for every game.
- **No timing/replay determinism yet** — delayed Nerve impulses fire on
  wall time and `Random` is unseeded today; todo.md items 2/5 give the
  core changes that make scenario replays bit-reproducible.
