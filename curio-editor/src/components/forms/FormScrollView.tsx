import React from 'react';
import { useEditorStore } from '../../store';
import type { Form } from '../../types';

function formIcon(form: Form): string {
  const names = form.components.map(c => c.name.toLowerCase());
  if (names.includes('camera')) return '📷';
  if (names.includes('light'))  return '💡';
  if (names.includes('mesh'))   return '⬡';
  if (form.children.length > 0) return '📁';
  return '◆';
}

function FormRow({ form, depth }: { form: Form; depth: number }) {
  const { selectedForm, expandedForms, selectForm, toggleForm } = useEditorStore();
  const isSelected  = selectedForm === form.id;
  const isExpanded  = expandedForms.has(form.id);
  const hasChildren = form.children.length > 0;

  return (
    <>
      <div
        className={`form-row ${isSelected ? 'selected' : ''}`}
        onClick={() => selectForm(isSelected ? null : form.id)}
        onDoubleClick={() => hasChildren && toggleForm(form.id)}
      >
        <div style={{ width: depth * 12 + 4, flexShrink: 0 }} />
        <div
          className={`form-chevron ${hasChildren ? (isExpanded ? 'expanded' : '') : 'leaf'}`}
          onClick={e => { e.stopPropagation(); if (hasChildren) toggleForm(form.id); }}
        >
          {hasChildren && (
            <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor">
              <polygon points="2,1 7,4 2,7" />
            </svg>
          )}
        </div>
        <span className="form-icon">{formIcon(form)}</span>
        <span className="form-name">{form.name}</span>
        {form.components.length > 0 && (
          <span className="form-comp-count">{form.components.length}</span>
        )}
      </div>

      {isExpanded && form.children.map(child => (
        <FormRow key={child.id} form={child} depth={depth + 1} />
      ))}
    </>
  );
}

export function FormScrollView() {
  const { forms } = useEditorStore();

  return (
    <div className="form-fill">
      {!forms ? (
        <div className="panel-empty">No scene loaded</div>
      ) : forms.forms.length === 0 ? (
        <div className="panel-empty">Scene is empty</div>
      ) : (
        forms.forms.map(form => (
          <FormRow key={form.id} form={form} depth={0} />
        ))
      )}
    </div>
  );
}
