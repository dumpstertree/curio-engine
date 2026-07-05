# Curio Editor — egui rewrite (pass 7: prefab transform gizmo)

This replaces the Tauri + React frontend with a native `eframe`/`egui`
application that talks to `curio_core` directly — no IPC layer, no webview,
one process.

## Confirmed `curio_core` API corrections (from your working build)

Two facts about `curio_core` that were wrong/incomplete in earlier passes,
now fixed in `runner/game_runner.rs` from your corrected copy:
- `Adapter::request_device` in this wgpu version takes a second
  `Option<&Path>` trace-path argument (`None` here) — this project's
  earlier passes were missing it.
- `EngineServices` has an `assets: *mut AssetLoader` field alongside
  `logger`, constructed the same way as `logger`
  (`self.assets.as_mut() as *mut AssetLoader`). `AssetLoader::new` takes an
  `AssetCache` and `AssetDatabase` (`curio_core::io::{asset_cache,
  asset_database, asset_loader}`). `GameRunner` now owns a `Box<AssetLoader>`
  the same way it owns `Box<Logger>`.

## Prefab 3D viewport: move/rotate/scale gizmo

The prefab viewport (`.comp` files) now has real click-and-drag transform
handles, not just numeric-field editing:

- **Select** an object by clicking it in the 3D view (ray-vs-triangle
  picking, already existed) — the gizmo appears at its position.
- **Move/Rotate/Scale** toolbar in the viewport switches which handles show.
- **Drag a handle** to edit that field live — the object visibly moves/
  rotates/scales in the 3D view as you drag.
- **If the object has no local `Transform3D` yet** (its position/rotation/
  scale is entirely inherited from a `base:`, or it's a fresh object with
  no transform at all), starting a drag adds one automatically, seeded from
  the object's current *effective* (possibly-inherited) values — so it
  doesn't visually jump the instant you grab a handle. Then the specific
  field you dragged gets updated on top of that.
- The actual file write happens **once, on mouse release** — not every
  frame during the drag. Live visual feedback during the drag is a
  transient, in-memory-only override of the render tree (`prefab_tab.rs`
  bakes the live value into a scratch copy before building the frame's
  render entries); `raw`/`resolved`/disk are untouched until you let go.
  This avoids hammering the disk with a write per frame while dragging,
  consistent with the "commit at natural interaction boundaries" pattern
  used everywhere else in this editor (asset rename on blur, dropdown
  selects, etc.).

### Design choices worth knowing about (`prefab_gizmo.rs`)

- **The gizmo is a 2D screen-space overlay**, not real 3D geometry in the
  wgpu pipeline — handles are projected from world space via the camera's
  `view_proj` and drawn/hit-tested with `ui.painter()`, reusing the same
  projection math already used for the Spine placeholder markers. Simpler
  than building actual 3D gizmo meshes, at the cost of handles not being
  depth-tested against the scene (a handle always draws on top, even
  "behind" a mesh from the current angle). Reasonable for a composition
  preview tool; a production-grade gizmo would want real 3D geometry.
- **Translate handles are world-aligned** (X/Y/Z always point the same way
  regardless of the object's rotation — the common "global" gizmo default).
  **Rotate and Scale handles are object-local** (transformed by the
  object's own world rotation) — rotating/scaling "along local X" is the
  meaningful operation for those, unlike translation.
- **Rotation is Euler-additive**, not quaternion composition: a rotate drag
  adds degrees directly to the corresponding Euler `rotation` component.
  Works well from a fresh/zero rotation (the common case) but doesn't
  correctly compose once multiple axes already have non-zero rotation — a
  real gizmo would compose quaternion deltas. Flagged in code rather than
  silently wrong.
- **Scale drags are pixel-delta × a sensitivity constant**, not derived
  from world distance — there's no coherent "world unit of scale" once
  parent rotation/non-uniform scale are involved, so this sidesteps that
  entirely (the same simplification most simple gizmo implementations use).
- Everything here builds on `prefab_transforms.rs`'s new
  `local_matrix`/`world_matrices_for_path` helpers, which factor out (and
  reuse) the same local-transform-matrix logic `collect_render_entries`
  already had.

Not implemented: multi-select, snapping, a "local vs. global" toggle for
translate (it's always global), and a numeric input overlay while dragging
(the inspector's fields update once the drag commits, not live — a minor,
deliberate scope trim to avoid threading the same transient-override
plumbing through both the viewport and the separate inspector panel for a
"nice to have" rather than a "must have").

## Build note

Not compile-checked (same standing toolchain limitation as every pass).
Worth flagging specifically for this one: while writing it, an editing
mistake in `prefab_state.rs` briefly deleted a function's own signature
line while leaving its body in place (caught and fixed by re-reading the
whole file before finalizing — mentioned here mainly as a reminder that
"the diff looked right" isn't sufficient verification without a compiler,
so a `cargo check` here is worth doing before assuming this pass is clean).

---

# Earlier passes (1–6): shell, viewports, asset browser, all asset previews, prefab editing, viewport reliability fix

# Curio Editor — egui rewrite (pass 6: viewport reliability fix + refactor plan)

This replaces the Tauri + React frontend with a native `eframe`/`egui`
application that talks to `curio_core` directly — no IPC layer, no webview,
one process.

## Viewport reliability — the race condition, root-caused and fixed

You reported the live game viewport hanging (something around "accessing
services") with no equivalent bug in the original Tauri build. That's a
real, structural regression from the "tight wgpu integration" work in an
earlier pass, and it's fixed in this one by **reverting it** for the game
viewport specifically.

**What was wrong:** the original Tauri build gave the game its own fully
**private** `wgpu::Device`/`Queue` — nothing else in the process ever
touched it. The "tight integration" pass moved the game runner onto the
*same* `Device`/`Queue`/`egui_wgpu::Renderer` eframe's own UI painting
uses, specifically so the live frame could be displayed with zero CPU
copies. That works, but it means two independent, always-running things —
the game runner thread and eframe's own paint loop — are both hammering the
*same* `renderer.write()`/`.read()` lock every frame, forever, for the
entire time a game is playing. That's a genuine cross-thread contention
point that **could not exist** in the original single-owner design, and
lines up exactly with an intermittent hang that "wasn't a bug before."

**The fix:** the game runner is back to a fully private headless
`wgpu::Device` (`runner/game_runner.rs::setup_gpu`), and frames cross to the
UI thread as plain RGBA bytes via a `Mutex<Option<Frame>>` (restored in
`runner/capture.rs`), uploaded into a persistent `egui::TextureHandle` each
repaint (`panels/center_panel.rs::game_texture`). Nothing else in the
process ever touches this device again — the race is gone structurally, not
patched over.

**This does NOT affect the GLB/PNG/Spine/prefab-scene previews** — those
all render synchronously on the UI thread itself (no independent
background thread), so there was never a concurrent producer to race with
in the first place. They keep the zero-copy shared-texture approach
(`render_shared.rs`, used directly in each preview's own `render()`).

**Cost of the fix:** the live game viewport now pays a CPU→GPU texture
reupload every frame instead of sampling in place — this is the exact cost
the original Tauri build's per-frame canvas texture update already paid, so
it's a return to previously-proven behavior, not a new regression.

If you still see hangs after this, they're a different bug — worth knowing
the shape of what changed so we can rule this cause out cleanly.

## Refactor plan (recommended, not yet started except where noted)

Now that every asset type has a real implementation, the next round of work
should be tightening the port into an application-shaped codebase rather
than a line-for-line translation. Recommended order:

1. **Unify the four parallel preview state machines** (`glb_preview`/
   `png_preview`/`anim_preview`/`prefab`, each with its own `ensure_X`
   loader and error field on `EditorState`) into one `AssetPreview` enum.
2. **Extract shared 3D-preview infrastructure** — `glb_viewer.rs`,
   `anim_viewer.rs`, and `prefab_viewer.rs` each hand-roll their own orbit/
   pan camera and texture-registration boilerplate. A `PreviewSurface`
   helper would remove real duplication.
3. **Cache `fs_ops::read_manifest`/`get_facets`** instead of re-reading disk
   on every dropdown render and every component block, each frame.
4. **Real undo/redo** — both `asset_state.rs` and `prefab_state.rs` already
   funnel every edit through an action-enum + `apply()`, so a proper undo
   stack is nearly free to add on top, and would make the (currently dead,
   inherited-as-dead-from-the-original) Undo/Redo menu items actually work.
5. **Split `EditorState`** into owned sub-structs (`play`, `previews`,
   etc.) instead of ~15 flat sibling fields.
6. **Reconsider the full-tree-clone-per-frame pattern** in `asset_tree.rs`/
   `prefab_tab.rs` — a carryover from React's immutable-render model, not
   strictly necessary in egui. Fine at current scene sizes; worth
   revisiting if prefabs grow large.

---

# Earlier passes (1–5): shell, viewports, asset browser, all asset previews, prefab editing

# Curio Editor — egui rewrite (pass 5: prefab `.comp` editing)

This replaces the Tauri + React frontend with a native `eframe`/`egui`
application that talks to `curio_core` directly — no IPC layer, no webview,
one process.

## What's new in this pass

The last unported asset type — `.comp` (prefab composition) — is now a
real, working editor rather than a placeholder. This was the biggest single
piece of work in the whole rewrite so far (~2,150 lines of original
TypeScript across 9 files), so it's broken down in detail below.

**Fully ported, no cuts:**
- The `base:` inheritance/override resolver (`prefab_resolver.rs`) — an
  object can inherit from another `.comp` (by manifest ID or legacy path),
  and every field tracks whether it's inherited or explicitly overridden.
  Cycle detection, multi-level chains, per-field/per-component merging —
  all ported faithfully from `prefabResolver.ts`.
- The full data model and field-encoding conventions (`prefab_types.rs`) —
  vector tuple parsing/formatting, Euler-to-quaternion conversion, the
  2D/3D transform arity handling, the on-disk field-string convention
  (`"position: (0,0,0)"`).
- The tree + component + field inspector (`panels/prefab_tab.rs`'s
  `show_inspector`) — enable/disable, rename, add/remove/duplicate
  children, add/remove/reorder components, edit every field type
  (vectors, bools, asset references, generic text), an asset-picker
  dropdown backed by the manifest, and a facet-driven "+ Add Facet" menu.
  Saves to disk (as YAML, only the child's own overrides — same
  keep-`.comp`-files-small philosophy as the original) on every commit.

**Real, but with two documented scope cuts** in the 3D scene preview
(`prefab_viewer.rs`):
- **No transform gizmos.** The original used three.js's `TransformControls`
  for click-and-drag translate/rotate/scale handles in the 3D view — a
  substantial, genuinely separate piece of work (custom gizmo geometry,
  ray-vs-handle hit testing, drag math) from "render the composed scene."
  Transforms are fully editable via the inspector's numeric fields instead,
  which was always a complete path to the same data even in the original.
  Click-to-select in the 3D view *is* implemented (real ray-vs-triangle
  picking against the loaded GLB geometry) and stays in sync with the
  inspector tree selection both ways.
- **Spine (`RendererDynamic`) entries render as a placeholder marker**
  (a colored dot + label), not the actual animated skeleton in 3D space.
  Getting real Spine rendering into an arbitrary 3D world transform means
  sharing a draw pass between `anim_viewer.rs`'s orthographic 2D pipeline
  and `glb_viewer.rs`'s perspective 3D one — a meaningfully separate
  integration task. The marker at least shows where the object sits in the
  composition and is clickable-select via the inspector tree.

With this, **every asset type the tree recognizes has a real editor or
preview**: PNG, GLB, Spine `.anim`, and now `.comp` prefabs.

## Architecture notes

- **Action-queue editing**, same pattern as `asset_state.rs`: the tree/
  inspector widgets walk a cloned snapshot of the prefab immutably each
  frame and return a list of `PrefabAction`s, applied afterward by
  `PrefabState::apply` — avoids fighting the borrow checker over mutating
  the tree while also reading it mid-traversal.
- **No debounced autosave.** The original had a 300ms debounce timer
  before writing to disk. This port commits (and saves) on blur/Enter/
  dropdown-select instead — the same "commit points" the asset tree's
  rename field already used — so there's no keystroke-level save thrashing
  to guard against in the first place, and no timer needed.
- **The 3D scene renderer deliberately duplicates `glb_viewer.rs`'s
  local-space GLB parsing** rather than sharing it. The prefab scene needs
  to load multiple GLBs and place each at its own world transform (with
  caching, since several prefab entries might reference the same asset);
  extracting a common function felt like more risk to the already-working
  single-asset GLB preview than it was worth for this pass.
- **Ray picking is CPU-side and brute-force** (every triangle, every
  click) — genuinely fine at preview-scene triangle counts, not something
  to optimize preemptively.

## Not verified — same caveat as every pass

None of this was compile-checked (see the persistent toolchain note
further down). The prefab system is pure Rust + `serde_yaml` + `glam` +
`gltf` — nothing here depends on an unverified third-party API shape the
way the Spine pass did, so the risk profile is more like "ordinary
borrow-checker/typo fixes" than "wrong method name for a crate I can't
read." The one thing genuinely worth a second look: `PrefabGameObjectRaw`'s
`#[serde(default = ...)]`-based tolerant parsing is less forgiving than the
original TS `normalize()` for deeply malformed YAML (e.g. `components:`
being a string instead of a list would hard-fail here, where the TS
version fell back to `[]`) — reasonable for files this same editor writes,
worth knowing if `.comp` files can come from elsewhere too.

---

# Earlier passes (1–4): shell, viewports, asset browser, PNG/GLB/Spine previews

# Curio Editor — egui rewrite (pass 4: Spine `.anim` preview)

This replaces the Tauri + React frontend with a native `eframe`/`egui`
application that talks to `curio_core` directly — no IPC layer, no webview,
one process.

## What's new in this pass

The last remaining asset-preview type is filled in. Previous passes covered
the shell, the live game viewport, the asset browser, and PNG/GLB previews:

| Asset type | Status |
|---|---|
| `.png` | Real preview (pass 3) — decoded via the `image` crate, uploaded as a normal egui texture (`png_viewer.rs`). |
| `.glb` | Real preview (pass 3) — parsed with `gltf`, rendered with a hand-written wgpu pipeline (`glb_viewer.rs`), orbit camera. Flat lit shading only, no materials/textures (documented scope cut). |
| **`.anim` (Spine)** | **New this pass — real preview.** The format turned out to be a plain zip of `skeleton.atlas` + `skeleton.json` (Spine 3.8 native JSON) + `skeleton.png` — confirmed by actually reading `AnimViewport.tsx`'s implementation rather than guessing. Parsed with `rusty_spine` (your local `rusty_spine3.8` crate), rendered with a small hand-written 2D wgpu pipeline (`anim_viewer.rs`): textured, vertex-colored, alpha-blended triangles, auto-framed 2D camera (drag to pan, scroll to zoom, reset button), animation-switcher dropdown, and a live elapsed/duration readout. Same shared-texture registration pattern as the game viewport and the GLB preview — zero-copy, its own separate `TextureId`. |
| `.comp` (prefab) | Still the only remaining placeholder. Bigger scope — a three.js prefab *scene* plus its own tree-editing inspector, not a single-asset preview — worth scoping as its own task. |

With this, **PNG, GLB, and Spine `.anim` all have working previews** — the
only unported asset-viewer type left is prefab editing.

## How the Spine preview works

1. `unpack_anim_zip` pulls `skeleton.atlas`/`skeleton.json`/`skeleton.png`
   out of the `.anim` zip by suffix match, exactly like the original
   `JSZip`-based TS code did (including erroring with the full file listing
   if something's missing).
2. **Scope cut, deliberate:** rather than wiring up `rusty_spine`'s
   process-global texture-callback registry (meant for atlases with
   multiple pages, each with its own filter/wrap settings), this just
   builds one wgpu texture straight from the decoded `skeleton.png` and
   binds it for every draw call. The `.anim` format always produces exactly
   one page, so this is correct for every file this pipeline can emit — it
   would only fall short for a hand-built multi-page Spine atlas, which
   isn't a shape your export step generates. The callbacks are still
   registered (satisfying whatever internal bookkeeping expects a
   `renderer_object` to exist) but do nothing meaningful.
3. Every frame: `SkeletonController::update(dt, Physics::Update)` advances
   playback, `combined_renderables()` produces the current frame's
   triangles across all slots, which get merged into one vertex/index
   buffer and drawn in one pass (also means per-slot blend modes and
   premultiplied-alpha are not respected — flagged clearly in
   `anim_viewer.rs`'s module doc, same spirit as GLB skipping materials).
4. Registers/updates the result with the shared `egui_wgpu::Renderer`, same
   `&egui_wgpu::RenderState`-in, `TextureId`-out pattern used everywhere
   else in this rewrite.

## Build note — `rusty_spine`'s exact Rust API wasn't directly inspectable

Unlike `gltf` (well-documented on docs.rs, verified against it directly)
and `wgpu` (verified against your specific `egui-wgpu = "0.31.1"` pin), your
`rusty_spine3.8` fork is a local path dependency I can't read — I worked
from:
- The mainline `rusty_spine` crate's docs.rs pages (confirmed:
  `Atlas::new(bytes, dir)`, `SkeletonJson::new`/`read_skeleton_data`,
  `AnimationStateData::new`, the general `SkeletonController` /
  `combined_renderables()` shape from the `miniquad` example)
- The `rusty_spine3.8` fork's own README, which states it keeps the same
  Rust API as mainline `rusty_spine`, just targeting the Spine 3.8 C runtime
  instead of 4.2

What's **not verified** (couldn't reach docs for the exact 3.8-runtime
struct fields): `SkeletonData`'s animation/bone/slot accessor names,
`Renderable`'s exact field types, `Color`'s field names, and `TrackEntry`'s
playback accessors. All called out explicitly in `anim_viewer.rs`'s module
doc comment with the specific assumption made for each. If your fork's API
differs, these should show up as clearly-isolated `cargo check` errors
inside `AnimPreview::load`/`AnimPreview::render` — nothing here reaches
outside that one file.

---

# Earlier passes (1–3): shell, viewports, asset browser, PNG/GLB previews

## Where this sits relative to your repo

Same overall pattern as the live game viewport (`runner/viewport.rs`), applied
to a static asset instead of a running simulation:

1. `GlbPreview::load` parses the GLB with `gltf::import_slice`, walks the
   scene graph baking each node's world transform straight into its
   vertices (simplest way to flatten an arbitrary hierarchy into one draw
   call for a preview — no skinning, no per-node draw calls), and uploads
   vertex/index buffers plus a tiny lit-shading pipeline (WGSL, inlined in
   `glb_viewer.rs`).
2. Every frame the preview is visible, `GlbPreview::render` re-renders into
   an owned color+depth texture pair (only actually reallocated on a real
   resize) and registers/updates that color texture with the *same shared*
   `egui_wgpu::Renderer` the game viewport and eframe's own UI painting use
   — via `render_state.renderer.write()`, no type-naming needed (same
   lesson learned in pass 2: don't hand-spell the lock type, just go through
   `&egui_wgpu::RenderState`).
3. `asset_tab.rs::glb_viewport` displays the result with `ui.image`-style
   painting, forwards drag/scroll input back into the camera, and requests
   continuous repaint while the user is actively orbiting (same idea as the
   game viewport requesting repaint while `Playing`).

Camera framing (initial distance/target) is computed from the mesh's
bounding sphere so an arbitrary GLB opens reasonably centered and scaled,
rather than needing manual framing per file.

## Build note — this file is the highest wgpu-API-surface risk yet

`glb_viewer.rs` touches a lot more of wgpu's low-level struct surface than
anything in passes 1–2 did — `RenderPipelineDescriptor`, `VertexState`,
`FragmentState`, `RenderPassColorAttachment`, `DepthStencilState`, and so on,
maybe a dozen struct literals in total. Fields on these have shifted across
wgpu releases more than once recently (`entry_point` went from `&str` to
`Option<&str>`, `compilation_options` and `cache` were added,
`RenderPassColorAttachment` gained a `depth_slice` field). Everything here
was written against wgpu 23-era APIs (matching your `egui-wgpu = "0.31.1"`
pin from pass 2), but this is genuinely the file most likely to need small
field-level fixes on your first `cargo check` — if so, they should all be
obvious "field X doesn't exist / expected Y found Z" errors localized to
`GlbPreview::load`'s pipeline-building code, not anything structural.

Also unverified: `gltf::import_slice`'s exact return shape
(`(Document, Vec<buffer::Data>, Vec<image::Data>)`) and `Node::transform()`
API — these have been stable in the `gltf` crate for a long time, so I'm
fairly confident here, but flagging since none of this was compile-checked
(same sandbox toolchain limitation as every prior pass — see below).

---

# Earlier passes (1–2): shell, game viewport, asset browser

## Where this sits relative to your repo

Originally: `curio-editor/src-tauri/Cargo.toml` had `curio_core = { path =
"../../curio_core" }`, i.e. `curio_core` was a sibling of `curio-editor/`.

This project drops the `src-tauri/` nesting, so it expects to sit **directly
next to** `curio_core/`:

```
your-repo/
  curio_core/
  curio-editor-egui/   <- this project
```

If you place it elsewhere, fix the `path` in `Cargo.toml`.

## What's new in this pass

Pass 1 was the app shell only, with the game viewport and Asset tab both
stubbed. This pass fills in both:

| Area | Status |
|---|---|
| **Game viewport** | **Ported — tight wgpu integration, zero CPU copies.** The runner thread now shares eframe's own `Device`/`Queue`/`egui_wgpu::Renderer` (grabbed once in `app.rs` from `cc.wgpu_render_state`) instead of opening a private headless device. It renders into `capture_texture` as before, then registers/updates that texture directly against the shared `Renderer` (`runner/viewport.rs`). `center_panel.rs` displays it with `ui.image(...)` and forwards pointer/keyboard input back to the game. No frame bytes ever leave the GPU. |
| **Asset browser** | **Ported.** Full file tree (`panels/asset_tree.rs` + `asset_state.rs`): lazy-loaded folders, include/exclude checkboxes backed by `.meta` sidecars, rename, delete (with confirm), drag-and-drop move, Import (native file picker via `rfd`), New Folder, New Comp. All the file ops from `commands.rs`'s file-system/meta/manifest sections are ported to `fs_ops.rs`. |
| **GLB / PNG / Animation / prefab preview** | **Still stubbed** — these were three.js scenes (orbit camera, lighting, grid) with no Rust equivalent. Selecting a `.png`/`.glb`/`.anim`/`.comp` file now correctly drives the inspector and shows a labeled placeholder in the center panel (`panels/asset_tab.rs`) instead of doing nothing, but there's no actual rendering yet. This is real work — happy to take it on next if useful, but it's a much bigger lift than the shared-texture swap or the file tree were. |
| App shell (toolbar, top tabs, panels, status bar) | Ported (pass 1) |
| Play/Pause/Stop + compile pipeline | Ported (pass 1) |
| Object tree + inspector | Ported (pass 1) |
| Input mapping tab | Still a placeholder (out of scope) |

## Viewport — how the shared-texture path works

1. `app.rs`'s `CurioEditorApp::new` reads `cc.wgpu_render_state` (only
   present because `Cargo.toml` builds eframe with the `"wgpu"` feature) and
   packages `Device`/`Queue`/`Arc<RwLock<egui_wgpu::Renderer>>` into a
   `RenderShared`, handed to `EditorState`.
2. `EditorState::ensure_runner_started` passes that into
   `GameRunner::new`. `GameRunner::setup_gpu` creates `capture_texture` from
   that *same* `Device` (with `TEXTURE_BINDING` added to its usage flags, so
   egui can sample it — the one behavioral addition beyond the original's
   `RENDER_ATTACHMENT | COPY_SRC | COPY_DST`).
3. Every frame, after `Curio::render` draws into `capture_texture` and the
   command buffer is submitted, `runner::viewport::register_or_update` is
   called with that texture's view. First call registers a new
   `egui::TextureId`; every call after that just repoints the same
   `TextureId` at the (possibly resized) view — cheap, no per-frame
   allocation on egui's side.
4. `center_panel.rs::game_texture` reads `runner::viewport::current()` each
   repaint and calls `ui.image(...)` — it's sampling whatever the render
   thread most recently drew, in place, with no readback.

**wgpu version matching — confirmed resolved.** `curio_core` depends on
`egui-wgpu = "0.31.1"` directly (not a separate `wgpu` crate), and this
project's `Cargo.toml` pins the exact same `egui = "0.31.1"` / `egui-wgpu =
"0.31.1"` and imports all wgpu types through `egui_wgpu::wgpu::*` rather than
a standalone `wgpu` dependency. Cargo will unify both crates onto a single
`wgpu` version graph-wide, so the `Device`/`Queue` pointer casts in
`GpuHandle` (`curio_core`) and `RenderShared` (this project) are guaranteed
to agree — no action needed here.

## Asset browser — what's real vs. what's a placeholder

Fully working, same behavior as `AssetFileTree.tsx`:
- Lazy directory loading (children fetched on first expand)
- Include/exclude checkbox per file, backed by a `<file>.meta` YAML sidecar
  (auto-created on first view, same random-ID scheme as the original)
- Rename (inline text edit, Enter/Escape), Delete (with Yes/No confirm row),
  both also move/delete the `.meta` sidecar
- Drag-and-drop move between folders, with name-conflict resolution
  (`name_1`, `name_2`, ...)
- Toolbar: Import (native file picker via `rfd`, replacing the Tauri dialog
  plugin), New Folder, New Comp (writes a stub `.comp` prefab file)
- `asset.manifest` rebuild fires after every mutating op, same as the
  original's `await api.rebuildManifest()` calls

Selecting a file correctly drives the right-hand inspector (shows path +
`.meta` id/included state) and switches the center panel to a
type-labeled placeholder — but there's no actual PNG/GLB/animation
rendering or prefab tree editing yet. See the table above.

## Assumptions about `curio_core`'s API

`curio_core` wasn't in the uploaded zip (it's the sibling crate at
`../../curio_core`), so its public surface was inferred entirely from how
`commands.rs`/`lib.rs`/`runner2.rs` used it, plus `src/types.ts`'s comment
`// Matches Rust structs exactly`. Everything from pass 1 still applies
(see below); nothing new was assumed for the viewport/asset work beyond the
`EngineServices`/`GpuHandle` shape already documented.

- `ObjectState { object_name: String, children: Vec<ObjectState>, components: Vec<ComponentState> }`
- `ComponentState { component_name: String, fields: Vec<FieldState> }`
- `FieldState { field_name: String, data: /* some enum */ }` — unknown
  shape; `panels/inspector.rs` renders `field.data` via `{:?}` (Debug)
  rather than the type-specific coloring the old TS inspector did. Flagged
  with a `NOTE:` comment at the call site — swap for a real `match` once you
  paste in the actual type.
- `TabState { tab_name: String, objects: Vec<ObjectState> }`,
  `TabGroupState { id_for_tabs: HashMap<String, Vec<TabState>> }` — both
  `Clone` + assumed `Send` (both true in the original, which lived in a
  `std::sync::Mutex`).
- `Curio::log(Severity, &str)` (static), `Severity::{Info, Warning, Error}`,
  `get_and_clear_logs() -> Vec<(Severity, String)>`, `Curio::render(&mut
  self, &Texture, &TextureView, &mut CommandEncoder)`,
  `Curio::application_refresh`/`window_opened`/`context_snapshot`/`tab_snapshot`,
  `GpuHandle::device()`/`queue()`, `EngineServices`, `GpuHandle`, `Logger`,
  `set_services` — all ported as direct 1:1 uses, unchanged from
  `runner2.rs` except that `GpuHandle`'s raw pointers now point into
  eframe's shared `Device`/`Queue` rather than a private headless one (see
  "Viewport" above — confirmed safe given your `egui-wgpu = "0.31.1"` pin).
- `curio_core::io::file::File::read(path) -> Vec<u8>` (used by `Project::load_local`).

If any of these don't match, the compiler will point at exactly the right
spot — nothing here is deeply threaded through the codebase.

## Build note

I don't have a working modern Rust toolchain in the sandbox I built this in
(only an apt-installed rustc 1.75, too old for current `eframe`/`wgpu`
releases — lots of their transitive deps now require the 2024 edition). What
I validated:

- `state.rs`/`project.rs`/`runner/*.rs` (minus the new `viewport.rs`/
  `game_runner.rs` texture-registration calls, which need real `egui-wgpu`
  to check) — compiled clean in pass 1 against a hand-built mock of the
  `curio_core` API, with real `wgpu`.
- `fs_ops.rs`/`asset_state.rs`/`panels/asset_tree.rs`/`panels/asset_tab.rs`/
  the viewport changes in `panels/center_panel.rs` — **not compile-checked**
  this pass; same toolchain wall as before (couldn't get a working
  `eframe`/`egui-wgpu` build with rustc 1.75). Written carefully against
  documented egui 0.31 / egui-wgpu 0.31 APIs
  (`register_native_texture`, `update_egui_texture_from_wgpu_texture`,
  `Painter::image`, `Response::drag_started_by`, etc.) but genuinely
  unverified.

Run `cargo check` on your end first. The likeliest friction points, in
rough order of likelihood:
1. `egui_wgpu::Renderer`'s exact method names/signatures for registering vs.
   updating a native texture — I used `register_native_texture` and
   `update_egui_texture_from_wgpu_texture`, which are the 0.31.1-era names
   (confirmed against your actual `egui-wgpu = "0.31.1"` pin), but worth a
   quick check against your `Cargo.lock` output.
2. `egui::Response`'s per-button drag/click methods
   (`drag_started_by`/`drag_stopped_by`/`clicked_by`) — these exist in
   modern egui but got renamed a few times across versions.
3. The `curio_core` API assumptions in the section below (`ObjectState`,
   `FieldState`, etc.) — unrelated to the wgpu-version question, which is
   now resolved (see "Viewport" above).

## Not ported (out of scope for this pass)

- GLB / PNG / Animation / prefab 3D preview rendering (see table above —
  this is the next logical chunk of work, and it's substantial: essentially
  building a wgpu-based mesh/texture/skeletal-animation viewer with orbit
  camera controls from scratch)
- Prefab tree editing (`PrefabInspectorView`/`prefabResolver`/
  `prefabTransforms` — depends on the prefab viewport existing first)
- Input mapping tab
- Facets (`get_facets`/`FacetManifest`) are ported to `fs_ops.rs` but not
  wired into any UI yet — the original never showed them anywhere either
  outside of what would presumably be prefab component editing
- File picker / native "Load project" dialog — same as before, just reads
  `./test.proj` on launch

