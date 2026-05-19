import React, { useEffect } from 'react';
import { useEditorStore } from './store';
import { Toolbar } from './components/Toolbar';
import { TabBar } from './components/TabBar';
import { LeftPanel } from './components/LeftPanel';
import { CenterPanel } from './components/CenterPanel';
import { InspectorView } from './components/forms/InspectorView';
import { StatusBar } from './components/StatusBar';
import { PlaceholderTab } from './components/tabs/PlaceholderTab';
import './App.css';

export default function App() {
  const { activeTab, mode, refreshForms, refreshLedger } = useEditorStore();

  useEffect(() => {
    refreshForms();
    refreshLedger();
  }, []);

  useEffect(() => {
    if (mode !== 'playing') return;
    const id = setInterval(() => {
      refreshForms();
      refreshLedger();
    }, 1000);
    return () => clearInterval(id);
  }, [mode]);

  return (
    <div className="editor">
      <Toolbar />
      <TabBar />

      <div className="main-layout">
        {activeTab === 'play' ? (
          // Play tab IS the combined scene + viewport + inspector layout
          <>
            <LeftPanel />
            <CenterPanel />
            <InspectorView />
          </>
        ) : (
          // TBD tabs show a placeholder over the full area
          <PlaceholderTab tab={activeTab} />
        )}
      </div>

      <StatusBar />
    </div>
  );
}
