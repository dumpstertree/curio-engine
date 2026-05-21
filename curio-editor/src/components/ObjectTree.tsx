import React from 'react';
import { useEditorStore } from '../store';
import type { ObjectState } from '../types';

// ○ hollow = has children   ● filled = leaf
function NodeIcon({ hasChildren }: { hasChildren: boolean }) {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" className="node-icon" fill="none">
      {hasChildren ? (
        <circle cx="5" cy="5" r="3.5" stroke="currentColor" strokeWidth="1.5" />
      ) : (
        <circle cx="5" cy="5" r="2.5" fill="currentColor" />
      )}
    </svg>
  );
}

interface NodeRowProps {
  obj:   ObjectState;
  path:  string;
  depth: number;
}

function NodeRow({ obj, path, depth }: NodeRowProps) {
  const { selectedObject, expandedNodes, selectObject, toggleNode } = useEditorStore();
  const isSelected  = selectedObject === obj;
  const isExpanded  = expandedNodes.has(path);
  const hasChildren = obj.children.length > 0;

  return (
    <>
      <div className={`pf-row ${isSelected ? 'selected' : ''}`}>
        {/* indent */}
        <div style={{ width: depth * 16 + 2, flexShrink: 0 }} />

        {/* chevron — only expand/collapse, independent of row click */}
        <button
          className={`pf-chevron-btn ${!hasChildren ? 'pf-chevron-hidden' : ''}`}
          onClick={e => { e.stopPropagation(); if (hasChildren) toggleNode(path); }}
          tabIndex={-1}
        >
          {hasChildren && (
            <svg
              width="8" height="8" viewBox="0 0 8 8" fill="currentColor"
              style={{ transform: isExpanded ? 'rotate(90deg)' : 'none', transition: 'transform .12s' }}
            >
              <polygon points="2,1 7,4 2,7" />
            </svg>
          )}
        </button>

        {/* circle icon */}
        <NodeIcon hasChildren={hasChildren} />

        {/* name — click loads inspector only */}
        <span
          className="pf-name"
          onClick={() => selectObject(isSelected ? null : obj)}
        >
          {obj.object_name}
        </span>

        {obj.components.length > 0 && (
          <span className="pf-data-count">{obj.components.length}</span>
        )}
      </div>

      {isExpanded && obj.children.map((child, i) => (
        <NodeRow
          key={`${path}/${child.object_name}${i}`}
          obj={child}
          path={`${path}/${child.object_name}${i}`}
          depth={depth + 1}
        />
      ))}
    </>
  );
}

export function ObjectTree({ objects }: { objects: ObjectState[] }) {
  if (objects.length === 0) return <div className="panel-empty">No objects</div>;

  return (
    <div className="pf-tree">
      {objects.map((obj, i) => (
        <NodeRow
          key={`root/${obj.object_name}${i}`}
          obj={obj}
          path={`root/${obj.object_name}${i}`}
          depth={0}
        />
      ))}
    </div>
  );
}
