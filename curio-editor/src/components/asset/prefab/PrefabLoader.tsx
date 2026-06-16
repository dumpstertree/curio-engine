import React, { useEffect, useRef, useState, useCallback } from 'react';
import { load as yamlLoad, dump as yamlDump } from 'js-yaml';
import { api } from '../../../api';
import type { PrefabGameObjectRaw } from './prefabTypes';
import type { ResolvedGameObject } from './prefabResolver';
import { resolveNode } from './prefabResolver';
import { PrefabViewport } from './PrefabViewport';
import { PrefabInspectorView } from './PrefabInspectorView';

const ASSET_ROOT = '/home/dumpstertree/Git/Rust/system_test/assets';
function toAssetRel(absPath: string): string {
  return absPath.startsWith(ASSET_ROOT + '/')
    ? absPath.slice(ASSET_ROOT.length + 1)
    : absPath;
}

function normalize(raw: any): PrefabGameObjectRaw {
  return {
    enabled:    raw?.enabled ?? true,
    name:       raw?.name ?? 'GameObject',
    base:       typeof raw?.base === 'string' && raw.base.trim() ? raw.base.trim() : undefined,
    components: Array.isArray(raw?.components) ? raw.components.map((c: any) => ({
      type:   c?.type ?? '',
      fields: Array.isArray(c?.fields) ? c.fields.map((f: any) => String(f)) : [],
    })) : [],
    children: Array.isArray(raw?.children) ? raw.children.map(normalize) : [],
  };
}

interface Props {
  path: string;
  name: string;
}

export function PrefabLoader({ path, name }: Props) {
  const [raw,        setRaw]        = useState<PrefabGameObjectRaw | null>(null);
  const [resolved,   setResolved]   = useState<ResolvedGameObject | null>(null);
  const [loading,    setLoading]    = useState(true);
  const [error,      setError]      = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const selfPath  = toAssetRel(path);

  useEffect(() => {
    setLoading(true);
    setError(null);
    setRaw(null);
    setResolved(null);

    api.readFileBytes(path)
      .then(async bytes => {
        const text    = new TextDecoder('utf-8').decode(new Uint8Array(bytes));
        const rawNode = normalize(yamlLoad(text));
        setRaw(rawNode);
        // Resolve for viewport only
        const res = await resolveNode(rawNode, selfPath);
        setResolved(res);
        setLoading(false);
      })
      .catch(e => { setError(String(e)); setLoading(false); });

    return () => { if (saveTimer.current) clearTimeout(saveTimer.current); };
  }, [path, refreshKey]);

  const handleRefresh = useCallback(() => setRefreshKey(k => k + 1), []);

  // Inspector edits the raw node directly
  const handleRawChange = useCallback(async (next: PrefabGameObjectRaw) => {
    setRaw(next);

    // Re-resolve for viewport whenever raw changes
    try {
      const res = await resolveNode(next, selfPath);
      setResolved(res);
    } catch (e) {
      console.error('[PrefabLoader] resolve failed:', e);
    }

    if (saveTimer.current) clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(() => {
      const text = yamlDump(next, { lineWidth: -1 });
      api.writeFileText(path, text).catch(e =>
        console.error('[PrefabLoader] save failed:', e)
      );
    }, 300);
  }, [path, selfPath]);

  return (
    <>
      <div className="center-panel">
        <div className="center-viewport">
          {loading && <div className="asset-viewport-overlay">Loading…</div>}
          {error   && <div className="asset-viewport-overlay asset-error">{error}</div>}
          {!loading && !error && resolved && <PrefabViewport root={resolved} />}
        </div>
      </div>

      <PrefabInspectorView
        fileName={name}
        raw={raw}
        onChange={handleRawChange}
        onRefresh={handleRefresh}
      />
    </>
  );
}
