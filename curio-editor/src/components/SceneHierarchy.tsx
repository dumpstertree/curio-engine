import React from 'react';
import type { Entity } from '../types';
import { useEditorStore } from '../store';

// ─────────────────────────────────────────────────────────────
// Entity icon — pick based on components present
// ─────────────────────────────────────────────────────────────
function entityIcon(entity: Entity): string {
  const names = entity.components.map((c) => c.name.toLowerCase());
  if (names.includes('camera'))    return '📷';
  if (names.includes('light'))     return '💡';
  if (names.includes('mesh'))      return '⬡';
  if (names.includes('player'))    return '🧑';
  if (entity.children.length > 0) return '📁';
  return '◆';
}

// ─────────────────────────────────────────────────────────────
// Single entity row (recursive)
// ─────────────────────────────────────────────────────────────
interface RowProps {
  entity: Entity;
  depth: number;
}

function EntityRow({ entity, depth }: RowProps) {
  const { selected, expanded, selectEntity, toggleExpand } = useEditorStore();
  const isSelected = selected === entity.id;
  const isExpanded = expanded.has(entity.id);
  const hasChildren = entity.children.length > 0;

  return (
    <>
      <div
        className={`entity-row ${isSelected ? 'selected' : ''}`}
        onClick={() => selectEntity(isSelected ? null : entity.id)}
        onDoubleClick={() => hasChildren && toggleExpand(entity.id)}
      >
        {/* indent */}
        <div className="entity-indent" style={{ width: depth * 12 + 4 }} />

        {/* expand chevron */}
        <div
          className={`entity-chevron ${hasChildren ? (isExpanded ? 'expanded' : '') : 'leaf'}`}
          onClick={(e) => {
            e.stopPropagation();
            if (hasChildren) toggleExpand(entity.id);
          }}
        >
          {hasChildren && (
            <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
              <polygon points="3,2 8,5 3,8" />
            </svg>
          )}
        </div>

        {/* icon */}
        <div className="entity-icon">{entityIcon(entity)}</div>

        {/* name */}
        <span className="entity-name">{entity.name}</span>

        {/* component count badge */}
        {entity.components.length > 0 && (
          <span className="entity-badge">{entity.components.length}</span>
        )}
      </div>

      {/* children */}
      {isExpanded && entity.children.map((child) => (
        <EntityRow key={child.id} entity={child} depth={depth + 1} />
      ))}
    </>
  );
}

// ─────────────────────────────────────────────────────────────
// Scene hierarchy panel
// ─────────────────────────────────────────────────────────────
export function SceneHierarchy() {
  const { snapshot, refreshSnapshot } = useEditorStore();

  return (
    <div className="side-panel">
      <div className="panel-section">
        {/* header */}
        <div className="panel-section-header">
          <span className="panel-section-title">Scene</span>
          <div className="panel-section-actions">
            <button
              className="panel-icon-btn"
              onClick={refreshSnapshot}
              title="Refresh"
            >
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                <path d="M1 7a6 6 0 1 0 1-3.2" />
                <polyline points="1,1 1,4 4,4" />
              </svg>
            </button>
            <button className="panel-icon-btn" title="Add entity">
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.5">
                <line x1="7" y1="2" x2="7" y2="12" />
                <line x1="2" y1="7" x2="12" y2="7" />
              </svg>
            </button>
          </div>
        </div>

        {/* tree */}
        <div className="panel-section-content">
          {snapshot == null ? (
            <div className="empty-state">No scene loaded</div>
          ) : snapshot.entities.length === 0 ? (
            <div className="empty-state">Scene is empty</div>
          ) : (
            snapshot.entities.map((entity) => (
              <EntityRow key={entity.id} entity={entity} depth={0} />
            ))
          )}
        </div>
      </div>
    </div>
  );
}
