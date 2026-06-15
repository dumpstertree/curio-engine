import React, { useEffect, useRef, useState } from 'react';
import { load as yamlLoad, dump as yamlDump } from 'js-yaml';
import { api } from '../../../api';
import type { PrefabGameObjectRaw } from './prefabTypes';
import { PrefabViewport } from './PrefabViewport';
import { PrefabInspectorView } from './PrefabInspectorView';

interface Props {
  path: string;
  name: string;
}

/** Normalizes a freshly-parsed YAML object to guarantee required fields exist,
 *  since hand-written prefab YAML may omit `enabled` or use partial structures. */
function normalize(raw: any): PrefabGameObjectRaw {
  return {
    enabled: raw?.enabled ?? true,
    name: raw?.name ?? 'GameObject',
    components: Array.isArray(raw?.components) ? raw.components.map((c: any) => ({
      type: c?.type ?? '',
      fields: Array.isArray(c?.fields) ? c.fields.map((f: any) => String(f)) : [],
    })) : [],
    children: Array.isArray(raw?.children) ? raw.children.map(normalize) : [],
  };
}

export function PrefabLoader({ path, name }: Props) {
  const [root,    setRoot]    = useState<PrefabGameObjectRaw | null>(null);
  const [loading, setLoading] = useState(true);
  const [error,   setError]   = useState<string | null>(null);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    setLoading(true);
    setError(null);
    setRoot(null);

    api.readFileBytes(path)
      .then(bytes => {
        const text = new TextDecoder('utf-8').decode(new Uint8Array(bytes));
        const parsed = yamlLoad(text);
        setRoot(normalize(parsed));
        setLoading(false);
      })
      .catch(e => { setError(String(e)); setLoading(false); });

    return () => {
      if (saveTimer.current) clearTimeout(saveTimer.current);
    };
  }, [path]);

  function handleChange(next: PrefabGameObjectRaw) {
    setRoot(next);

    // Debounce writes slightly so rapid edits (e.g. typing) don't thrash the filesystem.
    if (saveTimer.current) clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(() => {
      const text = yamlDump(next, { lineWidth: -1 });
      api.writeFileText(path, text).catch(e => {
        console.error('[PrefabLoader] failed to save:', e);
      });
    }, 300);
  }

  return (
    <>
      {/* Center: viewport */}
      <div className="center-panel">
        <div className="center-viewport">
          {loading && <div className="asset-viewport-overlay">Loading…</div>}
          {error   && <div className="asset-viewport-overlay asset-error">{error}</div>}
          {!loading && !error && root && <PrefabViewport root={root} />}
        </div>
      </div>

      {/* Right: inspector */}
      <PrefabInspectorView
        fileName={name}
        root={root}
        onChange={handleChange}
      />
    </>
  );
}
