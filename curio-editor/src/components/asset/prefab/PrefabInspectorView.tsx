import React, { useState, useEffect, useRef } from 'react';
import type { PrefabGameObjectRaw, PrefabComponentRaw } from './prefabTypes';
import {
  BUILTIN_COMPONENT_FIELDS,
  COMPONENT_TYPES,
  FACET_FIELDS,
  loadFacets,
  getComponentFields,
  isTransform,
  isRenderer,
  splitField,
  joinField,
  parseTuple,
  formatTuple,
  defaultComponent,
  defaultGameObject,
  getNodeAtPath,
  setNodeAtPath,
  type FieldDescriptor,
  type EntryType,
} from './prefabTypes';
import { AssetDropdown } from './AssetDropdown';

// ─── Edit / Discard buttons ───────────────────────────────────────────────────

function EditBtn({ onClick }: { onClick: () => void }) {
  return (
    <button className="field-edit-btn" onClick={onClick} title="Set this field">
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.3">
        <path d="M7 1.5l1.5 1.5L3 8.5H1.5V7L7 1.5z"/>
      </svg>
    </button>
  );
}

function DiscardBtn({ onClick }: { onClick: () => void }) {
  return (
    <button className="field-discard-btn" onClick={onClick} title="Remove this field">
      <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" strokeWidth="1.4">
        <line x1="1" y1="1" x2="8" y2="8"/>
        <line x1="8" y1="1" x2="1" y2="8"/>
      </svg>
    </button>
  );
}

// ─── Generic field row ────────────────────────────────────────────────────────

interface FieldRowProps {
  fieldKey:  string;
  value:     string | null;   // null = not set in .comp
  onSet:     (raw: string) => void;
  onRemove:  () => void;
}

function FieldRow({ fieldKey, value, onSet, onRemove }: FieldRowProps) {
  const isSet = value !== null;
  const [draft, setDraft] = useState(value ?? '');

  // Sync draft if value changes externally
  useEffect(() => { setDraft(value ?? ''); }, [value]);

  // Non-primitive JSON values (objects/arrays) are readonly
  const isNonPrimitive = (() => {
    if (!value) return false;
    try {
      const parsed = JSON.parse(value);
      return typeof parsed === 'object' && parsed !== null;
    } catch { return false; }
  })();

  function commit() {
    if (draft.trim() !== '') onSet(joinField(fieldKey, draft));
  }

  return (
    <div className={`pf-field-row ${!isSet ? 'field-inherited' : ''}`}>
      <span className="field-key">{fieldKey}</span>

      {isSet ? (
        isNonPrimitive ? (
          <>
            <span className="field-val-readonly field-val-complex">{value}</span>
            <DiscardBtn onClick={onRemove} />
          </>
        ) : (
          <>
            <input
              className="field-val-input"
              type="text"
              value={draft}
              onChange={e => setDraft(e.target.value)}
              onBlur={commit}
              onKeyDown={e => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
            />
            <DiscardBtn onClick={onRemove} />
          </>
        )
      ) : (
        <>
          <span className="field-val-readonly"><em className="field-val-empty">—</em></span>
          <EditBtn onClick={() => { setDraft(''); onSet(joinField(fieldKey, '')); }} />
        </>
      )}
    </div>
  );
}

// ─── Transform field row (position / rotation / scale with Vec3 inputs) ───────

interface TransformRowProps {
  fieldKey:  string;
  value:     string | null;
  is2d:      boolean;
  onSet:     (raw: string) => void;
  onRemove:  () => void;
}

function AxisInput({ axis, value, onChange }: { axis: string; value: number; onChange: (n: number) => void }) {
  const [draft, setDraft] = useState(String(value));

  // Sync when value changes externally (e.g. gizmo update)
  useEffect(() => { setDraft(String(value)); }, [value]);

  function commit() {
    const n = parseFloat(draft);
    if (Number.isFinite(n)) onChange(n);
    else setDraft(String(value)); // revert
  }

  return (
    <label className="vec-axis">
      <span className="vec-axis-label">{axis.toUpperCase()}</span>
      <input
        className="vec-axis-input"
        type="text"
        inputMode="decimal"
        value={draft}
        onChange={e => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={e => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
      />
    </label>
  );
}

function TransformRow({ fieldKey, value, is2d, onSet, onRemove }: TransformRowProps) {
  const isSet = value !== null;

  const parsed = value
    ? (() => { const t = parseTuple(value); return { x: t[0]??0, y: t[1]??0, z: t[2]??0 }; })()
    : { x: 0, y: 0, z: 0 };

  function commitVec(v: { x: number; y: number; z: number }) {
    const useXY = is2d && fieldKey !== 'rotation';
    const str = useXY ? formatTuple([v.x, v.y]) : formatTuple([v.x, v.y, v.z]);
    onSet(joinField(fieldKey, str));
  }

  return (
    <div className={`pf-transform-row ${!isSet ? 'field-inherited' : ''}`}>
      <span className="vec-group-label">{fieldKey}</span>

      {isSet ? (
        <div className="pf-transform-edit-row">
          <div className="vec3-input-wrap">
            {(['x','y','z'] as const)
              .filter((_, i) => !(is2d && fieldKey !== 'rotation' && i === 2))
              .map(axis => (
                <AxisInput
                  key={axis}
                  axis={axis}
                  value={parsed[axis as 'x'|'y'|'z']}
                  onChange={n => commitVec({ ...parsed, [axis]: n })}
                />
              ))
            }
          </div>
          <DiscardBtn onClick={onRemove} />
        </div>
      ) : (
        <div className="pf-transform-readonly-row">
          <span className="field-val-readonly"><em className="field-val-empty">—</em></span>
          <EditBtn onClick={() => commitVec(parsed)} />
        </div>
      )}
    </div>
  );
}

// ─── Renderer asset row ───────────────────────────────────────────────────────

function RendererAssetRow({ val, isSet, accepts, onSet, onRemove }: {
  val:      string | null;
  isSet:    boolean;
  accepts:  string[];
  onSet:    (id: string) => void;
  onRemove: () => void;
}) {
  const [editing, setEditing] = useState(isSet);

  // If a value gets set from outside (e.g. initial load), show as editing
  useEffect(() => { if (isSet) setEditing(true); }, [isSet]);

  return (
    <div className={`pf-field-row ${!isSet ? 'field-inherited' : ''}`}>
      <span className="field-key">asset</span>
      {editing ? (
        <>
          <AssetDropdown
            value={val}
            accepts={accepts}
            onChange={id => {
              if (id === null) { onRemove(); setEditing(false); }
              else onSet(id);
            }}
          />
          <DiscardBtn onClick={() => { onRemove(); setEditing(false); }} />
        </>
      ) : (
        <>
          <span className="field-val-readonly"><em className="field-val-empty">—</em></span>
          <EditBtn onClick={() => setEditing(true)} />
        </>
      )}
    </div>
  );
}

// ─── Component block ──────────────────────────────────────────────────────────

interface ComponentBlockProps {
  comp:          PrefabComponentRaw;
  index:         number;
  onChange:      (next: PrefabComponentRaw) => void;
  onRemove:      () => void;
  onPointerDown: (e: React.PointerEvent, i: number) => void;
  onPointerMove: (e: React.PointerEvent) => void;
  onPointerUp:   (e: React.PointerEvent) => void;
  isDragOver:    boolean;
}

function ComponentBlock({ comp, index, onChange, onRemove, onPointerDown, onPointerMove, onPointerUp, isDragOver }: ComponentBlockProps) {
  const [open, setOpen] = useState(true);
  const is2d = comp.type === 'Transform2D';

  // Get current value for a field key (null if not set)
  function getValue(key: string): string | null {
    const f = comp.fields.find(f => splitField(f)[0] === key);
    return f !== undefined ? splitField(f)[1] : null;
  }

  function setField(key: string, raw: string) {
    const fields = comp.fields.filter(f => splitField(f)[0] !== key);
    fields.push(raw);
    onChange({ ...comp, fields });
  }

  function removeField(key: string) {
    onChange({ ...comp, fields: comp.fields.filter(f => splitField(f)[0] !== key) });
  }

  // Known field descriptors from facets (or builtins)
  const knownFields: FieldDescriptor[] = getComponentFields(comp.type);
  const knownKeys = knownFields.map(f => f.name);
  // Extra fields in .comp not in known list — treat as generic Float
  const extraKeys = comp.fields
    .map(f => splitField(f)[0])
    .filter(k => !knownKeys.includes(k));
  const allFields: FieldDescriptor[] = [
    ...knownFields,
    ...extraKeys.map(k => ({ name: k, type: 'Float' as EntryType })),
  ];

  return (
    <>
      {isDragOver && <div className="comp-drop-line" />}
      <div className="comp-block" data-comp-index={index}>
        <div className="comp-header" onClick={() => setOpen(o => !o)}>
          {/* Drag handle — all pointer events here since capture routes to this element */}
          <span
            className="comp-drag-handle"
            onPointerDown={e => { e.stopPropagation(); onPointerDown(e, index); }}
            onPointerMove={e => { e.stopPropagation(); onPointerMove(e); }}
            onPointerUp={e => { e.stopPropagation(); onPointerUp(e); }}
          >
            <svg width="8" height="12" viewBox="0 0 8 12" fill="currentColor" opacity="0.4">
              <circle cx="2" cy="2"  r="1.2"/><circle cx="6" cy="2"  r="1.2"/>
              <circle cx="2" cy="6"  r="1.2"/><circle cx="6" cy="6"  r="1.2"/>
              <circle cx="2" cy="10" r="1.2"/><circle cx="6" cy="10" r="1.2"/>
            </svg>
          </span>
          <span className={`comp-chevron ${open ? 'expanded' : ''}`}>
            <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor">
              <polygon points="2,1 7,4 2,7"/>
            </svg>
          </span>
          <span className="comp-name">{comp.type}</span>
          <button className="comp-remove-btn"
            onClick={e => { e.stopPropagation(); onRemove(); }}>
            <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" strokeWidth="1.4">
              <line x1="1" y1="1" x2="8" y2="8"/><line x1="8" y1="1" x2="1" y2="8"/>
            </svg>
          </button>
        </div>

        {open && (
          <div className="comp-fields-list">
            {allFields.map(({ name: key, type: fieldType }) => {
              const val = getValue(key);

            // Vector types (including transform fields)
            const vecAxes = fieldType === 'Vector2' ? ['X','Y']
                          : fieldType === 'Vector3' ? ['X','Y','Z']
                          : fieldType === 'Vector4' ? ['X','Y','Z','W']
                          : null;
            if (vecAxes) {
              const is2dField = fieldType === 'Vector2';
              return (
                <TransformRow
                  key={key}
                  fieldKey={key}
                  value={val}
                  is2d={is2dField}
                  onSet={raw => setField(key, raw)}
                  onRemove={() => removeField(key)}
                />
              );
            }

            // Asset dropdown
            if (typeof fieldType === 'object' && 'Asset' in fieldType) {
              const suffix = fieldType.Asset;
              const isSet  = val !== null && val.trim() !== '';
              return (
                <RendererAssetRow
                  key={key}
                  val={val}
                  isSet={isSet}
                  accepts={[suffix]}
                  onSet={id => setField(key, joinField(key, id))}
                  onRemove={() => removeField(key)}
                />
              );
            }

            // Bool → checkbox
            if (fieldType === 'Bool') {
              const isSet   = val !== null;
              const checked = val === 'true';
              return (
                <div key={key} className={`pf-field-row ${!isSet ? 'field-inherited' : ''}`}>
                  <span className="field-key">{key}</span>
                  {isSet ? (
                    <>
                      <input
                        type="checkbox"
                        className="field-bool-check"
                        checked={checked}
                        onChange={e => setField(key, joinField(key, String(e.target.checked)))}
                      />
                      <DiscardBtn onClick={() => removeField(key)} />
                    </>
                  ) : (
                    <>
                      <span className="field-val-readonly"><em className="field-val-empty">—</em></span>
                      <EditBtn onClick={() => setField(key, joinField(key, 'false'))} />
                    </>
                  )}
                </div>
              );
            }

            // Float / Int / generic → text field
            return (
              <FieldRow
                key={key}
                fieldKey={key}
                value={val}
                onSet={raw => setField(key, raw)}
                onRemove={() => removeField(key)}
              />
            );
          })}
          </div>
        )}
      </div>
    </>
  );
}

// ─── Component drag reorder (pointer-based, not HTML5 drag) ──────────────────

function useDragReorder(
  onReorder: (from: number, to: number) => void,
) {
  const fromIdx = useRef<number | null>(null);
  const [overIdx, setOverIdx] = useState<number | null>(null);

  function startDrag(e: React.PointerEvent, idx: number) {
    e.preventDefault();
    fromIdx.current = idx;
    setOverIdx(idx);
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }

  function moveDrag(e: React.PointerEvent) {
    if (fromIdx.current === null) return;
    // Find which comp-block is under the cursor
    const el = document.elementFromPoint(e.clientX, e.clientY);
    const block = el?.closest('[data-comp-index]') as HTMLElement | null;
    if (block) {
      const idx = parseInt(block.dataset.compIndex ?? '-1', 10);
      if (idx >= 0) setOverIdx(idx);
    }
  }

  function endDrag(e: React.PointerEvent) {
    const from = fromIdx.current;
    fromIdx.current = null;
    // Final position from elementFromPoint
    const el    = document.elementFromPoint(e.clientX, e.clientY);
    const block = el?.closest('[data-comp-index]') as HTMLElement | null;
    const to    = block ? parseInt(block.dataset.compIndex ?? '-1', 10) : -1;
    setOverIdx(null);
    if (from !== null && to >= 0 && from !== to) onReorder(from, to);
  }

  return { overIdx, startDrag, moveDrag, endDrag };
}

function AddComponentButton({ onAdd, existingTypes }: { onAdd: (type: string) => void; existingTypes: string[] }) {
  const [open,    setOpen]    = useState(false);
  const [types,   setTypes]   = useState<string[]>(COMPONENT_TYPES);
  const [loading, setLoading] = useState(false);

  async function handleOpen() {
    setOpen(o => !o);
    if (COMPONENT_TYPES.length === 0) {
      setLoading(true);
      await loadFacets();
    }
    // Filter out types already on this GameObject
    setTypes(COMPONENT_TYPES.filter(t => !existingTypes.includes(t)));
    setLoading(false);
  }

  return (
    <div className="add-component-wrap">
      <button className="add-component-btn" onClick={handleOpen}>+ Add Facet</button>
      {open && (
        <div className="add-component-menu">
          {loading && <div className="add-component-item" style={{ color: 'var(--text-muted)' }}>Loading…</div>}
          {!loading && types.length === 0 && (
            <div className="add-component-item" style={{ color: 'var(--text-muted)' }}>All facets present</div>
          )}
          {!loading && types.map(type => (
            <div key={type} className="add-component-item"
              onClick={() => { onAdd(type); setOpen(false); }}>
              {type}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

// ─── GameObject node (recursive, raw only) ────────────────────────────────────

interface GameObjectNodeProps {
  node:         PrefabGameObjectRaw;
  onChange:     (next: PrefabGameObjectRaw) => void;
  onRemove?:    () => void;
  depth:        number;
  path:         number[];        // this node's path in the raw tree
  selectedPath: number[] | null; // currently selected path
}

function GameObjectNode({ node, onChange, onRemove, depth, path, selectedPath }: GameObjectNodeProps) {
  const [open, setOpen] = useState(true);
  const [name, setName] = useState(node.name);

  const isSelected = selectedPath !== null && JSON.stringify(path) === JSON.stringify(selectedPath);

  // Auto-open and scroll into view when selected
  const nodeRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (isSelected) {
      setOpen(true);
      nodeRef.current?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  }, [isSelected]);

  const { overIdx: dragOver, startDrag, moveDrag, endDrag } = useDragReorder(
    (from, to) => {
      const comps = [...node.components];
      const [moved] = comps.splice(from, 1);
      comps.splice(to, 0, moved);
      onChange({ ...node, components: comps });
    }
  );

  function updateComp(i: number, next: PrefabComponentRaw) {
    const components = [...node.components];
    components[i] = next;
    onChange({ ...node, components });
  }
  function removeComp(i: number) {
    onChange({ ...node, components: node.components.filter((_, idx) => idx !== i) });
  }
  function addComp(type: string) {
    onChange({ ...node, components: [...node.components, defaultComponent(type)] });
  }
  function updateChild(i: number, next: PrefabGameObjectRaw) {
    const children = [...node.children];
    children[i] = next;
    onChange({ ...node, children });
  }
  function removeChild(i: number) {
    onChange({ ...node, children: node.children.filter((_, idx) => idx !== i) });
  }
  function addChild() {
    onChange({ ...node, children: [...node.children, defaultGameObject()] });
  }

  return (
    <div ref={nodeRef} className="gobj-node" style={{ marginLeft: depth > 0 ? 12 : 0 }}>
      <div className={`gobj-header ${isSelected ? 'gobj-header-selected' : ''}`}>
        <button className="gobj-chevron-btn" onClick={() => setOpen(o => !o)}>
          <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor"
            style={{ transform: open ? 'rotate(90deg)' : 'none', transition: 'transform .12s' }}>
            <polygon points="2,1 7,4 2,7"/>
          </svg>
        </button>
        <input type="checkbox" className="gobj-enabled-check" checked={node.enabled}
          onChange={e => onChange({ ...node, enabled: e.target.checked })} />
        <input className="gobj-name-input" type="text" value={name}
          onChange={e => setName(e.target.value)}
          onBlur={() => onChange({ ...node, name })}
          onKeyDown={e => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }} />
        {onRemove && (
          <button className="comp-remove-btn" onClick={onRemove}>
            <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" strokeWidth="1.4">
              <line x1="1" y1="1" x2="8" y2="8"/><line x1="8" y1="1" x2="1" y2="8"/>
            </svg>
          </button>
        )}
      </div>

      {open && (
        <div className="gobj-body">
          {/* base — dropdown of .comp assets */}
          <div className="gobj-base-row">
            <span className="gobj-base-label">base</span>
            <AssetDropdown
              value={node.base ?? null}
              accepts={['.comp']}
              onChange={id => onChange({ ...node, base: id ?? undefined })}
              placeholder="— no base —"
            />
          </div>

          {node.components.map((comp, i) => (
            <ComponentBlock key={comp.type + i} comp={comp} index={i}
              onChange={next => updateComp(i, next)}
              onRemove={() => removeComp(i)}
              onPointerDown={startDrag}
              onPointerMove={moveDrag}
              onPointerUp={endDrag}
              isDragOver={dragOver === i}
            />
          ))}

          <AddComponentButton
            onAdd={addComp}
            existingTypes={node.components.map(c => c.type)}
          />

          {node.children.length > 0 && (
            <div className="gobj-children-label">Children ({node.children.length})</div>
          )}
          {node.children.map((child, i) => (
            <GameObjectNode key={child.name + i} node={child} depth={depth + 1}
              path={[...path, i]}
              selectedPath={selectedPath}
              onChange={next => updateChild(i, next)}
              onRemove={() => removeChild(i)} />
          ))}
          <button className="add-child-btn" onClick={addChild}>+ Add Child</button>
        </div>
      )}
    </div>
  );
}

// ─── Top-level inspector ──────────────────────────────────────────────────────

interface Props {
  fileName:     string | null;
  raw:          PrefabGameObjectRaw | null;
  selectedPath: number[] | null;
  onChange:     (next: PrefabGameObjectRaw) => void;
  onRefresh:    () => void;
}

export function PrefabInspectorView({ fileName, raw, selectedPath, onChange, onRefresh }: Props) {
  const panelRef = useRef<HTMLDivElement>(null);

  // Load facets eagerly so field types are known before user opens any dropdown
  useEffect(() => {
    if (COMPONENT_TYPES.length === 0) loadFacets();
  }, []);

  // Ctrl+D: duplicate selected node; Delete: remove selected node
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      // Only fire when focus is inside our panel
      if (!panelRef.current?.contains(document.activeElement)) return;
      if (!raw || !selectedPath || selectedPath.length === 0) return;

      if (e.key === 'Delete') {
        e.preventDefault();
        const parentPath = selectedPath.slice(0, -1);
        const idx = selectedPath[selectedPath.length - 1];
        const parent = getNodeAtPath(raw, parentPath);
        if (!parent) return;
        const children = parent.children.filter((_, i) => i !== idx);
        onChange(setNodeAtPath(raw, parentPath, { ...parent, children }));
      }

      if ((e.ctrlKey || e.metaKey) && e.key === 'd') {
        e.preventDefault();
        const parentPath = selectedPath.slice(0, -1);
        const idx = selectedPath[selectedPath.length - 1];
        const parent = getNodeAtPath(raw, parentPath);
        if (!parent) return;
        const node = parent.children[idx];
        if (!node) return;
        const duplicate = { ...node, name: `${node.name}_1` };
        const children  = [
          ...parent.children.slice(0, idx + 1),
          duplicate,
          ...parent.children.slice(idx + 1),
        ];
        onChange(setNodeAtPath(raw, parentPath, { ...parent, children }));
      }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [raw, selectedPath, onChange]);

  return (
    <div className="inspector-view" ref={panelRef} tabIndex={-1}>
      <div className="panel-header">
        <span className="panel-title">Inspector</span>
        {fileName && (
          <button className="prefab-refresh-btn" onClick={onRefresh} title="Re-resolve from disk">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.4">
              <path d="M10 6a4 4 0 1 1-1.17-2.83"/>
              <polyline points="7,1 10.5,3.5 8,6.5"/>
            </svg>
            Refresh
          </button>
        )}
      </div>

      {!fileName || !raw ? (
        <div className="panel-empty">Select a prefab</div>
      ) : (
        <>
          <div className="inspector-header">
            <div className="inspector-subject-name">{fileName}</div>
            <div className="inspector-subject-meta">
              Prefab{raw.base ? ` · base: ${raw.base}` : ''}
            </div>
          </div>
          <div className="inspector-content prefab-inspector-content">
            <GameObjectNode node={raw} depth={0} path={[]} selectedPath={selectedPath} onChange={onChange} />
          </div>
        </>
      )}
    </div>
  );
}
