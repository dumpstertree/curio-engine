import React from 'react';
import { useEditorStore } from '../../store';
import type { LedgerRecord, RecordPermission } from '../../types';

function PermBadge({ perm }: { perm: RecordPermission }) {
  return (
    <div className="perm-badge-group">
      <span className={`perm-badge ${perm === 'read' || perm === 'readwrite' ? 'perm-on' : 'perm-off'}`} title="Read">R</span>
      <span className={`perm-badge ${perm === 'write' || perm === 'readwrite' ? 'perm-on' : 'perm-off'}`} title="Write">W</span>
    </div>
  );
}

function RecordRow({ record }: { record: LedgerRecord }) {
  const { selectedRecord, selectRecord } = useEditorStore();
  const isSelected = selectedRecord?.name === record.name;

  return (
    // clicking sends record to inspector — no inline expansion
    <div
      className={`record-row ${isSelected ? 'selected' : ''}`}
      onClick={() => selectRecord(isSelected ? null : record)}
    >
      <span className="record-name">{record.name}</span>
      <span className="record-type">{record.record_type}</span>
      <PermBadge perm={record.permissions} />
    </div>
  );
}

export function LedgerView() {
  const { ledger, selectedInstance } = useEditorStore();
  const instance = ledger?.instances.find(i => i.id === selectedInstance);

  return (
    <div className="ledger-fill">
      {!ledger ? (
        <div className="panel-empty">No ledger data</div>
      ) : !instance ? (
        <div className="panel-empty">Select an instance</div>
      ) : instance.records.length === 0 ? (
        <div className="panel-empty">No records</div>
      ) : (
        instance.records.map(record => (
          <RecordRow key={record.name} record={record} />
        ))
      )}
    </div>
  );
}
