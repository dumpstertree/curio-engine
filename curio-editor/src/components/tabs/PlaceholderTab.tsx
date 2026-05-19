import React from 'react';
import type { TopTab } from '../../types';

const LABELS: Record<string, string> = {
  asset:  'Asset Browser',
  input:  'Input Mapping',
  prefab: 'Prefab Editor',
};

export function PlaceholderTab({ tab }: { tab: TopTab }) {
  return (
    <div className="placeholder-tab">
      <div className="placeholder-content">
        <svg width="48" height="48" viewBox="0 0 48 48" fill="none" stroke="currentColor" strokeWidth="1" style={{ color: 'var(--text-muted)', marginBottom: 12 }}>
          <rect x="6" y="6" width="36" height="36" rx="4" strokeDasharray="5 3" />
          <line x1="24" y1="16" x2="24" y2="32" />
          <line x1="16" y1="24" x2="32" y2="24" />
        </svg>
        <div className="placeholder-label">{LABELS[tab] ?? tab}</div>
        <div className="placeholder-sub">Not yet implemented</div>
      </div>
    </div>
  );
}
