import React from 'react';
import { useEditorStore } from '../store';
import type { ObjectState } from '../types';

function countObjects(objects: ObjectState[]): number {
  return objects.reduce((acc, o) => acc + 1 + countObjects(o.children), 0);
}

export function StatusBar() {
  const { mode, tabGroupState, selectedInstance } = useEditorStore();

  const instanceCount = Object.keys(tabGroupState?.id_for_tabs ?? {}).length;

  const nodeCount = tabGroupState
    ? Object.values(tabGroupState.id_for_tabs)
        .flat()
        .reduce((acc, tab) => acc + countObjects(tab.objects), 0)
    : 0;

  return (
    <div className={`status-bar mode-${mode}`}>
      <div className="status-item">
        {mode === 'playing' && <><span className="status-dot" />Playing</>}
        {mode === 'paused'  && '⏸ Paused'}
        {mode === 'stopped' && '■ Stopped'}
      </div>
      {nodeCount > 0 && (
        <div className="status-item">{nodeCount} objects</div>
      )}
      {instanceCount > 0 && (
        <div className="status-item">
          {instanceCount} instance{instanceCount !== 1 ? 's' : ''}
        </div>
      )}
      <div className="status-item" style={{ marginLeft: 'auto' }}>curio engine</div>
    </div>
  );
}
