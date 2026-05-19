import React, { useState } from 'react';
import { useEditorStore } from '../../store';
import type { Component, Form, LedgerRecord, RecordPermission } from '../../types';

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────

function findForm(forms: Form[], id: number): Form | null {
  for (const f of forms) {
    if (f.id === id) return f;
    const found = findForm(f.children, id);
    if (found) return found;
  }
  return null;
}

function formatValue(value: unknown): { text: string; cls: string } {
  if (value === null || value === undefined) return { text: 'null',         cls: 'fv-null' };
  if (typeof value === 'boolean')            return { text: String(value),  cls: value ? 'fv-true' : 'fv-false' };
  if (typeof value === 'number')             return { text: Number.isInteger(value) ? String(value) : value.toFixed(3), cls: 'fv-num' };
  if (typeof value === 'string')             return { text: `"${value}"`,   cls: 'fv-str' };
  if (typeof value === 'object')             return { text: '{…}',          cls: 'fv-obj' };
  return { text: String(value), cls: '' };
}

// ─────────────────────────────────────────────────────────────
// Form inspector
// ─────────────────────────────────────────────────────────────

function ComponentBlock({ comp }: { comp: Component }) {
  const [open, setOpen] = useState(true);
  const fields = Object.entries(comp.fields);

  return (
    <div className="comp-block">
      <div className="comp-header" onClick={() => setOpen(o => !o)}>
        <span className={`comp-chevron ${open ? 'expanded' : ''}`}>
          <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor">
            <polygon points="2,1 7,4 2,7" />
          </svg>
        </span>
        <span className="comp-name">{comp.name}</span>
        <span className="comp-field-count">{fields.length}</span>
      </div>
      {open && (
        <div className="comp-fields">
          {fields.length === 0
            ? <span className="field-empty">no fields</span>
            : fields.map(([key, val]) => {
                const { text, cls } = formatValue(val);
                return (
                  <div key={key} className="field-row">
                    <span className="field-key">{key}</span>
                    <span className={`field-val ${cls}`}>{text}</span>
                  </div>
                );
              })
          }
        </div>
      )}
    </div>
  );
}

function FormInspector({ form }: { form: Form }) {
  return (
    <>
      <div className="inspector-header">
        <div className="inspector-subject-label">Form</div>
        <div className="inspector-subject-name">{form.name}</div>
        <div className="inspector-subject-meta">
          id: {form.id}
          {form.children.length > 0 && ` · ${form.children.length} children`}
          {` · ${form.components.length} component${form.components.length !== 1 ? 's' : ''}`}
        </div>
      </div>
      <div className="inspector-content">
        {form.components.length === 0
          ? <div className="panel-empty">No components</div>
          : form.components.map(comp => (
              <ComponentBlock key={comp.name} comp={comp} />
            ))
        }
      </div>
    </>
  );
}

// ─────────────────────────────────────────────────────────────
// Record inspector
// ─────────────────────────────────────────────────────────────

function PermBadge({ perm }: { perm: RecordPermission }) {
  return (
    <div className="perm-badge-group">
      <span className={`perm-badge ${perm === 'read'  || perm === 'readwrite' ? 'perm-on' : 'perm-off'}`}>R</span>
      <span className={`perm-badge ${perm === 'write' || perm === 'readwrite' ? 'perm-on' : 'perm-off'}`}>W</span>
    </div>
  );
}

function RecordInspector({ record }: { record: LedgerRecord }) {
  const fields = Object.entries(record.value);

  return (
    <>
      <div className="inspector-header">
        <div className="inspector-subject-label">Record</div>
        <div className="inspector-subject-name">{record.name}</div>
        <div className="inspector-subject-meta" style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span>{record.record_type}</span>
          <PermBadge perm={record.permissions} />
        </div>
      </div>
      <div className="inspector-content">
        <div className="comp-block">
          <div className="comp-fields" style={{ paddingLeft: 12 }}>
            {fields.length === 0
              ? <span className="field-empty">no fields</span>
              : fields.map(([key, val]) => {
                  const { text, cls } = formatValue(val);
                  return (
                    <div key={key} className="field-row">
                      <span className="field-key">{key}</span>
                      <span className={`field-val ${cls}`}>{text}</span>
                    </div>
                  );
                })
            }
          </div>
        </div>
      </div>
    </>
  );
}

// ─────────────────────────────────────────────────────────────
// Inspector panel
// ─────────────────────────────────────────────────────────────

export function InspectorView() {
  const { forms, selectedForm, selectedRecord } = useEditorStore();

  const form = selectedForm != null && forms
    ? findForm(forms.forms, selectedForm)
    : null;

  return (
    <div className="inspector-view">
      <div className="panel-header">
        <span className="panel-title">Inspector</span>
      </div>

      {!form && !selectedRecord ? (
        <div className="panel-empty">
          Select a form or record
        </div>
      ) : form ? (
        <FormInspector form={form} />
      ) : selectedRecord ? (
        <RecordInspector record={selectedRecord} />
      ) : null}
    </div>
  );
}
