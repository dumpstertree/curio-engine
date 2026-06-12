import React, { useEffect, useState } from 'react';
import { api } from '../../api';
import type { DirEntry } from '../../api';

const ASSET_ROOT = '/home/dumpstertree/Git/Rust/system_test/assets';

const SUPPORTED_EXTS = new Set(['.png', '.glb', '.anim']);

function fileExt(name: string): string {
  const dot = name.lastIndexOf('.');
  return dot >= 0 ? name.slice(dot).toLowerCase() : '';
}

function fileIcon(entry: DirEntry): string {
  if (entry.is_dir) return '📁';
  const ext = fileExt(entry.name);
  if (ext === '.png')  return '🖼';
  if (ext === '.glb')  return '📦';
  if (ext === '.anim') return '🎬';
  return '📄';
}

interface TreeNodeProps {
  entry:        DirEntry;
  selectedPath: string | null;
  onSelect:     (entry: DirEntry) => void;
  depth:        number;
}

function TreeNode({ entry, selectedPath, onSelect, depth }: TreeNodeProps) {
  const [expanded, setExpanded] = useState(false);
  const [children, setChildren] = useState<DirEntry[]>([]);
  const [loaded,   setLoaded]   = useState(false);
  const [loading,  setLoading]  = useState(false);

  const ext         = fileExt(entry.name);
  const isSupported = !entry.is_dir && SUPPORTED_EXTS.has(ext);
  const isSelected  = selectedPath === entry.path;

  async function handleClick() {
    if (entry.is_dir) {
      if (!expanded && !loaded) {
        setLoading(true);
        try {
          const entries = await api.listDir(entry.path);
          setChildren(entries);
          setLoaded(true);
        } catch (e) {
          console.error('listDir failed:', e);
        } finally {
          setLoading(false);
        }
      }
      setExpanded(e => !e);
    } else if (isSupported) {
      onSelect(entry);
    }
  }

  return (
    <>
      <div
        className={`asset-tree-row${isSelected ? ' selected' : ''}${!entry.is_dir && !isSupported ? ' unsupported' : ''}`}
        style={{ paddingLeft: 10 + depth * 14 }}
        onClick={handleClick}
      >
        {entry.is_dir ? (
          <span className="asset-chevron">
            <svg
              width="7" height="7" viewBox="0 0 7 7" fill="currentColor"
              style={{ transform: expanded ? 'rotate(90deg)' : 'none', transition: 'transform .12s' }}
            >
              <polygon points="1,1 6,3.5 1,6" />
            </svg>
          </span>
        ) : (
          <span className="asset-chevron-spacer" />
        )}
        <span className="asset-icon">{fileIcon(entry)}</span>
        <span className="asset-name">{entry.name}</span>
        {loading && <span className="asset-loading">…</span>}
        {!entry.is_dir && !isSupported && (
          <span className="asset-unsupported-badge">{ext || '?'}</span>
        )}
      </div>

      {entry.is_dir && expanded && children.map(child => (
        <TreeNode
          key={child.path}
          entry={child}
          selectedPath={selectedPath}
          onSelect={onSelect}
          depth={depth + 1}
        />
      ))}
    </>
  );
}

interface AssetFileTreeProps {
  selectedPath: string | null;
  onSelect:     (entry: DirEntry) => void;
}

export function AssetFileTree({ selectedPath, onSelect }: AssetFileTreeProps) {
  const [roots,   setRoots]   = useState<DirEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error,   setError]   = useState<string | null>(null);

  useEffect(() => {
    api.listDir(ASSET_ROOT)
      .then(entries => { setRoots(entries); setLoading(false); })
      .catch(e => { setError(String(e)); setLoading(false); });
  }, []);

  if (loading) return <div className="panel-empty">Loading…</div>;
  if (error)   return <div className="panel-empty asset-error">{error}</div>;
  if (roots.length === 0) return <div className="panel-empty">Empty folder</div>;

  return (
    <div className="asset-tree">
      {roots.map(entry => (
        <TreeNode
          key={entry.path}
          entry={entry}
          selectedPath={selectedPath}
          onSelect={onSelect}
          depth={0}
        />
      ))}
    </div>
  );
}
