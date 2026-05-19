import React from 'react';
import { LedgerView } from '../ledger/LedgerView';
import { FormScrollView } from '../forms/FormScrollView';
import { InspectorView } from '../forms/InspectorView';

export function SceneTab() {
  return (
    <div className="scene-tab">
      {/* Left — Ledger */}
      <LedgerView />

      {/* Center — placeholder */}
      <div className="scene-center">
        <div className="scene-center-placeholder">
          <svg width="40" height="40" viewBox="0 0 40 40" fill="none" stroke="currentColor" strokeWidth="1" style={{ color: 'var(--text-muted)', marginBottom: 8 }}>
            <rect x="4" y="4" width="32" height="32" rx="3" />
            <line x1="4"  y1="14" x2="36" y2="14" />
            <line x1="14" y1="14" x2="14" y2="36" />
          </svg>
          <div>Scene View</div>
          <div style={{ fontSize: 11, marginTop: 4 }}>3D editor coming soon</div>
        </div>
      </div>

      {/* Right — Forms + Inspector stacked */}
      <div className="right-panel">
        <FormScrollView />
        <InspectorView />
      </div>
    </div>
  );
}
