# Plan: Figma → Curio Composition Importer

Goal: given a Figma API token + file/node URL, produce Curio `.comp` assets (plus textures/fonts)
that the existing runtime spawn path (`spawn_prefab_recursive_internal`) instantiates as-is.
No new runtime code is required to *display* the result — the importer just has to emit
valid YAML in the engine's own schema.

---

## 0. Pipeline overview

```
Figma token + file_key (+ optional node_id)
        │
        ▼  Stage 1 — Fetch (REST, cached)
   figma.json  (node tree + globalDefs styles/components)
        │
        ▼  Stage 2 — Translate (pure, testable)
   Intermediate IR (engine-flavored node tree: kind, abs bounds, fill, text, z-order)
        │
        ▼  Stage 3 — Emit
   assets/compositions/<name>.comp  +  assets/texture/*.png  +  assets/fonts/*
        │
        ▼  Stage 4 — Register
   asset.manifest entry  →  generated_assets()  →  Assets::append_listing_group
        │
        ▼  Runtime (already exists, unchanged)
   Composition::from_bits (serde_yaml) → spawn_prefab_recursive_internal → hecs entities + facets
```

Stages 1–3 live in **one importer**; stage 4 reuses the existing build flow.

## 1. Where the importer lives — recommended shape

**Build-time importer binary**, not a runtime system. Two placements:

- **Primary: a new tool binary** (e.g. `tools/figma_import/` or under `workflow_scripts/`) run in
  CI/dev loop: `FIGMA_TOKEN=... figma-import --file <file_key> --node <id> --out assets/compositions/`.
  Output lands in the normal `assets/` tree, gets embedded through `asset.manifest` →
  `generated_assets()` like every other asset. Ships offline, zero network at game boot.
- **Optional second surface: a tab in `curio-editor`** (Tauri/React). The editor already parses
  `.comp` prefabs (`prefabResolver.ts`, `assetLoaders.ts`), so an "Import from Figma" tab can
  reuse its prefab types to preview the result before it's ever committed.

**Why not runtime-from-Figma:** the engine *can* do it — `AssetDatabaseListing::RemoteToCache`
already fetches remote URLs (reqwest, HEAD dirty-check, local cache). But Figma's image render
URLs expire (~7 days) and the REST JSON endpoints require the `X-Figma-Token` header, which
`fetch_remote_asset` (plain GET, no headers) can't send today. Runtime fetching would need an
importer proxy or an API change; build-time import sidesteps all of it and matches how every
other asset is handled. (If hot-reloading designs is a hard requirement, add a small header-aware
listing variant later — don't start there.)

## 2. Stage 1 — Fetch (REST API)

- Auth: `X-Figma-Token` header from env (`FIGMA_TOKEN`), never in argv/committed config.
- Endpoints:
  - `GET /v1/files/{file_key}?depth=1..3` — node tree. Fetch shallow first; re-GET with `ids=`
    for subtrees that contain the target frames. Large files 429 → honor `Retry-After`,
    exponential backoff, persist raw JSON to a `.cache/figma/` dir.
  - `GET /v1/files/{file_key}/styles` and `/components` — resolve component/style metadata.
  - `GET /v1/images/{file_key}?ids=…&format=png&scale=2` — raster exports (image fills,
    unsupported vectors, whole subtrees as fallback bakes).
- Pin inputs in a small project manifest (e.g. `figma.manifest`: file_key, node_id list, export
  scale) so the import is re-runnable and diff-able, same spirit as `asset.manifest`.
- Determinism rule: raw fetched JSON is committed to the cache dir; the *translation* stage must
  not touch the network so re-imports are golden-testable.

## 3. Stage 2 — Translation map (the actual "parser")

Pure function: `figma_json + globalDefs → Vec<Composition>`. Node-kind mapping:

| Figma | Curio |
|---|---|
| `FRAME` / `GROUP` | `Composition` node (recursion; children tree) |
| `RECTANGLE`/`SHAPE` with image fill | child w/ `rendererimage` facet (`asset: <name>`, `tint`) |
| `TEXT` | child w/ `renderer_text` + `transform2d` facets (string content, font asset) |
| `INSTANCE` of a Figma Component | `base:` reference to the component's own emitted `.comp` (prefab reuse!) |
| `VECTOR` / boolean ops / strokes | rasterize via Images export → treated like image fill |
| `auto-layout` frames | `ui_node` (ui_overlay flow) children, or baked offsets — decide per project |
| hidden layers (`visible: false`) | `enabled: false` on that node (keep it, don't drop: diffs stay stable) |
| opacity / blend modes | opacity → `tint` alpha; exotic blends → warn + skip |

Per-node data extraction → facet `fields: ["key:value", …]`:

- **Transform**: `absoluteTransform` (or parent-relative `relativeTransform`) →
  `transform2d` `position:x,y`, `scale:…`, `rotation:deg`
  - Coordinate conversion: Figma is top-left px; `Transform2D::get_matrix` remaps `0..1` to the
    frustum and y grows upward. So emit `x = fx / canvasW`, `y = 1 - fy / canvasH` (flip), scale
    relative to the unit quad `RendererImage::set_asset` builds (1.0 wide, aspect-corrected).
    All of this belongs in one `figma_to_transform()` function with unit tests.
- **Z-order**: Figma child order → `order:<n>` on `transform2d` (render_order).
- **Fill color** → `tint:#RRGGBBAA` — must match the exact string grammar each facet's
  `FieldOverride::apply()` parses (`Color::parse`, `Vector2::parse`, `Quaternion::from_euler`).
  Mirror the parsers: one small Rust `value_fmt` module shared by importer and any editor codegen,
  so the importer emits exactly what `apply()` consumes.
- **Naming**: Figma layer name → slug for asset keys (`assets/texture/…`, `assets/compositions/…`);
  collisions get deterministic suffixes; name collisions warn (lookup is by name via
  `Assets::try_lookup_key_for_name`).

Reuse is the headline feature: emit one `.comp` per Figma Component/variant, and each `<INSTANCE>`
becomes `{ base: <prefab-name> }` — `spawn_prefab_recursive_internal` already loads `base`
recursively, so Figma component reuse maps 1:1 onto Curio prefab reuse for free.

## 4. Stage 3 — Emission

- YAML: `serde_yaml::dump` of the real `Composition` structs (import `curio_core` types or
  duplicate the schema with `#[derive(Serialize)]` against the identical shape — importing the
  types is better: schema drift becomes a compile error).
- Textures: request PNG exports for every image-fill node (batch ≤ 500 ids/call), write
  `assets/texture/<slug>.png`. Same render URLs also cover baked vectors.
- Fonts: every `styleName` in TEXT nodes → check against `assets/fonts/`; emit missing-fonts
  report (can't auto-download font binaries from the REST API; require manual add).
- Output contract per import run: `out/manifest.json` listing every file written + Figma node id
  each came from → enables diff-based re-import and editor "sync from Figma" buttons.

## 5. Stage 4 — Registration (existing flow, no new concept)

- New `.comp`/`.png` files get appended to `asset.manifest` → build script emits
  `generated_assets()` entries (`AssetDatabaseListing::Embedded`) as today.
- Optional dev-loop mode: `AssetDatabaseListing::Local("…")` during playtesting without
  re-embedding.

## 6. Validation layer (import-time, before runtime ever sees it)

Runtime is deliberately forgiving (unknown facet → `Curio::log(Error)` + skip; unknown asset →
Warning). The importer must be the strict side:

- Facet `type` names validated against the registry key rule
  (`type_name::<T>().split("::").last().to_lowercase()`), i.e. emit only
  `transform2d`, `rendererimage`, `renderer_text`, `ui_node`, … — fail the import otherwise.
- Field keys validated against each facet's `apply()` match arms — generate this list from a
  small table that mirrors the facets (or parse `FieldOverride` impls via a codegen macro later).
- Cycle guard on `base:` chains (spawn recursion is unguarded today); depth cap + error.
- Report: `{errors[], warnings[]}` per node id → printed summary + per-node annotations written
  back to the editor's import preview.

## 7. Testing

- **Golden tests** in `curio_core_tests` style: committed fixture `figma.json` files (a hand-crafted
  mini doc covering every mapping row: frame, image rect, text, instance, hidden, auto-layout) →
  assert exact emitted YAML. This is the parse-table regression net.
- **Round-trip**: `serde_yaml::from_str::<Composition>` on generated output — catches schema drift
  at compile/test time (same test catches it upstream).
- One integration test in `curio_core_tests/src/io/asset_database.rs` style: load generated comp
  through the asset DB, confirm facet types resolve in `COMPONENT_REGISTRY`.

## 7b. Facet-gap audit (Figma provides → engine has/hasn't)

Bake rule: purely-visual Figma features can be rasterized per-node via the Images export API
(which already applies effects/strokes/corners/gradients/masks/vectors/blends), so they need
**no new component**. Features that must stay live (layout, text, interactivity) need engine surface.

Needed before the importer is feature-complete (new fields/facets):
- ~~`absolute`~~ **done** (2026-11): `UiNode` grew an `absolute` bool; `ui_style` hangs flagged panels
  at their own `Transform2D` offset via taffy `Position::Absolute` + insets, and the importer sets
  it on frames without auto-layout — free-form Figma placement survives at all nesting levels.
- **UiNode flex fields** — biggest win left: `update_ui_layout.rs` runs on taffy but hardcodes
  `flex_direction: Row` and defaults. Add `direction`, `justify`, `align_items`, `align_self`,
  `layout_align` → taffy supports all natively, near-free.
- **RendererText**: `line_height`, `letter_spacing` fields; mixed-style runs = split runs into
  sibling leaves or a `RendererRichText` facet (baking kills localization).
- **RendererImage**: `crop:x,y,w,h` + `tile` for FILL/FIT/CROP/TILE modes — or rely on
  per-node cropped PNG exports (cheaper).
- **Constraint2D** facet — only if runtime canvas ≠ Figma canvas; else resolve at import.
- **ClipRect / scrollable** facet — warn-and-skip now, facet later.
- `UiNode.font` asset field (ui_overlay labels are a shared 12px mono atlas) — or map Figma UI
  text to `RendererText` leaves under a `UiNode` parent.
- Component variants/states: emit one `.comp` per variant; switching is impulse/game-logic glue.

Bake, don't build: gradients, rounded corners, strokes, drop shadow/layer blur, blend modes,
vector/boolean shapes, clip frames (engine only ships `lit`/`unlit` shaders; native versions
only if the property must animate at runtime).

Editor-only concepts with no engine equivalent: design grids, sections → skip.

## 8. Milestones

1. **M1 — Fetch + IR** (~small): REST client, token env, caching, `depth` paging, fixtures.
2. **M2 — Translate + emit minimal** (~medium): frames/rect-with-image/instance only → first real
   `.comp` boots through the existing spawn path in a scratch Curio. This is the proof the schema
   guess is right.
3. **M3 — Facet gaps first** (~medium): add UiNode flex fields, RendererText
   line_height/letter_spacing, RendererImage crop (see §7b) — then extend the full mapping table
   (text, fonts, tint/opacity, z-order, y-flip calibration, auto-layout as UiNode panels).
4. **M4 — Validation + reports** (~small): strict registry cross-check, cycle guard, manifest.json.
5. **M5 — Editor integration** (~medium, optional): import tab in `curio-editor` reusing
   `prefabResolver.ts` for preview; `sync from Figma` diff-import using out/manifest.json.

## 9. Known gaps / parked

- Effects, constraints, layout grids, vector editing → rasterize now, native facets later.
- Figma Variables → no first-class Curio theming target today; resolve variables at import time.
- Runtime RemoteToCache-from-Figma (header support gap in `fetch_remote_asset`) → only if live
  design hot-reload is demanded; needs an importer-proxied listing variant.
- `ui_overlay`'s 256-parent chain guard and flow semantics constrain auto-layout mapping — pick
  baked offsets for M2, ui_node for M3 after a look at `update_ui_layout.rs` behavior.
