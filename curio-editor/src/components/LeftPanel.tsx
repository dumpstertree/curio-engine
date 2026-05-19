import React from 'react';
import { useEditorStore } from '../store';
import { LedgerView }     from './ledger/LedgerView';
import { FormScrollView } from './forms/FormScrollView';

export function LeftPanel() {
  const { ledger, selectedInstance, selectInstance, leftTab, setLeftTab } = useEditorStore();

  const instances = ledger?.instances ?? [];

  return (
    <div className="left-panel">

      {/* Instance dropdown — top level of scene tab */}
      <div className="instance-bar">
        <label className="instance-label">Instance</label>
        <select
          className="instance-select"
          value={selectedInstance}
          onChange={e => selectInstance(Number(e.target.value))}
        >
          {instances.length === 0
            ? <option value={0}>No instances</option>
            : instances.map(inst => (
                <option key={inst.id} value={inst.id}>
                  {inst.name} ({inst.role})
                </option>
              ))
          }
        </select>
      </div>

      {/* Ledger / Forms tab strip */}
      <div className="left-panel-tabs">
        <button
          className={`left-tab ${leftTab === 'ledger' ? 'active' : ''}`}
          onClick={() => setLeftTab('ledger')}
        >
          Ledger
        </button>
        <button
          className={`left-tab ${leftTab === 'forms' ? 'active' : ''}`}
          onClick={() => setLeftTab('forms')}
        >
          Forms
        </button>
      </div>

      {/* Content */}
      <div className="left-panel-content">
        {leftTab === 'ledger' && <LedgerView />}
        {leftTab === 'forms'  && <FormScrollView />}
      </div>

    </div>
  );
}
