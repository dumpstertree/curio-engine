import React, { useEffect, useState, useRef, useCallback } from 'react';
import { api } from '../../api';
import type { DirEntry } from '../../api';

const ASSET_ROOT = '/home/dumpstertree/Git/Rust/system_test/assets';
const SUPPORTED_EXTS = new Set(['.png', '.glb', '.anim', '.comp']);

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
  if (ext === '.comp') return '🧩';
  return '📄';
}

/** Returns path/to/dir/name, deduplicating with _1, _2... if needed. */
async function uniquePath(dir: string, name: string): Promise<string> {
  const ext  = fileExt(name);
  const base = ext ? name.slice(0, name.length - ext.length) : name;

  let candidate = `${dir}/${name}`;
  let i = 1;
  while (true) {
    try {
      await api.listDir(candidate.replace(/\/[^/]+$/, '')); // will throw if parent missing
      // try to list the candidate as a dir or check existence by reading
      await api.readFileBytes(candidate);
      // if we get here it exists — try next
      candidate = `${dir}/${base}_${i}${ext}`;
      i++;
    } catch {
      // doesn't exist — safe to use
      return candidate;
    }
  }
}

// ─── Context passed down to every node ───────────────────────────────────────

interface TreeContext {
  selectedPath:   string | null;
  dragPath:       string | null;
  dropTarget:     string | null; // path of folder being hovered over as drop target
  dropBeforePath: string | null; // path of entry to insert before (sibling drop)
  onSelect:       (entry: DirEntry) => void;
  onDragStart:    (path: string) => void;
  onDragOver:     (folderPath: string | null, beforePath: string | null) => void;
  onDrop:         (targetDir: string) => void;
  onRefresh:      (dirPath: string) => void;
  setFocusedDir:  (path: string) => void;
}

// ─── TreeNode ─────────────────────────────────────────────────────────────────

interface TreeNodeProps {
  entry:   DirEntry;
  depth:   number;
  ctx:     TreeContext;
  refresh: number; // increment to force child reload
}

function TreeNode({ entry, depth, ctx, refresh }: TreeNodeProps) {
  const [expanded,    setExpanded]    = useState(false);
  const [children,    setChildren]    = useState<DirEntry[]>([]);
  const [loaded,      setLoaded]      = useState(false);
  const [loading,     setLoading]     = useState(false);
  const [renaming,    setRenaming]    = useState(false);
  const [renameDraft, setRenameDraft] = useState(entry.name);
  const [childRefresh, setChildRefresh] = useState(0);

  const ext         = fileExt(entry.name);
  const isSupported = !entry.is_dir && SUPPORTED_EXTS.has(ext);
  const isSelected  = ctx.selectedPath === entry.path;
  const isDragging  = ctx.dragPath === entry.path;
  const isDropTarget = entry.is_dir && ctx.dropTarget === entry.path;
  const isDropBefore = ctx.dropBeforePath === entry.path;

  const loadChildren = useCallback(async () => {
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
  }, [entry.path]);

  useEffect(() => {
    if (expanded) loadChildren();
  }, [childRefresh, refresh]);

  // Register refresh callback so parent can trigger child reloads
  useEffect(() => {
    if (entry.is_dir) {
      // expose via ctx.onRefresh — see AssetFileTree
    }
  }, []);

  async function handleClick() {
    if (renaming) return;
    if (entry.is_dir) {
      if (!expanded && !loaded) await loadChildren();
      setExpanded(e => !e);
      ctx.setFocusedDir(entry.path);
    } else if (isSupported) {
      ctx.onSelect(entry);
    }
  }

  async function commitRename() {
    const newName = renameDraft.trim();
    if (!newName || newName === entry.name) { setRenaming(false); return; }
    const dir     = entry.path.slice(0, entry.path.lastIndexOf('/'));
    const newPath = `${dir}/${newName}`;
    try {
      await api.renamePath(entry.path, newPath);
      ctx.onRefresh(dir);
    } catch (e) {
      console.error('rename failed:', e);
    }
    setRenaming(false);
  }

  const [confirming, setConfirming] = useState(false);

  async function handleDelete() {
    setConfirming(true);
  }

  async function confirmDelete() {
    setConfirming(false);
    try {
      await api.deletePath(entry.path);
      const dir = entry.path.slice(0, entry.path.lastIndexOf('/'));
      ctx.onRefresh(dir);
    } catch (e) {
      console.error('delete failed:', e);
    }
  }

  // Drag and drop handlers
  function onDragStart(e: React.DragEvent) {
    e.stopPropagation();
    e.dataTransfer.effectAllowed = 'move';
    ctx.onDragStart(entry.path);
  }

  function onDragOver(e: React.DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (entry.is_dir) {
      e.dataTransfer.dropEffect = 'move';
      ctx.onDragOver(entry.path, null);
    } else {
      e.dataTransfer.dropEffect = 'move';
      ctx.onDragOver(null, entry.path);
    }
  }

  function onDrop(e: React.DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    const targetDir = entry.is_dir
      ? entry.path
      : entry.path.slice(0, entry.path.lastIndexOf('/'));
    ctx.onDrop(targetDir);
  }

  function onDragEnd(e: React.DragEvent) {
    ctx.onDragStart(''); // clear drag
    ctx.onDragOver(null, null);
  }

  return (
    <>
      {/* Drop-before indicator */}
      {isDropBefore && (
        <div className="asset-drop-line" style={{ marginLeft: 10 + depth * 14 }} />
      )}

      <div
        className={[
          'asset-tree-row',
          isSelected   ? 'selected'    : '',
          isDragging   ? 'dragging'    : '',
          isDropTarget ? 'drop-target' : '',
          !entry.is_dir && !isSupported ? 'unsupported' : '',
        ].filter(Boolean).join(' ')}
        style={{ paddingLeft: 10 + depth * 14 }}
        draggable
        onClick={handleClick}
        onDragStart={onDragStart}
        onDragOver={onDragOver}
        onDrop={onDrop}
        onDragEnd={onDragEnd}
      >
        {entry.is_dir ? (
          <span className="asset-chevron">
            <svg width="7" height="7" viewBox="0 0 7 7" fill="currentColor"
              style={{ transform: expanded ? 'rotate(90deg)' : 'none', transition: 'transform .12s' }}>
              <polygon points="1,1 6,3.5 1,6" />
            </svg>
          </span>
        ) : (
          <span className="asset-chevron-spacer" />
        )}

        <span className="asset-icon">{fileIcon(entry)}</span>

        {renaming ? (
          <input
            className="asset-rename-input"
            autoFocus
            value={renameDraft}
            onChange={e => setRenameDraft(e.target.value)}
            onBlur={commitRename}
            onKeyDown={e => {
              if (e.key === 'Enter')  commitRename();
              if (e.key === 'Escape') { setRenaming(false); setRenameDraft(entry.name); }
            }}
            onClick={e => e.stopPropagation()}
          />
        ) : (
          <span className="asset-name">{entry.name}</span>
        )}

        {loading && <span className="asset-loading">…</span>}

        {!entry.is_dir && !isSupported && (
          <span className="asset-unsupported-badge">{ext || '?'}</span>
        )}

        {/* Action buttons — shown on hover */}
        <div className="asset-row-actions">
          <button
            className="asset-action-btn"
            title="Rename"
            onClick={e => { e.stopPropagation(); setRenameDraft(entry.name); setRenaming(true); }}
          >
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.3">
              <path d="M7 1.5l1.5 1.5L3 8.5H1.5V7L7 1.5z"/>
            </svg>
          </button>
          <button
            className="asset-action-btn asset-action-delete"
            title="Delete"
            onClick={e => { e.stopPropagation(); handleDelete(); }}
          >
            <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" strokeWidth="1.4">
              <line x1="1" y1="1" x2="8" y2="8"/><line x1="8" y1="1" x2="1" y2="8"/>
            </svg>
          </button>
        </div>
      </div>

      {/* Inline delete confirmation */}
      {confirming && (
        <div className="asset-confirm-row" style={{ paddingLeft: 10 + depth * 14 }}
          onClick={e => e.stopPropagation()}>
          <span className="asset-confirm-msg">
            {entry.is_dir ? `Delete folder + all contents?` : `Delete "${entry.name}"?`}
          </span>
          <button className="asset-confirm-yes" onClick={confirmDelete}>Yes</button>
          <button className="asset-confirm-no"  onClick={() => setConfirming(false)}>No</button>
        </div>
      )}

      {entry.is_dir && expanded && children.map(child => (
        <TreeNode
          key={child.path}
          entry={child}
          depth={depth + 1}
          ctx={ctx}
          refresh={childRefresh}
        />
      ))}
    </>
  );
}

// ─── Top-level AssetFileTree ──────────────────────────────────────────────────

interface AssetFileTreeProps {
  selectedPath: string | null;
  onSelect:     (entry: DirEntry) => void;
}

export function AssetFileTree({ selectedPath, onSelect }: AssetFileTreeProps) {
  const [roots,       setRoots]       = useState<DirEntry[]>([]);
  const [loading,     setLoading]     = useState(true);
  const [error,       setError]       = useState<string | null>(null);
  const [refreshKey,  setRefreshKey]  = useState(0);
  const [focusedDir,  setFocusedDir]  = useState<string>(ASSET_ROOT);

  // Drag state
  const [dragPath,       setDragPath]       = useState<string>('');
  const [dropTarget,     setDropTarget]     = useState<string | null>(null);
  const [dropBeforePath, setDropBeforePath] = useState<string | null>(null);

  useEffect(() => {
    api.listDir(ASSET_ROOT)
      .then(entries => { setRoots(entries); setLoading(false); })
      .catch(e => { setError(String(e)); setLoading(false); });
  }, [refreshKey]);

  function refresh(_dirPath: string) {
    // Simplest correct approach: refresh root which cascades
    setRefreshKey(k => k + 1);
  }

  async function handleDrop(targetDir: string) {
    setDropTarget(null);
    setDropBeforePath(null);
    if (!dragPath || dragPath === targetDir) return;

    // Prevent dropping a folder into itself or its descendants
    if (targetDir.startsWith(dragPath + '/')) return;

    const name    = dragPath.slice(dragPath.lastIndexOf('/') + 1);
    const dstBase = `${targetDir}/${name}`;

    // Resolve conflicts with _N suffix
    let dst = dstBase;
    let i   = 1;
    const ext  = fileExt(name);
    const base = ext ? name.slice(0, name.length - ext.length) : name;
    while (true) {
      try {
        await api.readFileBytes(dst);
        // exists — try next
        dst = `${targetDir}/${base}_${i}${ext}`;
        i++;
      } catch {
        // Check if it's a dir conflict
        try {
          await api.listDir(dst);
          dst = `${targetDir}/${base}_${i}`;
          i++;
        } catch {
          break; // safe to use
        }
      }
    }

    try {
      await api.movePath(dragPath, dst);
      refresh(targetDir);
    } catch (e) {
      console.error('move failed:', e);
    }
    setDragPath('');
  }

  async function handleImport() {
    const src = await api.pickFile();
    if (!src) return;
    const name    = src.slice(src.lastIndexOf('/') + 1);
    const ext     = fileExt(name);
    const base    = ext ? name.slice(0, name.length - ext.length) : name;
    let   dst     = `${focusedDir}/${name}`;
    let   i       = 1;
    while (true) {
      try {
        await api.readFileBytes(dst);
        dst = `${focusedDir}/${base}_${i}${ext}`;
        i++;
      } catch {
        break;
      }
    }
    try {
      await api.copyFile(src, dst);
      refresh(focusedDir);
    } catch (e) {
      console.error('import failed:', e);
    }
  }

  async function handleNewComp() {
    const base = 'new_prefab';
    let   name = `${base}.comp`;
    let   path = `${focusedDir}/${name}`;
    let   i    = 1;
    while (true) {
      try {
        await api.readFileBytes(path);
        name = `${base}_${i}.comp`;
        path = `${focusedDir}/${name}`;
        i++;
      } catch {
        break;
      }
    }
    try {
      await api.createCompFile(path);
      refresh(focusedDir);
    } catch (e) {
      console.error('create comp failed:', e);
    }
  }

  const ctx: TreeContext = {
    selectedPath,
    dragPath,
    dropTarget,
    dropBeforePath,
    onSelect,
    onDragStart:   setDragPath,
    onDragOver:    (folder, before) => { setDropTarget(folder); setDropBeforePath(before); },
    onDrop:        handleDrop,
    onRefresh:     refresh,
    setFocusedDir,
  };

  return (
    <div className="asset-tree-wrap"
      onDragOver={e => e.preventDefault()}
      onDrop={e => { e.preventDefault(); handleDrop(focusedDir); }}
    >
      {/* Toolbar */}
      <div className="asset-tree-toolbar">
        <button className="asset-toolbar-btn" onClick={handleImport} title="Import file">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3">
            <polyline points="6,1 6,8"/><polyline points="3,5 6,8 9,5"/>
            <polyline points="1,10 11,10"/>
          </svg>
          Import
        </button>
        <button className="asset-toolbar-btn" onClick={handleNewComp} title="New prefab (.comp)">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3">
            <rect x="2" y="2" width="8" height="8" rx="1"/>
            <line x1="6" y1="4" x2="6" y2="8"/><line x1="4" y1="6" x2="8" y2="6"/>
          </svg>
          New Comp
        </button>
      </div>

      {/* Tree */}
      <div className="asset-tree">
        {loading && <div className="panel-empty">Loading…</div>}
        {error   && <div className="panel-empty asset-error">{error}</div>}
        {!loading && !error && roots.length === 0 && <div className="panel-empty">Empty folder</div>}
        {!loading && !error && roots.map(entry => (
          <TreeNode key={entry.path} entry={entry} depth={0} ctx={ctx} refresh={refreshKey} />
        ))}
      </div>
    </div>
  );
}
