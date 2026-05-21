import React, { useState } from 'react';
import { useEditorStore } from '../../store';
import type { ComponentState, FieldState } from '../../types';

// ─────────────────────────────────────────────────────────────
// Field rendering — handles nested objects recursively
// ─────────────────────────────────────────────────────────────

function formatPrimitive(value: unknown): { text: string; cls: string } {
  if (value === null || value === undefined) return { text: 'null',        cls: 'fv-null' };
  if (typeof value === 'boolean')            return { text: String(value), cls: value ? 'fv-true' : 'fv-false' };
  if (typeof value === 'number')             return { text: Number.isInteger(value) ? String(value) : (value as number).toFixed(3), cls: 'fv-num' };
  if (typeof value === 'string')             return { text: `"${value}"`,  cls: 'fv-str' };
  return { text: String(value), cls: '' };
}

function isPlainObject(v: unknown): v is Record<string, unknown> {
  return typeof v === 'object' && v !== null && !Array.isArray(v);
}

interface FieldRowProps {
  name:   string;
  value:  unknown;
  depth?: number;
}

function FieldRow({ name, value, depth = 0 }: FieldRowProps) {
  const [open, setOpen] = useState(true);
  const indent = depth * 12;

  if (isPlainObject(value)) {
    const entries = Object.entries(value);
    return (
      <div className="field-obj-group">
        {/* object header — clickable to collapse */}
        <div
          className="field-row field-obj-header"
          style={{ paddingLeft: 12 + indent }}
          onClick={() => setOpen(o => !o)}
        >
          <span className={`field-obj-chevron ${open ? 'expanded' : ''}`}>
            <svg width="7" height="7" viewBox="0 0 7 7" fill="currentColor">
              <polygon points="1,1 6,3.5 1,6" />
            </svg>
          </span>
          <span className="field-key">{name}</span>
          <span className="field-obj-hint">{`{${entries.length}}`}</span>
        </div>

        {open && entries.map(([k, v]) => (
          <FieldRow key={k} name={k} value={v} depth={depth + 1} />
        ))}
      </div>
    );
  }

  if (Array.isArray(value)) {
    return (
      <div className="field-row" style={{ paddingLeft: 12 + indent }}>
        <span className="field-key">{name}</span>
        <span className="field-val fv-obj">[{(value as unknown[]).length}]</span>
      </div>
    );
  }

  const { text, cls } = formatPrimitive(value);
  return (
    <div className="field-row" style={{ paddingLeft: 12 + indent }}>
      <span className="field-key">{name}</span>
      <span className={`field-val ${cls}`}>{text}</span>
    </div>
  );
}

// ─────────────────────────────────────────────────────────────
// Component block
// ─────────────────────────────────────────────────────────────

function ComponentBlock({ comp }: { comp: ComponentState }) {
  const [open, setOpen] = useState(true);

  return (
    <div className="comp-block">
      <div className="comp-header" onClick={() => setOpen(o => !o)}>
        <span className={`comp-chevron ${open ? 'expanded' : ''}`}>
          <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor">
            <polygon points="2,1 7,4 2,7" />
          </svg>
        </span>
        <span className="comp-name">{comp.component_name}</span>
        <span className="comp-field-count">{comp.fields.length}</span>
      </div>

      {open && (
        <div className="comp-fields-list">
          {comp.fields.length === 0 ? (
            <span className="field-empty" style={{ paddingLeft: 12 }}>no fields</span>
          ) : (
            comp.fields.map((f, i) => (
              <FieldRow key={f.field_name + i} name={f.field_name} value={f.data} />
            ))
          )}
        </div>
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────
// Inspector panel
// ─────────────────────────────────────────────────────────────

export function InspectorView() {
  const { selectedObject } = useEditorStore();

  return (
    <div className="inspector-view">
      <div className="panel-header">
        <span className="panel-title">Inspector</span>
      </div>

      {!selectedObject ? (
        <div className="panel-empty">Select an object</div>
      ) : (
        <>
          <div className="inspector-header">
            <div className="inspector-subject-name">{selectedObject.object_name}</div>
            <div className="inspector-subject-meta">
              {selectedObject.components.length} component{selectedObject.components.length !== 1 ? 's' : ''}
              {selectedObject.children.length > 0 && ` · ${selectedObject.children.length} children`}
            </div>
          </div>

          <div className="inspector-content">
            {selectedObject.components.length === 0 ? (
              <div className="panel-empty">No components</div>
            ) : (
              selectedObject.components.map((comp, i) => (
                <ComponentBlock key={comp.component_name + i} comp={comp} />
              ))
            )}
          </div>
        </>
      )}
    </div>
  );
}
