import React, { useEffect, useState, useRef, useCallback } from 'react';
import { api } from '../../api';
import type { DirEntry, MetaFile } from '../../api';

import { getAssetsRoot } from '../../paths';
const ASSET_ROOT = getAssetsRoot();
const SUPPORTED_EXTS = new Set(['.png', '.glb', '.anim', '.comp']);

function fileExt(name: string): string {
  const dot = name.lastIndexOf('.');
  return dot >= 0 ? name.slice(dot).toLowerCase() : '';
}

function isMeta(name: string): boolean {
  return name.endsWith('.meta');
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

// ── ID generation ─────────────────────────────────────────────────────────────

function randomId(): number {
  return Math.floor(Math.random() * 32767) + 1;
}

async function getOrCreateMeta(assetPath: string): Promise<MetaFile> {
  const existing = await api.readMeta(assetPath);
  if (existing) return existing;
  const meta: MetaFile = { id: randomId(), included: true };
  await api.writeMeta(assetPath, meta);
  return meta;
}

// ── Context ───────────────────────────────────────────────────────────────────

interface TreeContext {
  selectedPath:   string | null;
  dragPath:       string;
  dropTarget:     string | null;
  dropBeforePath: string | null;
  onSelect:       (entry: DirEntry) => void;
  onDragStart:    (path: string) => void;
  onDragOver:     (folder: string | null, before: string | null) => void;
  onDrop:         (targetDir: string) => void;
  onRefresh:      (dirPath: string) => void;
  setFocusedDir:  (path: string) => void;
}

// ── MetaCheckbox — loads/creates meta, shows include toggle ──────────────────

function MetaCheckbox({ assetPath, onClick }: { assetPath: string; onClick: (e: React.MouseEvent) => void }) {
  const [meta,    setMeta]    = useState<MetaFile | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getOrCreateMeta(assetPath)
      .then(m => { setMeta(m); setLoading(false); })
      .catch(() => setLoading(false));
  }, [assetPath]);

  async function toggle(e: React.MouseEvent) {
    e.stopPropagation();
    if (!meta) return;
    const next = { ...meta, included: !meta.included };
    setMeta(next);
    await api.writeMeta(assetPath, next);
    await api.rebuildManifest();
  }

  if (loading) return <span className="asset-meta-placeholder" />;

  return (
    <input
      type="checkbox"
      className="asset-include-check"
      checked={meta?.included ?? true}
      title={meta?.included ? `Included (ID: ${meta?.id})` : `Excluded (ID: ${meta?.id})`}
      onChange={() => {}}
      onClick={toggle}
    />
  );
}

// ── TreeNode ──────────────────────────────────────────────────────────────────

interface TreeNodeProps {
  entry:   DirEntry;
  depth:   number;
  ctx:     TreeContext;
  refresh: number;
}

function TreeNode({ entry, depth, ctx, refresh }: TreeNodeProps) {
  const [expanded,     setExpanded]     = useState(false);
  const [children,     setChildren]     = useState<DirEntry[]>([]);
  const [loaded,       setLoaded]       = useState(false);
  const [loading,      setLoading]      = useState(false);
  const [renaming,     setRenaming]     = useState(false);
  const [renameDraft,  setRenameDraft]  = useState(entry.name);
  const [confirming,   setConfirming]   = useState(false);
  const [childRefresh, setChildRefresh] = useState(0);

  const ext         = fileExt(entry.name);
  const isSupported = !entry.is_dir && SUPPORTED_EXTS.has(ext);
  const isSelected  = ctx.selectedPath === entry.path;
  const isDragging  = ctx.dragPath === entry.path;
  const isDropTarget = entry.is_dir && ctx.dropTarget === entry.path;

  const loadChildren = useCallback(async () => {
    setLoading(true);
    try {
      const entries = await api.listDir(entry.path);
      // Hide .meta files
      setChildren(entries.filter(e => !isMeta(e.name)));
      setLoaded(true);
    } catch (e) { console.error('listDir failed:', e); }
    finally { setLoading(false); }
  }, [entry.path]);

  useEffect(() => {
    if (expanded) loadChildren();
  }, [childRefresh, refresh]);

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
      // Move .meta file too if it exists (files only)
      if (!entry.is_dir) {
        try { await api.renamePath(entry.path + '.meta', newPath + '.meta'); } catch {}
      }
      ctx.onRefresh(dir);
      await api.rebuildManifest();
    } catch (e) { console.error('rename failed:', e); }
    setRenaming(false);
  }

  async function confirmDelete() {
    setConfirming(false);
    try {
      await api.deletePath(entry.path);
      // Delete .meta too
      if (!entry.is_dir) {
        try { await api.deletePath(entry.path + '.meta'); } catch {}
      }
      const dir = entry.path.slice(0, entry.path.lastIndexOf('/'));
      ctx.onRefresh(dir);
      await api.rebuildManifest();
    } catch (e) { console.error('delete failed:', e); }
  }

  function onDragStart(e: React.DragEvent) {
    e.stopPropagation();
    e.dataTransfer.effectAllowed = 'move';
    ctx.onDragStart(entry.path);
  }

  function onDragOver(e: React.DragEvent) {
    e.preventDefault(); e.stopPropagation();
    if (entry.is_dir) ctx.onDragOver(entry.path, null);
    else              ctx.onDragOver(null, entry.path);
  }

  function onDrop(e: React.DragEvent) {
    e.preventDefault(); e.stopPropagation();
    const targetDir = entry.is_dir
      ? entry.path
      : entry.path.slice(0, entry.path.lastIndexOf('/'));
    ctx.onDrop(targetDir);
  }

  return (
    <>
      {ctx.dropBeforePath === entry.path && (
        <div className="asset-drop-line" style={{ marginLeft: 10 + depth * 14 }} />
      )}

      <div
        className={[
          'asset-tree-row',
          isSelected    ? 'selected'    : '',
          isDragging    ? 'dragging'    : '',
          isDropTarget  ? 'drop-target' : '',
          !entry.is_dir && !isSupported ? 'unsupported' : '',
        ].filter(Boolean).join(' ')}
        style={{ paddingLeft: 10 + depth * 14 }}
        draggable
        onClick={handleClick}
        onDragStart={onDragStart}
        onDragOver={onDragOver}
        onDrop={onDrop}
        onDragEnd={() => { ctx.onDragStart(''); ctx.onDragOver(null, null); }}
      >
        {entry.is_dir ? (
          <span className="asset-chevron">
            <svg width="7" height="7" viewBox="0 0 7 7" fill="currentColor"
              style={{ transform: expanded ? 'rotate(90deg)' : 'none', transition: 'transform .12s' }}>
              <polygon points="1,1 6,3.5 1,6"/>
            </svg>
          </span>
        ) : (
          <span className="asset-chevron-spacer"/>
        )}

        {/* Include checkbox — only for files */}
        {!entry.is_dir && (
          <MetaCheckbox assetPath={entry.path} onClick={e => e.stopPropagation()} />
        )}

        <span className="asset-icon">{fileIcon(entry)}</span>

        {renaming ? (
          <input className="asset-rename-input" autoFocus
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

        <div className="asset-row-actions">
          <button className="asset-action-btn" title="Rename"
            onClick={e => { e.stopPropagation(); setRenameDraft(entry.name); setRenaming(true); }}>
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.3">
              <path d="M7 1.5l1.5 1.5L3 8.5H1.5V7L7 1.5z"/>
            </svg>
          </button>
          <button className="asset-action-btn asset-action-delete" title="Delete"
            onClick={e => { e.stopPropagation(); setConfirming(true); }}>
            <svg width="9" height="9" viewBox="0 0 9 9" fill="none" stroke="currentColor" strokeWidth="1.4">
              <line x1="1" y1="1" x2="8" y2="8"/><line x1="8" y1="1" x2="1" y2="8"/>
            </svg>
          </button>
        </div>
      </div>

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
        <TreeNode key={child.path} entry={child} depth={depth + 1} ctx={ctx} refresh={childRefresh} />
      ))}
    </>
  );
}

// ── Top-level AssetFileTree ───────────────────────────────────────────────────

interface AssetFileTreeProps {
  selectedPath: string | null;
  onSelect:     (entry: DirEntry) => void;
}

export function AssetFileTree({ selectedPath, onSelect }: AssetFileTreeProps) {
  const [roots,      setRoots]      = useState<DirEntry[]>([]);
  const [loading,    setLoading]    = useState(true);
  const [error,      setError]      = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const [focusedDir, setFocusedDir] = useState<string>(ASSET_ROOT);
  const [dragPath,       setDragPath]       = useState('');
  const [dropTarget,     setDropTarget]     = useState<string | null>(null);
  const [dropBeforePath, setDropBeforePath] = useState<string | null>(null);

  useEffect(() => {
    api.listDir(ASSET_ROOT)
      .then(entries => {
        setRoots(entries.filter(e => !isMeta(e.name)));
        setLoading(false);
      })
      .catch(e => { setError(String(e)); setLoading(false); });
  }, [refreshKey]);

  function refresh(_dir: string) { setRefreshKey(k => k + 1); }

  async function resolveConflict(dir: string, name: string): Promise<string> {
    const ext  = fileExt(name);
    const base = ext ? name.slice(0, name.length - ext.length) : name;
    let dst = `${dir}/${name}`;
    let i   = 1;
    while (true) {
      try { await api.readFileBytes(dst); dst = `${dir}/${base}_${i}${ext}`; i++; }
      catch { try { await api.listDir(dst); dst = `${dir}/${base}_${i}`; i++; } catch { break; } }
    }
    return dst;
  }

  async function handleDrop(targetDir: string) {
    setDropTarget(null); setDropBeforePath(null);
    if (!dragPath || dragPath === targetDir) return;
    if (targetDir.startsWith(dragPath + '/')) return;
    const name = dragPath.slice(dragPath.lastIndexOf('/') + 1);
    const dst  = await resolveConflict(targetDir, name);
    try {
      await api.movePath(dragPath, dst);
      // Move .meta if file
      try { await api.movePath(dragPath + '.meta', dst + '.meta'); } catch {}
      refresh(targetDir);
      await api.rebuildManifest();
    } catch (e) { console.error('move failed:', e); }
    setDragPath('');
  }

  async function handleImport() {
    const src = await api.pickFile();
    if (!src) return;
    const name = src.slice(src.lastIndexOf('/') + 1);
    const dst  = await resolveConflict(focusedDir, name);
    try {
      await api.copyFile(src, dst);
      // Create fresh meta for imported file
      await getOrCreateMeta(dst);
      refresh(focusedDir);
      await api.rebuildManifest();
    } catch (e) { console.error('import failed:', e); }
  }

  async function handleNewFolder() {
    const dst = await resolveConflict(focusedDir, 'new_folder');
    try {
      await api.createFolder(dst);
      refresh(focusedDir);
    } catch (e) { console.error('create folder failed:', e); }
  }

  async function handleNewComp() {
    const dst = await resolveConflict(focusedDir, 'new_prefab.comp');
    try {
      await api.createCompFile(dst);
      await getOrCreateMeta(dst);
      refresh(focusedDir);
      await api.rebuildManifest();
    } catch (e) { console.error('create comp failed:', e); }
  }

  const ctx: TreeContext = {
    selectedPath, dragPath, dropTarget, dropBeforePath, onSelect,
    onDragStart:  setDragPath,
    onDragOver:   (folder, before) => { setDropTarget(folder); setDropBeforePath(before); },
    onDrop:       handleDrop,
    onRefresh:    refresh,
    setFocusedDir,
  };

  return (
    <div className="asset-tree-wrap"
      onDragOver={e => e.preventDefault()}
      onDrop={e => { e.preventDefault(); handleDrop(focusedDir); }}>

      <div className="asset-tree-toolbar">
        <button className="asset-toolbar-btn" onClick={handleImport} title="Import file">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3">
            <polyline points="6,1 6,8"/><polyline points="3,5 6,8 9,5"/>
            <polyline points="1,10 11,10"/>
          </svg>
          Import
        </button>
        <button className="asset-toolbar-btn" onClick={handleNewFolder} title="New folder">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3">
            <path d="M1 3.5C1 2.9 1.4 2.5 2 2.5h3l1 1.5h5c.6 0 1 .4 1 1V9c0 .6-.4 1-1 1H2c-.6 0-1-.4-1-1V3.5z"/>
            <line x1="6" y1="5.5" x2="6" y2="8.5"/><line x1="4.5" y1="7" x2="7.5" y2="7"/>
          </svg>
          New Folder
        </button>
        <button className="asset-toolbar-btn" onClick={handleNewComp} title="New prefab (.comp)">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3">
            <rect x="2" y="2" width="8" height="8" rx="1"/>
            <line x1="6" y1="4" x2="6" y2="8"/><line x1="4" y1="6" x2="8" y2="6"/>
          </svg>
          New Comp
        </button>
      </div>

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
