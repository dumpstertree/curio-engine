import React, { useState, useEffect, useRef } from 'react';
import type { PrefabGameObjectRaw, PrefabComponentRaw, KnownComponentType } from './prefabTypes';
import {
  COMPONENT_TYPES,
  COMPONENT_FIELDS,
  isTransform,
  isRenderer,
  splitField,
  joinField,
  parseTuple,
  formatTuple,
  defaultComponent,
  defaultGameObject,
} from './prefabTypes';

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

  function commit() {
    if (draft.trim() !== '') onSet(joinField(fieldKey, draft));
  }

  return (
    <div className={`pf-field-row ${!isSet ? 'field-inherited' : ''}`}>
      <span className="field-key">{fieldKey}</span>

      {isSet ? (
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
            {(['x','y','z'] as const).filter((_, i) => !(is2d && fieldKey !== 'rotation' && i === 2)).map(axis => (
              <label key={axis} className="vec-axis">
                <span className="vec-axis-label">{axis.toUpperCase()}</span>
                <input
                  className="vec-axis-input"
                  type="number"
                  step="0.1"
                  value={parsed[axis]}
                  onChange={e => {
                    const n = parseFloat(e.target.value);
                    commitVec({ ...parsed, [axis]: Number.isFinite(n) ? n : 0 });
                  }}
                />
              </label>
            ))}
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

// ─── Component block ──────────────────────────────────────────────────────────

interface ComponentBlockProps {
  comp:     PrefabComponentRaw;
  onChange: (next: PrefabComponentRaw) => void;
  onRemove: () => void;
}

function ComponentBlock({ comp, onChange, onRemove }: ComponentBlockProps) {
  const [open, setOpen] = useState(true);
  const is2d = comp.type === 'transform2d';

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

  // Known field keys for this component type
  const knownKeys: string[] = COMPONENT_FIELDS[comp.type as KnownComponentType] ?? [];
  // Any extra fields in the .comp that aren't in the known list
  const extraKeys = comp.fields
    .map(f => splitField(f)[0])
    .filter(k => !knownKeys.includes(k));
  const allKeys = [...knownKeys, ...extraKeys];

  return (
    <div className="comp-block">
      <div className="comp-header" onClick={() => setOpen(o => !o)}>
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
          {allKeys.map(key => {
            const val = getValue(key);
            if (isTransform(comp.type) && (key === 'position' || key === 'rotation' || key === 'scale')) {
              return (
                <TransformRow
                  key={key}
                  fieldKey={key}
                  value={val}
                  is2d={is2d}
                  onSet={raw => setField(key, raw)}
                  onRemove={() => removeField(key)}
                />
              );
            }
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
  );
}

// ─── Add component dropdown ───────────────────────────────────────────────────

function AddComponentButton({ onAdd }: { onAdd: (type: KnownComponentType) => void }) {
  const [open, setOpen] = useState(false);
  return (
    <div className="add-component-wrap">
      <button className="add-component-btn" onClick={() => setOpen(o => !o)}>+ Add Component</button>
      {open && (
        <div className="add-component-menu">
          {COMPONENT_TYPES.map(type => (
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
  const [base, setBase] = useState(node.base ?? '');

  const isSelected = selectedPath !== null && JSON.stringify(path) === JSON.stringify(selectedPath);

  // Auto-open and scroll into view when selected
  const nodeRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (isSelected) {
      setOpen(true);
      nodeRef.current?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  }, [isSelected]);

  function updateComp(i: number, next: PrefabComponentRaw) {
    const components = [...node.components];
    components[i] = next;
    onChange({ ...node, components });
  }
  function removeComp(i: number) {
    onChange({ ...node, components: node.components.filter((_, idx) => idx !== i) });
  }
  function addComp(type: KnownComponentType) {
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
          {/* base path */}
          <div className="gobj-base-row">
            <span className="gobj-base-label">base</span>
            <input className="field-val-input gobj-base-input" type="text"
              placeholder="path/to/base.comp (optional)"
              value={base}
              onChange={e => setBase(e.target.value)}
              onBlur={() => onChange({ ...node, base: base.trim() || undefined })}
              onKeyDown={e => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }} />
          </div>

          {node.components.map((comp, i) => (
            <ComponentBlock key={comp.type + i} comp={comp}
              onChange={next => updateComp(i, next)}
              onRemove={() => removeComp(i)} />
          ))}

          <AddComponentButton onAdd={addComp} />

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
  return (
    <div className="inspector-view">
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
