import React from 'react';
import { useEditorStore } from '../store';

function countForms(forms: import('../types').Form[]): number {
  return forms.reduce((acc, f) => acc + 1 + countForms(f.children), 0);
}

export function StatusBar() {
  const { mode, forms, ledger } = useEditorStore();

  const formCount     = forms  ? countForms(forms.forms)            : 0;
  const instanceCount = ledger ? ledger.instances.length            : 0;

  return (
    <div className={`status-bar mode-${mode}`}>
      <div className="status-item">
        {mode === 'playing' && <><span className="status-dot" />Playing</>}
        {mode === 'paused'  && '⏸ Paused'}
        {mode === 'stopped' && '■ Stopped'}
      </div>
      <div className="status-item">
        <svg width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" strokeWidth="1.2">
          <rect x="1" y="1" width="4" height="4" rx="0.5" />
          <rect x="6" y="1" width="4" height="4" rx="0.5" />
          <rect x="1" y="6" width="4" height="4" rx="0.5" />
          <rect x="6" y="6" width="4" height="4" rx="0.5" />
        </svg>
        {formCount} forms
      </div>
      {instanceCount > 0 && (
        <div className="status-item">{instanceCount} instance{instanceCount !== 1 ? 's' : ''}</div>
      )}
      <div className="status-item" style={{ marginLeft: 'auto' }}>curio engine</div>
    </div>
  );
}
