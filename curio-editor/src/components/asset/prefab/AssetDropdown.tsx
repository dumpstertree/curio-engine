import React, { useEffect, useRef, useState } from 'react';
import { api } from '../../../api';
import type { ManifestEntry } from '../../../api';

interface Props {
  /** Current stored value — a numeric ID as string, or empty/null if unset */
  value:       string | null;
  /** File extensions this field accepts e.g. ['.glb'] or ['.anim'] or ['.comp'] */
  accepts:     string[];
  /** Called with the new ID as string, or null to clear */
  onChange:    (id: string | null) => void;
  placeholder?: string;
}

export function AssetDropdown({ value, accepts, onChange, placeholder = '— select asset —' }: Props) {
  const [open,    setOpen]    = useState(false);
  const [entries, setEntries] = useState<ManifestEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  // Load manifest on mount (to resolve display name) AND whenever dropdown opens
  useEffect(() => {
    setLoading(true);
    api.readManifest()
      .then(all => {
        const filtered = all.filter(e => {
          const dot = e.uri.lastIndexOf('.');
          const ext = dot >= 0 ? e.uri.slice(dot).toLowerCase() : '';
          return accepts.includes(ext);
        });
        setEntries(filtered);
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, [open]); // re-runs on open to pick up new assets; also runs once on mount

  // Close on outside click
  useEffect(() => {
    if (!open) return;
    function onDown(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    }
    document.addEventListener('mousedown', onDown);
    return () => document.removeEventListener('mousedown', onDown);
  }, [open]);

  // Resolve display name from current ID
  const currentId    = value ? parseInt(value, 10) : null;
  const currentEntry = entries.find(e => e.id === currentId);
  const displayName  = currentEntry
    ? currentEntry.name
    : currentId !== null
      ? `unknown (${currentId})`
      : null;

  function select(entry: ManifestEntry) {
    onChange(String(entry.id));
    setOpen(false);
  }

  function clear(e: React.MouseEvent) {
    e.stopPropagation();
    onChange(null);
    setOpen(false);
  }

  return (
    <div className="asset-dropdown" ref={ref}>
      <div
        className={`asset-dropdown-trigger ${open ? 'open' : ''} ${!displayName ? 'empty' : ''}`}
        onClick={() => setOpen(o => !o)}
      >
        <span className="asset-dropdown-value">
          {displayName ?? <em className="field-val-empty">{placeholder}</em>}
        </span>
        {displayName && (
          <button className="asset-dropdown-clear" onClick={clear} title="Clear">
            <svg width="8" height="8" viewBox="0 0 8 8" fill="none" stroke="currentColor" strokeWidth="1.4">
              <line x1="1" y1="1" x2="7" y2="7"/><line x1="7" y1="1" x2="1" y2="7"/>
            </svg>
          </button>
        )}
        <span className="asset-dropdown-arrow">
          <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor"
            style={{ transform: open ? 'rotate(180deg)' : 'none', transition: 'transform .12s' }}>
            <polygon points="1,2 7,2 4,6"/>
          </svg>
        </span>
      </div>

      {open && (
        <div className="asset-dropdown-menu">
          {loading && <div className="asset-dropdown-item muted">Loading…</div>}
          {!loading && entries.length === 0 && (
            <div className="asset-dropdown-item muted">No matching assets</div>
          )}
          {!loading && entries.map(entry => (
            <div
              key={entry.id}
              className={`asset-dropdown-item ${entry.id === currentId ? 'selected' : ''}`}
              onClick={() => select(entry)}
            >
              <span className="asset-dropdown-item-name">{entry.name}</span>
              <span className="asset-dropdown-item-meta">
                {entry.uri.slice(entry.uri.lastIndexOf('/') + 1)}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
