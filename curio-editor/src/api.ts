import type { FormsSnapshot, LedgerSnapshot, PlayMode } from './types';

// ─────────────────────────────────────────────────────────────
// Mock data (used when running outside Tauri in the browser)
// ─────────────────────────────────────────────────────────────

const MOCK_FORMS: FormsSnapshot = {
  forms: [
    {
      id: 1, name: 'World', children: [
        {
          id: 2, name: 'Player', children: [], components: [
            { name: 'Transform', fields: { x: 0.0, y: 1.2, z: 0.0, scale: 1.0 } },
            { name: 'Health',    fields: { max: 100, current: 87 } },
            { name: 'Velocity',  fields: { dx: 0.0, dy: -0.1 } },
          ],
        },
        {
          id: 3, name: 'Camera', children: [], components: [
            { name: 'Transform', fields: { x: 0.0, y: 5.0, z: -10.0, scale: 1.0 } },
            { name: 'Camera',    fields: { fov: 60.0, near: 0.1, far: 1000.0 } },
          ],
        },
        {
          id: 4, name: 'Environment', children: [
            {
              id: 5, name: 'Ground', children: [], components: [
                { name: 'Transform', fields: { x: 0.0, y: 0.0, z: 0.0, scale: 10.0 } },
                { name: 'Mesh',      fields: { path: 'mesh/court.glb', visible: true } },
              ],
            },
            {
              id: 6, name: 'Sun', children: [], components: [
                { name: 'Transform', fields: { x: 10.0, y: 20.0, z: 5.0, scale: 1.0 } },
                { name: 'Light',     fields: { intensity: 1.0, color: '#ffffff', shadows: true } },
              ],
            },
          ],
          components: [],
        },
      ],
      components: [],
    },
  ],
};

const MOCK_LEDGER: LedgerSnapshot = {
  instances: [
    {
      id: 0, name: 'Host', role: 'host',
      records: [
        { name: 'Time',      record_type: 'SysRecordTime',      permissions: 'readwrite', value: { delta: 0.016, elapsed: 12.4 } },
        { name: 'Camera',    record_type: 'SysRecordCamera',    permissions: 'readwrite', value: { fov: 60, near: 0.1, far: 1000 } },
        { name: 'Rendering', record_type: 'SysRecordRendering', permissions: 'read',      value: { draw_calls: 12 } },
        { name: 'Input',     record_type: 'SysRecordInput',     permissions: 'read',      value: { axis: 'cursor' } },
        { name: 'Debug',     record_type: 'SysRecordDebug',     permissions: 'readwrite', value: { show_gizmos: true } },
      ],
    },
    {
      id: 1, name: 'Peer 1', role: 'peer',
      records: [
        { name: 'Input',    record_type: 'SysRecordInput',    permissions: 'readwrite', value: { key: 'W', state: 'down' } },
        { name: 'Network',  record_type: 'SysRecordNetwork',  permissions: 'read',      value: { latency_ms: 14 } },
        { name: 'Camera',   record_type: 'SysRecordCamera',   permissions: 'read',      value: { fov: 60 } },
      ],
    },
  ],
};

// ─────────────────────────────────────────────────────────────
// Tauri detection
// ─────────────────────────────────────────────────────────────

const isTauri = (): boolean => '__TAURI_INTERNALS__' in window;

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) throw new Error(`Not in Tauri — command: ${cmd}`);
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
  return tauriInvoke<T>(cmd, args);
}

// ─────────────────────────────────────────────────────────────
// API
// ─────────────────────────────────────────────────────────────

export const api = {
  pressPlay:  (): Promise<void> => isTauri() ? invoke('press_play')  : Promise.resolve(),
  pressStop:  (): Promise<void> => isTauri() ? invoke('press_stop')  : Promise.resolve(),
  pressPause: (): Promise<void> => isTauri() ? invoke('press_pause') : Promise.resolve(),

  getForms: async (): Promise<FormsSnapshot> => {
    if (!isTauri()) return MOCK_FORMS;
    return invoke<FormsSnapshot>('get_forms');
  },

  getLedgerSnapshot: async (): Promise<LedgerSnapshot> => {
    if (!isTauri()) return MOCK_LEDGER;
    return invoke<LedgerSnapshot>('get_ledger_snapshot');
  },

  onViewportFrame: (cb: (dataUrl: string) => void): (() => void) => {
    if (!isTauri()) return () => {};
    let unlisten: Promise<() => void>;
    import('@tauri-apps/api/event').then(({ listen }) => {
      unlisten = listen<string>('viewport_frame', (e) => {
        cb(`data:image/png;base64,${e.payload}`);
      });
    });
    return () => { unlisten?.then(f => f()); };
  },
};
