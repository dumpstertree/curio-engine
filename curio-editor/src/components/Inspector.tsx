import React, { useState } from 'react';
import type { Component, Entity } from '../types';
import { useEditorStore } from '../store';

// ─────────────────────────────────────────────────────────────
// Find entity by id in the tree (recursive)
// ─────────────────────────────────────────────────────────────
function findEntity(entities: Entity[], id: number): Entity | null {
  for (const e of entities) {
    if (e.id === id) return e;
    const found = findEntity(e.children, id);
    if (found) return found;
  }
  return null;
}

// ─────────────────────────────────────────────────────────────
// Format a field value for display
// ─────────────────────────────────────────────────────────────
function formatValue(value: unknown): { text: string; className: string } {
  if (value === null || value === undefined) {
    return { text: 'null', className: 'field-value null-val' };
  }
  if (typeof value === 'boolean') {
    return {
      text: String(value),
      className: `field-value ${value ? 'bool-true' : 'bool-false'}`,
    };
  }
  if (typeof value === 'number') {
    const text = Number.isInteger(value) ? String(value) : value.toFixed(3);
    return { text, className: 'field-value' };
  }
  if (typeof value === 'string') {
    return { text: `"${value}"`, className: 'field-value string-val' };
  }
  if (typeof value === 'object') {
    return { text: '{...}', className: 'field-value nested' };
  }
  return { text: String(value), className: 'field-value' };
}

// ─────────────────────────────────────────────────────────────
// Single component block (collapsible)
// ─────────────────────────────────────────────────────────────
function ComponentBlock({ component }: { component: Component }) {
  const [open, setOpen] = useState(true);
  const fieldEntries = Object.entries(component.fields);

  return (
    <div className="component-block">
      {/* header */}
      <div className="component-header" onClick={() => setOpen((o) => !o)}>
        <div className={`component-chevron ${open ? 'expanded' : ''}`}>
          <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
            <polygon points="3,2 8,5 3,8" />
          </svg>
        </div>
        <span className="component-name">{component.name}</span>
        <span className="component-field-count">{fieldEntries.length}</span>
      </div>

      {/* fields */}
      {open && (
        <div className="component-fields">
          {fieldEntries.length === 0 ? (
            <span className="field-empty">no fields</span>
          ) : (
            fieldEntries.map(([key, value]) => {
              const { text, className } = formatValue(value);
              return (
                <div key={key} className="field-row">
                  <span className="field-key">{key}</span>
                  <span className={className}>{text}</span>
                </div>
              );
            })
          )}
        </div>
      )}
    </div>
  );
}

// ─────────────────────────────────────────────────────────────
// Inspector panel
// ─────────────────────────────────────────────────────────────
export function Inspector() {
  const { snapshot, selected } = useEditorStore();

  const entity = selected != null && snapshot != null
    ? findEntity(snapshot.entities, selected)
    : null;

  return (
    <div className="inspector-panel">
      {/* header */}
      <div className="panel-section-header" style={{ borderBottom: '1px solid var(--border)', height: 28, flexShrink: 0 }}>
        <span className="panel-section-title">Inspector</span>
      </div>

      {/* content */}
      {entity == null ? (
        <div className="inspector-empty">
          <div>
            <div style={{ textAlign: 'center', marginBottom: 4 }}>
              <svg width="32" height="32" viewBox="0 0 32 32" fill="none" stroke="currentColor" strokeWidth="1" style={{ color: 'var(--text-muted)' }}>
                <circle cx="16" cy="12" r="5" />
                <path d="M8 28c0-4.4 3.6-8 8-8s8 3.6 8 8" />
              </svg>
            </div>
            Select an entity
          </div>
        </div>
      ) : (
        <>
          {/* entity header */}
          <div className="inspector-header">
            <div className="inspector-entity-name">{entity.name}</div>
            <div className="inspector-entity-id">id: {entity.id}</div>
            <div className="inspector-entity-id">
              {entity.components.length} component{entity.components.length !== 1 ? 's' : ''}
              {entity.children.length > 0 && ` · ${entity.children.length} children`}
            </div>
          </div>

          {/* components */}
          <div className="inspector-content">
            {entity.components.length === 0 ? (
              <div className="empty-state">No components</div>
            ) : (
              entity.components.map((comp) => (
                <ComponentBlock key={comp.name} component={comp} />
              ))
            )}
          </div>
        </>
      )}
    </div>
  );
}
