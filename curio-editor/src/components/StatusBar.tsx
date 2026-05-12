import React from 'react';
import { useEditorStore } from '../store';
import type { Entity } from '../types';

function countEntities(entities: Entity[]): number {
  return entities.reduce((acc, e) => acc + 1 + countEntities(e.children), 0);
}

export function StatusBar() {
  const { mode, snapshot } = useEditorStore();

  const entityCount = snapshot ? countEntities(snapshot.entities) : 0;

  const modeLabel =
    mode === 'playing' ? '▶ Playing'
    : mode === 'paused' ? '⏸ Paused'
    : '■ Stopped';

  return (
    <div className={`status-bar ${mode}`}>
      <div className="status-item">{modeLabel}</div>
      <div className="status-item">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.2">
          <rect x="1" y="1" width="4" height="4" rx="0.5" />
          <rect x="7" y="1" width="4" height="4" rx="0.5" />
          <rect x="1" y="7" width="4" height="4" rx="0.5" />
          <rect x="7" y="7" width="4" height="4" rx="0.5" />
        </svg>
        {entityCount} entities
      </div>
      <div className="status-item" style={{ marginLeft: 'auto' }}>
        curio engine
      </div>
    </div>
  );
}
