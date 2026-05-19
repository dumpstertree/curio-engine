import React from 'react';
import { useEditorStore } from '../store';
import type { TopTab } from '../types';

const TABS: { id: TopTab; label: string; tbd?: boolean }[] = [
  { id: 'play',   label: '▶ Play' },
  { id: 'asset',  label: 'Asset',  tbd: true },
  { id: 'input',  label: 'Input',  tbd: true },
  { id: 'prefab', label: 'Prefab', tbd: true },
];

export function TabBar() {
  const { activeTab, setActiveTab } = useEditorStore();

  return (
    <div className="tab-bar">
      {TABS.map(tab => (
        <button
          key={tab.id}
          className={`top-tab ${activeTab === tab.id ? 'active' : ''} ${tab.tbd ? 'tbd' : ''}`}
          onClick={() => setActiveTab(tab.id)}
        >
          {tab.label}
          {tab.tbd && <span className="tbd-badge">TBD</span>}
        </button>
      ))}
    </div>
  );
}
