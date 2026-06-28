import React, { useEffect } from 'react';
import { useEditorStore }  from './store';
import { api }             from './api';
import { Toolbar }         from './components/Toolbar';
import { TabBar }          from './components/TabBar';
import { LeftPanel }       from './components/LeftPanel';
import { CenterPanel }     from './components/CenterPanel';
import { InspectorView }   from './components/forms/InspectorView';
import { StatusBar }       from './components/StatusBar';
import { PlaceholderTab }  from './components/tabs/PlaceholderTab';
import { AssetTab }        from './components/asset/AssetTab';
import './App.css';

export default function App() {
  const {
    activeTab, mode, refreshTabGroup, startPolling, stopPolling,
    selectedObject, loadProjectPath,
  } = useEditorStore();

  // Spin up the GameRunner2 thread immediately on app start so it's ready
  // before any compile or play commands are issued.
  useEffect(() => { api.initialize().catch(console.error); }, []);

  // Load project path once
  useEffect(() => { loadProjectPath(); }, []);

  // Initial tab group load
  useEffect(() => { refreshTabGroup(); }, []);

  // Poll when object selected or playing
  useEffect(() => {
    if (selectedObject) {
      startPolling();
      return () => stopPolling();
    }
    if (mode === 'playing') {
      const id = setInterval(refreshTabGroup, 1000);
      return () => clearInterval(id);
    }
  }, [selectedObject, mode]);

  return (
    <div className="editor">
      <Toolbar />
      <TabBar />
      <div className="main-layout">
        {activeTab === 'play' && (
          <>
            <LeftPanel />
            <CenterPanel />
            <InspectorView />
          </>
        )}
        {activeTab === 'asset' && <AssetTab />}
        {activeTab !== 'play' && activeTab !== 'asset' && (
          <PlaceholderTab tab={activeTab} />
        )}
      </div>
      <StatusBar />
    </div>
  );
}
