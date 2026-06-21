// ─────────────────────────────────────────────────────────────────────────
// Raw YAML shape — mirrors the Rust PrefabGameObject / PrefabComponent structs
// ─────────────────────────────────────────────────────────────────────────

export interface PrefabComponentRaw {
  type:   string;
  fields: string[];
}

export interface PrefabGameObjectRaw {
  enabled:    boolean;
  name:       string;
  base?:      string;   // optional path (relative to assets/) to another .comp file
  components: PrefabComponentRaw[];
  children:   PrefabGameObjectRaw[];
}

// ─────────────────────────────────────────────────────────────────────────
// Vector helpers — fields are stored as strings like "position: (0.0,0.0)"
// or "position:(0.0,0.0,0.0)" — key/value separated by ':', whitespace optional.
// ─────────────────────────────────────────────────────────────────────────

export interface Vec2 { x: number; y: number; }
export interface Vec3 { x: number; y: number; z: number; }
export interface Quat { x: number; y: number; z: number; w: number; }

/** Split a raw field string "key: value" / "key:value" into [key, value]. */
export function splitField(field: string): [string, string] {
  const idx = field.indexOf(':');
  if (idx === -1) return [field.trim(), ''];
  return [field.slice(0, idx).trim(), field.slice(idx + 1).trim()];
}

export function joinField(key: string, value: string): string {
  return `${key}: ${value}`;
}

/** Parse "(1.0,2,3)" -> [1, 2, 3]. Tolerates whitespace and missing parens. */
export function parseTuple(value: string): number[] {
  const inner = value.trim().replace(/^\(/, '').replace(/\)$/, '');
  if (inner === '') return [];
  return inner.split(',').map(s => {
    const n = parseFloat(s.trim());
    return Number.isFinite(n) ? n : 0;
  });
}

export function formatTuple(nums: number[]): string {
  return `(${nums.map(n => formatNum(n)).join(',')})`;
}

/** Format a number the way Rust's f32 Display tends to look — trims unneeded trailing zeros
 *  but keeps at least one decimal so it round-trips as a float. */
function formatNum(n: number): string {
  if (Number.isInteger(n)) return n.toFixed(1);
  // Trim to a reasonable precision, drop trailing zeros, keep at least 1 decimal
  let s = n.toFixed(6).replace(/0+$/, '');
  if (s.endsWith('.')) s += '0';
  return s;
}

export function parseVec2(value: string): Vec2 {
  const [x = 0, y = 0] = parseTuple(value);
  return { x, y };
}
export function parseVec3(value: string): Vec3 {
  const [x = 0, y = 0, z = 0] = parseTuple(value);
  return { x, y, z };
}
export function formatVec2(v: Vec2): string {
  return formatTuple([v.x, v.y]);
}
export function formatVec3(v: Vec3): string {
  return formatTuple([v.x, v.y, v.z]);
}

// ─────────────────────────────────────────────────────────────────────────
// Euler (degrees) -> Quaternion. Matches typical Unity/engine convention:
// rotation order Z * Y * X applied as intrinsic XYZ (i.e. roll, pitch, yaw).
// ─────────────────────────────────────────────────────────────────────────

export function eulerDegToQuat(euler: Vec3): Quat {
  const toRad = Math.PI / 180;
  const x = euler.x * toRad * 0.5;
  const y = euler.y * toRad * 0.5;
  const z = euler.z * toRad * 0.5;

  const cx = Math.cos(x), sx = Math.sin(x);
  const cy = Math.cos(y), sy = Math.sin(y);
  const cz = Math.cos(z), sz = Math.sin(z);

  // XYZ intrinsic order
  return {
    x: sx * cy * cz + cx * sy * sz,
    y: cx * sy * cz - sx * cy * sz,
    z: cx * cy * sz + sx * sy * cz,
    w: cx * cy * cz - sx * sy * sz,
  };
}

// ─────────────────────────────────────────────────────────────────────────
// Component-specific typed views
// ─────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────
// Component type registry — populated dynamically from getFacets()
// ─────────────────────────────────────────────────────────────────────────

/** All component type names known to the engine. Populated by loadFacets(). */
export let COMPONENT_TYPES: string[] = [];

export type EntryType =
  | { Asset: string }
  | 'Float' | 'Int' | 'Bool'
  | 'Vector2' | 'Vector3' | 'Vector4';

export interface FieldDescriptor { name: string; type: EntryType; }

/** Maps component_name → ordered FieldDescriptors. Populated by loadFacets(). */
export const FACET_FIELDS = new Map<string, FieldDescriptor[]>();

/** Load facets from the facet.manifest via Tauri and populate COMPONENT_TYPES + FACET_FIELDS. */
export async function loadFacets(): Promise<void> {
  const { api } = await import('../../../api');
  const result = await api.getFacets();
  COMPONENT_TYPES.length = 0;
  FACET_FIELDS.clear();
  for (const comp of result.manifest) {
    COMPONENT_TYPES.push(comp.name);
    FACET_FIELDS.set(comp.name, comp.data.map(f => ({ name: f.name, type: f.data as EntryType })));
  }
}

/** Returns ordered FieldDescriptors for a component type. */
export function getComponentFields(type: string): FieldDescriptor[] {
  if (FACET_FIELDS.has(type)) return FACET_FIELDS.get(type)!;
  return (BUILTIN_COMPONENT_FIELDS[type] ?? []).map(name => ({ name, type: 'Float' as EntryType }));
}

/** Hardcoded fallback field lists before facets load. */
export const BUILTIN_COMPONENT_FIELDS: Record<string, string[]> = {
  'Transform2D':    ['position', 'rotation', 'scale'],
  'Transform3D':    ['position', 'rotation', 'scale'],
  'RendererStatic':  ['asset'],
  'RendererDynamic': ['asset'],
};

export function isTransform(type: string): boolean {
  return type === 'Transform2D' || type === 'Transform3D';
}
export function isRenderer(type: string): boolean {
  return type === 'RendererStatic' || type === 'RendererDynamic';
}

export interface TransformFields {
  position: Vec3;
  rotation: Vec3; // euler degrees, always 3 components even for Transform2D
  scale:    Vec3;
}

/** Reads position/rotation/scale out of a Transform2D/Transform3D component's
 *  fields array, applying the defaults specified by the engine when absent:
 *  position (0,0,0), rotation (0,0,0) (= identity quat), scale (1,1,1). */
export function readTransformFields(comp: PrefabComponentRaw): TransformFields {
  const is2d = comp.type === 'Transform2D';
  let position: Vec3 = { x: 0, y: 0, z: 0 };
  let rotation: Vec3 = { x: 0, y: 0, z: 0 };
  let scale:    Vec3 = { x: 1, y: 1, z: 1 };

  for (const f of comp.fields) {
    const [key, val] = splitField(f);
    if (key === 'position') {
      position = is2d ? { ...parseVec2(val), z: 0 } : parseVec3(val);
    } else if (key === 'rotation') {
      // rotation is always stored as a vec3 of euler degrees, even for Transform2D
      rotation = parseVec3(val);
    } else if (key === 'scale') {
      if (is2d) {
        const t = parseTuple(val);
        scale = { x: t[0] ?? 1, y: t[1] ?? 1, z: t[2] ?? 1 };
      } else {
        scale = parseVec3(val);
      }
    }
  }
  return { position, rotation, scale };
}

/** Writes position/rotation/scale back into the component's fields array,
 *  preserving 2d vs 3d tuple arity and any other fields untouched. */
export function writeTransformFields(comp: PrefabComponentRaw, t: TransformFields): PrefabComponentRaw {
  const is2d = comp.type === 'Transform2D';
  const fields = [...comp.fields];

  const posStr = is2d ? formatTuple([t.position.x, t.position.y]) : formatVec3(t.position);
  const rotStr = formatVec3(t.rotation); // rotation always stored as vec3 euler regardless of 2d/3d
  const scaleStr = is2d ? formatTuple([t.scale.x, t.scale.y, t.scale.z]) : formatVec3(t.scale);

  const setOrAppend = (key: string, valueStr: string) => {
    const idx = fields.findIndex(f => splitField(f)[0] === key);
    const entry = joinField(key, valueStr);
    if (idx >= 0) fields[idx] = entry;
    else fields.push(entry);
  };

  setOrAppend('position', posStr);
  setOrAppend('rotation', rotStr);
  setOrAppend('scale', scaleStr);

  return { ...comp, fields };
}

/** Reads the `asset` field path from a RendererStatic/RendererDynamic component. */
export function readRendererAsset(comp: PrefabComponentRaw): string | null {
  for (const f of comp.fields) {
    const [key, val] = splitField(f);
    if (key === 'asset') return val;
  }
  return null;
}

export function writeRendererAsset(comp: PrefabComponentRaw, assetPath: string): PrefabComponentRaw {
  const fields = [...comp.fields];
  const idx = fields.findIndex(f => splitField(f)[0] === 'asset');
  const entry = joinField('asset', assetPath);
  if (idx >= 0) fields[idx] = entry;
  else fields.push(entry);
  return { ...comp, fields };
}

// ─────────────────────────────────────────────────────────────────────────
// Default factories — used when adding new components/children
// ─────────────────────────────────────────────────────────────────────────

/** A freshly-added component starts with NO fields set. */
export function defaultComponent(type: string): PrefabComponentRaw {
  return { type, fields: [] };
}

export function defaultGameObject(name = 'New GameObject'): PrefabGameObjectRaw {
  return { enabled: true, name, components: [], children: [] };
}

// ─────────────────────────────────────────────────────────────────────────
// Raw tree navigation helpers
// ─────────────────────────────────────────────────────────────────────────

/** Walk a path of child indices down the raw tree. Returns null if path is invalid. */
export function getNodeAtPath(root: PrefabGameObjectRaw, path: number[]): PrefabGameObjectRaw | null {
  let node: PrefabGameObjectRaw = root;
  for (const idx of path) {
    if (!node.children[idx]) return null;
    node = node.children[idx];
  }
  return node;
}

/** Return a new root with the node at `path` replaced by `next`. */
export function setNodeAtPath(
  root: PrefabGameObjectRaw,
  path: number[],
  next: PrefabGameObjectRaw,
): PrefabGameObjectRaw {
  if (path.length === 0) return next;
  const [head, ...tail] = path;
  const children = [...root.children];
  children[head] = setNodeAtPath(children[head], tail, next);
  return { ...root, children };
}

/** Add or replace a field on a component by key, returning a new component. */
export function setComponentField(
  comp: PrefabComponentRaw,
  key: string,
  value: string,
): PrefabComponentRaw {
  const fields = comp.fields.filter(f => splitField(f)[0] !== key);
  fields.push(joinField(key, value));
  return { ...comp, fields };
}
