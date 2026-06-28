import { load as yamlLoad } from 'js-yaml';
import { api } from '../../../api';
import type { PrefabGameObjectRaw, PrefabComponentRaw } from './prefabTypes';
import { splitField } from './prefabTypes';

// ─────────────────────────────────────────────────────────────────────────
// Resolved types — carry override metadata alongside merged data
// ─────────────────────────────────────────────────────────────────────────

/** A field that knows whether it was overridden by the child or inherited from base. */
export interface ResolvedField {
  raw:        string;   // full "key: value" string
  overridden: boolean;  // true = set explicitly in child, false = inherited from base
}

/** A component that knows whether it was added/overridden by the child. */
export interface ResolvedComponent {
  type:           string;
  fields:         ResolvedField[];
  /** true = this component was not present in base (added by child or is a fresh object) */
  addedByChild:   boolean;
}

/** A fully resolved (merged) GameObject. */
export interface ResolvedGameObject {
  enabled:    boolean;
  name:       string;
  base?:      string;
  /** true = this child node was added by the child prefab (not in base) */
  addedByChild: boolean;
  components: ResolvedComponent[];
  children:   ResolvedGameObject[];
}

// ─────────────────────────────────────────────────────────────────────────
// YAML loading helpers
// ─────────────────────────────────────────────────────────────────────────

function normalize(raw: any): PrefabGameObjectRaw {
  return {
    enabled:    raw?.enabled ?? true,
    name:       raw?.name ?? 'GameObject',
    base:       typeof raw?.base === 'string' && raw.base.trim() ? raw.base.trim() : undefined,
    components: Array.isArray(raw?.components) ? raw.components.map((c: any) => ({
      type:   c?.type ?? '',
      fields: Array.isArray(c?.fields) ? c.fields.map((f: any) => String(f)) : [],
    })) : [],
    children: Array.isArray(raw?.children) ? raw.children.map(normalize) : [],
  };
}

import { getProjectRoot } from '../../../paths';
const PROJECT_ROOT = getProjectRoot();

async function resolveIdToUri(id: string): Promise<string | null> {
  const entries = await api.readManifest();
  const entry   = entries.find(e => String(e.id) === id.trim());
  return entry?.uri ?? null;
}

async function loadRaw(idOrPath: string): Promise<PrefabGameObjectRaw> {
  // idOrPath is either a numeric ID (new format) or a relative path (legacy)
  let relPath = idOrPath;
  if (/^\d+$/.test(idOrPath.trim())) {
    const uri = await resolveIdToUri(idOrPath);
    if (!uri) throw new Error(`No manifest entry for ID ${idOrPath}`);
    relPath = uri;
  }
  // relPath is relative to project root (e.g. "assets/compositions/test.comp")
  const fullPath = `${PROJECT_ROOT}/${relPath}`;
  const bytes    = await api.readFileBytes(fullPath);
  const text     = new TextDecoder('utf-8').decode(new Uint8Array(bytes));
  return normalize(yamlLoad(text));
}

// ─────────────────────────────────────────────────────────────────────────
// Field-level merge: start with base fields, apply child overrides
// ─────────────────────────────────────────────────────────────────────────

function mergeFields(
  baseFields:  string[],
  childFields: string[],
): ResolvedField[] {
  // Build a map of key -> raw string from the child's explicit overrides
  const childMap = new Map<string, string>();
  for (const f of childFields) {
    const [key] = splitField(f);
    childMap.set(key, f);
  }

  // Start with base fields, mark which ones are overridden
  const out: ResolvedField[] = baseFields.map(f => {
    const [key] = splitField(f);
    if (childMap.has(key)) {
      const overrideRaw = childMap.get(key)!;
      childMap.delete(key); // consume it
      return { raw: overrideRaw, overridden: true };
    }
    return { raw: f, overridden: false };
  });

  // Any remaining child fields not in base are appended as overrides
  for (const raw of childMap.values()) {
    out.push({ raw, overridden: true });
  }

  return out;
}

// ─────────────────────────────────────────────────────────────────────────
// Component-level merge
// ─────────────────────────────────────────────────────────────────────────

function mergeComponents(
  baseComps:  PrefabComponentRaw[],
  childComps: PrefabComponentRaw[],
): ResolvedComponent[] {
  const childMap = new Map<string, PrefabComponentRaw>();
  for (const c of childComps) childMap.set(c.type, c);

  const out: ResolvedComponent[] = baseComps.map(bc => {
    if (childMap.has(bc.type)) {
      const cc = childMap.get(bc.type)!;
      childMap.delete(bc.type);
      return {
        type:         bc.type,
        fields:       mergeFields(bc.fields, cc.fields),
        addedByChild: false,
      };
    }
    return {
      type:         bc.type,
      fields:       bc.fields.map(f => ({ raw: f, overridden: false })),
      addedByChild: false,
    };
  });

  // Components only in child → new additions
  for (const cc of childMap.values()) {
    out.push({
      type:         cc.type,
      fields:       cc.fields.map(f => ({ raw: f, overridden: true })),
      addedByChild: true,
    });
  }

  return out;
}

// ─────────────────────────────────────────────────────────────────────────
// GameObject-level merge (recursive over children)
// ─────────────────────────────────────────────────────────────────────────

function mergeGameObjects(
  base:  PrefabGameObjectRaw,
  child: PrefabGameObjectRaw,
): ResolvedGameObject {
  // Match children by name — first match wins
  const unmatchedBase = [...base.children];
  const resolvedChildren: ResolvedGameObject[] = [];

  for (const bc of unmatchedBase) {
    const childIdx = child.children.findIndex(cc => cc.name === bc.name);
    if (childIdx !== -1) {
      const cc = child.children[childIdx];
      resolvedChildren.push(mergeGameObjects(bc, cc));
    } else {
      // Base-only child — inherited, no overrides
      resolvedChildren.push(resolveNoBase(bc, false));
    }
  }

  // Children only in child prefab → added
  for (const cc of child.children) {
    const inBase = base.children.some(bc => bc.name === cc.name);
    if (!inBase) {
      resolvedChildren.push(resolveNoBase(cc, true));
    }
  }

  return {
    enabled:      child.enabled,
    name:         child.name,
    base:         child.base,
    addedByChild: false,
    components:   mergeComponents(base.components, child.components),
    children:     resolvedChildren,
  };
}

/** Resolve a node that has no base to merge with — all fields are either
 *  all-inherited (fromChild=false) or all-added (fromChild=true). */
function resolveNoBase(node: PrefabGameObjectRaw, fromChild: boolean): ResolvedGameObject {
  return {
    enabled:      node.enabled,
    name:         node.name,
    base:         node.base,
    addedByChild: fromChild,
    components:   node.components.map(c => ({
      type:         c.type,
      fields:       c.fields.map(f => ({ raw: f, overridden: fromChild })),
      addedByChild: fromChild,
    })),
    children: node.children.map(ch => resolveNoBase(ch, fromChild)),
  };
}

// ─────────────────────────────────────────────────────────────────────────
// Public entry point — resolves the full chain for a single node
// ─────────────────────────────────────────────────────────────────────────

/**
 * Resolves a prefab node by loading its `base` chain (A→B→C),
 * merging from the bottom up (C first, then B overrides C, then A overrides B).
 *
 * @param node        - The raw node to resolve (already parsed from YAML)
 * @param visitedPaths - Set of asset-relative paths already in the chain (cycle guard)
 * @param selfPath     - Asset-relative path of this node's file (for cycle detection)
 */
export async function resolveNode(
  node:         PrefabGameObjectRaw,
  selfPath:     string,
  visitedPaths: Set<string> = new Set(),
): Promise<ResolvedGameObject> {
  if (!node.base || node.base.trim() === '') {
    // No base — resolve children that may themselves have bases
    const children = await Promise.all(
      node.children.map(ch => resolveNode(ch, selfPath, visitedPaths))
    );
    return {
      ...resolveNoBase(node, true),
      children,
    };
  }

  const basePath = node.base.trim();

  // Cycle detection
  if (visitedPaths.has(basePath)) {
    console.error(`[PrefabResolver] cycle detected: ${basePath} already in chain [${[...visitedPaths].join(' → ')}]`);
    // Treat as if base is empty
    return resolveNode({ ...node, base: undefined }, selfPath, visitedPaths);
  }

  // Load and recursively resolve the base
  let baseRaw: PrefabGameObjectRaw;
  try {
    baseRaw = await loadRaw(basePath);
  } catch (e) {
    console.error(`[PrefabResolver] failed to load base "${basePath}":`, e);
    return resolveNode({ ...node, base: undefined }, selfPath, visitedPaths);
  }

  const nextVisited = new Set(visitedPaths).add(basePath);
  const resolvedBase = await resolveNode(baseRaw, basePath, nextVisited);

  // Merge child node over resolved base
  const merged = mergeResolvedWithChild(resolvedBase, node);

  // Now resolve the merged children's own bases
  const children = await Promise.all(
    merged.children.map(async ch => {
      // Find the corresponding raw child node to check for its own base
      const rawChild = node.children.find(c => c.name === ch.name);
      if (rawChild?.base) {
        return resolveNode(rawChild, selfPath, nextVisited);
      }
      return ch;
    })
  );

  return { ...merged, children };
}

/** Apply a raw child node's overrides onto an already-resolved base. */
function mergeResolvedWithChild(
  base:  ResolvedGameObject,
  child: PrefabGameObjectRaw,
): ResolvedGameObject {
  const childCompMap = new Map<string, PrefabComponentRaw>();
  for (const c of child.components) childCompMap.set(c.type, c);

  // Merge components
  const components: ResolvedComponent[] = base.components.map(bc => {
    if (childCompMap.has(bc.type)) {
      const cc = childCompMap.get(bc.type)!;
      childCompMap.delete(bc.type);
      // Merge fields: base resolved fields + child overrides
      const childFieldMap = new Map<string, string>();
      for (const f of cc.fields) {
        const [key] = splitField(f);
        childFieldMap.set(key, f);
      }
      const fields: ResolvedField[] = bc.fields.map(bf => {
        const [key] = splitField(bf.raw);
        if (childFieldMap.has(key)) {
          const raw = childFieldMap.get(key)!;
          childFieldMap.delete(key);
          return { raw, overridden: true };
        }
        return { ...bf }; // keep base override status
      });
      // Fields only in child
      for (const raw of childFieldMap.values()) {
        fields.push({ raw, overridden: true });
      }
      return { type: bc.type, fields, addedByChild: false };
    }
    return bc; // not touched by child
  });

  // Components only in child → new additions
  for (const cc of childCompMap.values()) {
    components.push({
      type:         cc.type,
      fields:       cc.fields.map(f => ({ raw: f, overridden: true })),
      addedByChild: true,
    });
  }

  // Merge children by name
  const unmatchedBase = [...base.children];
  const resolvedChildren: ResolvedGameObject[] = [];

  for (const bc of unmatchedBase) {
    const cc = child.children.find(c => c.name === bc.name);
    if (cc) {
      resolvedChildren.push(mergeResolvedWithChild(bc, cc));
    } else {
      resolvedChildren.push(bc);
    }
  }

  // Children only in child → added
  for (const cc of child.children) {
    const inBase = base.children.some(bc => bc.name === cc.name);
    if (!inBase) {
      resolvedChildren.push(resolveNoBase(cc, true));
    }
  }

  return {
    enabled:      child.enabled,
    name:         child.name,
    base:         child.base,
    addedByChild: false,
    components,
    children:     resolvedChildren,
  };
}

// ─────────────────────────────────────────────────────────────────────────
// Convert ResolvedGameObject back to a raw PrefabGameObjectRaw
// (only the child's own overrides — not the base data)
// Used by the inspector when reading current editable state.
// ─────────────────────────────────────────────────────────────────────────

export function resolvedToRaw(node: ResolvedGameObject): PrefabGameObjectRaw {
  return {
    enabled: node.enabled,
    name:    node.name,
    base:    node.base,
    components: node.components
      .filter(c => c.addedByChild || c.fields.some(f => f.overridden))
      .map(c => ({
        type:   c.type,
        fields: c.fields.filter(f => f.overridden).map(f => f.raw),
      })),
    children: node.children.map(resolvedToRaw),
  };
}

/**
 * Converts a ResolvedGameObject to a full PrefabGameObjectRaw where every
 * field (inherited or overridden) is included. Used by the viewport to render
 * the fully merged scene without needing to know override status.
 */
export function resolvedToRawFull(node: ResolvedGameObject): PrefabGameObjectRaw {
  return {
    enabled:    node.enabled,
    name:       node.name,
    base:       node.base,
    components: node.components.map(c => ({
      type:   c.type,
      fields: c.fields.map(f => f.raw),
    })),
    children: node.children.map(resolvedToRawFull),
  };
}
