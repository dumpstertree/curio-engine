import React, { useEffect, useState } from 'react';
import { useEditorStore } from './store';
import { TitleBar } from './components/TitleBar';
import { ActivityBar } from './components/ActivityBar';
import { SceneHierarchy } from './components/SceneHierarchy';
import { Viewport } from './components/Viewport';
import { Inspector } from './components/Inspector';
import { StatusBar } from './components/StatusBar';
import './App.css';

type PanelId = 'hierarchy' | 'search' | 'settings';

export default function App() {
  const { refreshSnapshot, mode } = useEditorStore();
  const [activePanel, setActivePanel] = useState<PanelId>('hierarchy');

  // load scene on mount
  useEffect(() => {
    refreshSnapshot();
  }, []);

  // poll snapshot while playing
  useEffect(() => {
    if (mode !== 'playing') return;
    const id = setInterval(refreshSnapshot, 1000);
    return () => clearInterval(id);
  }, [mode]);

  return (
    <div className="editor">
      <TitleBar />
      <div className="editor-body">
        <ActivityBar active={activePanel} onChange={setActivePanel} />

        {/* side panel — swappable by activity bar */}
        {activePanel === 'hierarchy' && <SceneHierarchy />}
        {activePanel === 'search' && (
          <div className="side-panel">
            <div className="panel-section-header">
              <span className="panel-section-title">Search</span>
            </div>
            <div className="empty-state">Coming soon</div>
          </div>
        )}
        {activePanel === 'settings' && (
          <div className="side-panel">
            <div className="panel-section-header">
              <span className="panel-section-title">Settings</span>
            </div>
            <div className="empty-state">Coming soon</div>
          </div>
        )}

        {/* main content */}
        <Viewport />

        {/* right panel */}
        <Inspector />
      </div>
      <StatusBar />
    </div>
  );
}
