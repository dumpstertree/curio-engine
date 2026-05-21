import React from 'react';
import { useEditorStore } from '../store';
import { ObjectTree }     from './ObjectTree';
import { CustomSelect }   from './CustomSelect';

export function LeftPanel() {
  const {
    tabGroupState,
    selectedInstance, selectInstance,
    activeLeftTab,    setActiveLeftTab,
  } = useEditorStore();

  const idForTabs     = tabGroupState?.id_for_tabs ?? {};
  const instanceKeys  = Object.keys(idForTabs).sort();
  const tabs          = idForTabs[selectedInstance] ?? [];
  const activeObjects = tabs[activeLeftTab]?.objects ?? [];

  const instanceOptions = instanceKeys.map(k => ({ value: k, label: k }));

  return (
    <div className="left-panel">

      {/* Instance dropdown — custom to fix Tauri WebKit styling */}
      <div className="instance-bar">
        <label className="instance-label">Instance</label>
        <CustomSelect
          value={selectedInstance}
          options={instanceOptions.length > 0 ? instanceOptions : [{ value: '', label: 'No instances' }]}
          onChange={selectInstance}
          className="instance-dropdown"
        />
      </div>

      {/* Dynamic tab strip */}
      <div className="left-panel-tabs">
        {tabs.length === 0 ? (
          <span className="left-tab-empty">No tabs</span>
        ) : (
          tabs.map((tab, idx) => (
            <button
              key={tab.tab_name}
              className={`left-tab ${activeLeftTab === idx ? 'active' : ''}`}
              onClick={() => setActiveLeftTab(idx)}
            >
              {tab.tab_name}
            </button>
          ))
        )}
      </div>

      {/* Object tree */}
      <div className="left-panel-content">
        {!tabGroupState ? (
          <div className="panel-empty">No data</div>
        ) : (
          <ObjectTree objects={activeObjects} />
        )}
      </div>

    </div>
  );
}
