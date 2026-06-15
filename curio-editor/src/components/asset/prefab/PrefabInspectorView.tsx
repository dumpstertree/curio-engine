import React, { useState } from 'react';
import type { PrefabGameObjectRaw, PrefabComponentRaw, KnownComponentType } from './prefabTypes';
import {
  COMPONENT_TYPES,
  isTransform,
  isRenderer,
  readTransformFields,
  writeTransformFields,
  readRendererAsset,
  writeRendererAsset,
  splitField,
  joinField,
  defaultComponent,
  defaultGameObject,
} from './prefabTypes';
import { Vec3Input } from './VectorInput';

// ─────────────────────────────────────────────────────────────────────────
// Generic key:value field row (for unknown component types)
// ─────────────────────────────────────────────────────────────────────────

function GenericFieldRow({ field, onChange }: { field: string; onChange: (next: string) => void }) {
  const [key, val] = splitField(field);
  const [text, setText] = useState(val);

  return (
    <div className="field-row" style={{ paddingLeft: 22 }}>
      <span className="field-key">{key}</span>
      <input
        className="field-val-input"
        type="text"
        value={text}
        onChange={e => setText(e.target.value)}
        onBlur={() => onChange(joinField(key, text))}
        onKeyDown={e => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
      />
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────
// Transform component (transform2d / transform3d)
// ─────────────────────────────────────────────────────────────────────────

function TransformFieldsBlock({ comp, onChange }: { comp: PrefabComponentRaw; onChange: (next: PrefabComponentRaw) => void }) {
  const t     = readTransformFields(comp);
  const is2d  = comp.type === 'transform2d';

  return (
    <div className="comp-fields-list">
      <div className="vec-group">
        <span className="vec-group-label">position</span>
        <Vec3Input is2d={is2d} value={t.position} onChange={position => onChange(writeTransformFields(comp, { ...t, position }))} />
      </div>
      <div className="vec-group">
        <span className="vec-group-label">rotation</span>
        <Vec3Input value={t.rotation} onChange={rotation => onChange(writeTransformFields(comp, { ...t, rotation }))} />
      </div>
      <div className="vec-group">
        <span className="vec-group-label">scale</span>
        <Vec3Input is2d={is2d} value={t.scale} onChange={scale => onChange(writeTransformFields(comp, { ...t, scale }))} />
      </div>
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────
// Renderer component (RendererStatic / RendererDynamic)
// ─────────────────────────────────────────────────────────────────────────

function RendererFieldsBlock({ comp, onChange }: { comp: PrefabComponentRaw; onChange: (next: PrefabComponentRaw) => void }) {
  const asset = readRendererAsset(comp) ?? '';
  const [text, setText] = useState(asset);
  const expectedExt = comp.type === 'RendererDynamic' ? '.anim' : '.glb';

  return (
    <div className="comp-fields-list">
      <div className="field-row" style={{ paddingLeft: 22 }}>
        <span className="field-key">asset</span>
        <input
          className="field-val-input"
          type="text"
          placeholder={`mesh/example${expectedExt}`}
          value={text}
          onChange={e => setText(e.target.value)}
          onBlur={() => onChange(writeRendererAsset(comp, text))}
          onKeyDown={e => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
        />
      </div>
      <div className="field-row" style={{ paddingLeft: 22 }}>
        <span className="field-hint">expects {expectedExt}, relative to assets/</span>
      </div>
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────
// Component block — collapsible, with remove button
// ─────────────────────────────────────────────────────────────────────────

interface ComponentBlockProps {
  comp:     PrefabComponentRaw;
  onChange: (next: PrefabComponentRaw) => void;
  onRemove: () => void;
}

function ComponentBlock({ comp, onChange, onRemove }: ComponentBlockProps) {
  const [open, setOpen] = useState(true);

  return (
    <div className="comp-block">
      <div className="comp-header" onClick={() => setOpen(o => !o)}>
        <span className={`comp-chevron ${open ? 'expanded' : ''}`}>
          <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor">
            <polygon points="2,1 7,4 2,7" />
          </svg>
        </span>
        <span className="comp-name">{comp.type}</span>
        <button
          className="comp-remove-btn"
          onClick={e => { e.stopPropagation(); onRemove(); }}
          title="Remove component"
        >
          <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" strokeWidth="1.4">
            <line x1="1" y1="1" x2="8" y2="8" />
            <line x1="8" y1="1" x2="1" y2="8" />
          </svg>
        </button>
      </div>

      {open && (
        isTransform(comp.type) ? <TransformFieldsBlock comp={comp} onChange={onChange} />
        : isRenderer(comp.type) ? <RendererFieldsBlock comp={comp} onChange={onChange} />
        : (
          <div className="comp-fields-list">
            {comp.fields.length === 0 ? (
              <span className="field-empty" style={{ paddingLeft: 22 }}>no fields</span>
            ) : (
              comp.fields.map((f, i) => (
                <GenericFieldRow
                  key={i}
                  field={f}
                  onChange={next => {
                    const fields = [...comp.fields];
                    fields[i] = next;
                    onChange({ ...comp, fields });
                  }}
                />
              ))
            )}
          </div>
        )
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────
// Add-component dropdown
// ─────────────────────────────────────────────────────────────────────────

function AddComponentButton({ onAdd }: { onAdd: (type: KnownComponentType) => void }) {
  const [open, setOpen] = useState(false);

  return (
    <div className="add-component-wrap">
      <button className="add-component-btn" onClick={() => setOpen(o => !o)}>
        + Add Component
      </button>
      {open && (
        <div className="add-component-menu">
          {COMPONENT_TYPES.map(type => (
            <div
              key={type}
              className="add-component-item"
              onClick={() => { onAdd(type); setOpen(false); }}
            >
              {type}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────
// GameObject node — recursive
// ─────────────────────────────────────────────────────────────────────────

interface GameObjectNodeProps {
  node:      PrefabGameObjectRaw;
  onChange:  (next: PrefabGameObjectRaw) => void;
  onRemove?: () => void; // absent for the root (cannot remove root)
  depth:     number;
}

function GameObjectNode({ node, onChange, onRemove, depth }: GameObjectNodeProps) {
  const [open, setOpen] = useState(true);
  const [name, setName] = useState(node.name);

  function updateComponent(i: number, next: PrefabComponentRaw) {
    const components = [...node.components];
    components[i] = next;
    onChange({ ...node, components });
  }
  function removeComponent(i: number) {
    const components = node.components.filter((_, idx) => idx !== i);
    onChange({ ...node, components });
  }
  function addComponent(type: KnownComponentType) {
    onChange({ ...node, components: [...node.components, defaultComponent(type)] });
  }

  function updateChild(i: number, next: PrefabGameObjectRaw) {
    const children = [...node.children];
    children[i] = next;
    onChange({ ...node, children });
  }
  function removeChild(i: number) {
    const children = node.children.filter((_, idx) => idx !== i);
    onChange({ ...node, children });
  }
  function addChild() {
    onChange({ ...node, children: [...node.children, defaultGameObject()] });
  }

  return (
    <div className="gobj-node" style={{ marginLeft: depth > 0 ? 12 : 0 }}>
      <div className="gobj-header">
        <button className="gobj-chevron-btn" onClick={() => setOpen(o => !o)}>
          <svg
            width="8" height="8" viewBox="0 0 8 8" fill="currentColor"
            style={{ transform: open ? 'rotate(90deg)' : 'none', transition: 'transform .12s' }}
          >
            <polygon points="2,1 7,4 2,7" />
          </svg>
        </button>

        <input
          type="checkbox"
          className="gobj-enabled-check"
          checked={node.enabled}
          onChange={e => onChange({ ...node, enabled: e.target.checked })}
          title="enabled"
        />

        <input
          className="gobj-name-input"
          type="text"
          value={name}
          onChange={e => setName(e.target.value)}
          onBlur={() => onChange({ ...node, name })}
          onKeyDown={e => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
        />

        {onRemove && (
          <button className="comp-remove-btn" onClick={onRemove} title="Remove GameObject">
            <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" strokeWidth="1.4">
              <line x1="1" y1="1" x2="8" y2="8" />
              <line x1="8" y1="1" x2="1" y2="8" />
            </svg>
          </button>
        )}
      </div>

      {open && (
        <div className="gobj-body">
          {node.components.map((comp, i) => (
            <ComponentBlock
              key={i}
              comp={comp}
              onChange={next => updateComponent(i, next)}
              onRemove={() => removeComponent(i)}
            />
          ))}

          <AddComponentButton onAdd={addComponent} />

          {node.children.length > 0 && (
            <div className="gobj-children-label">Children ({node.children.length})</div>
          )}
          {node.children.map((child, i) => (
            <GameObjectNode
              key={i}
              node={child}
              depth={depth + 1}
              onChange={next => updateChild(i, next)}
              onRemove={() => removeChild(i)}
            />
          ))}

          <button className="add-child-btn" onClick={addChild}>+ Add Child</button>
        </div>
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────────────────
// Top-level inspector panel
// ─────────────────────────────────────────────────────────────────────────

interface Props {
  fileName: string | null;
  root:     PrefabGameObjectRaw | null;
  onChange: (next: PrefabGameObjectRaw) => void;
}

export function PrefabInspectorView({ fileName, root, onChange }: Props) {
  return (
    <div className="inspector-view">
      <div className="panel-header">
        <span className="panel-title">Inspector</span>
      </div>

      {!fileName || !root ? (
        <div className="panel-empty">Select a prefab</div>
      ) : (
        <>
          <div className="inspector-header">
            <div className="inspector-subject-name">{fileName}</div>
            <div className="inspector-subject-meta">Prefab</div>
          </div>

          <div className="inspector-content prefab-inspector-content">
            <GameObjectNode node={root} depth={0} onChange={onChange} />
          </div>
        </>
      )}
    </div>
  );
}
