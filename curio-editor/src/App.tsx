import React, { useEffect } from 'react';
import { useEditorStore }  from './store';
import { Toolbar }         from './components/Toolbar';
import { TabBar }          from './components/TabBar';
import { LeftPanel }       from './components/LeftPanel';
import { CenterPanel }     from './components/CenterPanel';
import { InspectorView }   from './components/forms/InspectorView';
import { StatusBar }       from './components/StatusBar';
import { PlaceholderTab }  from './components/tabs/PlaceholderTab';
import './App.css';

export default function App() {
  const { activeTab, mode, refreshTabGroup } = useEditorStore();

  useEffect(() => {
    refreshTabGroup();
  }, []);

  useEffect(() => {
    if (mode !== 'playing') return;
    const id = setInterval(refreshTabGroup, 1000);
    return () => clearInterval(id);
  }, [mode]);

  return (
    <div className="editor">
      <Toolbar />
      <TabBar />
      <div className="main-layout">
        {activeTab === 'play' ? (
          <>
            <LeftPanel />
            <CenterPanel />
            <InspectorView />
          </>
        ) : (
          <PlaceholderTab tab={activeTab} />
        )}
      </div>
      <StatusBar />
    </div>
  );
}
