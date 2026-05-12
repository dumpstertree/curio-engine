import type { SceneSnapshot } from './types';

// ─────────────────────────────────────────────────────────────
// Mock data — used when running outside Tauri (browser dev)
// ─────────────────────────────────────────────────────────────
const MOCK_SNAPSHOT: SceneSnapshot = {
  entities: [
    {
      id: 1,
      name: 'World',
      children: [
        {
          id: 2,
          name: 'Player',
          children: [],
          components: [
            { name: 'Transform', fields: { x: 0.0, y: 1.2, z: 0.0, scale: 1.0 } },
            { name: 'Health',    fields: { max: 100, current: 87 } },
            { name: 'Velocity',  fields: { dx: 0.0, dy: -0.1 } },
          ],
        },
        {
          id: 3,
          name: 'Camera',
          children: [],
          components: [
            { name: 'Transform', fields: { x: 0.0, y: 5.0, z: -10.0, scale: 1.0 } },
            { name: 'Camera',    fields: { fov: 60.0, near: 0.1, far: 1000.0 } },
          ],
        },
        {
          id: 4,
          name: 'Environment',
          children: [
            {
              id: 5,
              name: 'Ground',
              children: [],
              components: [
                { name: 'Transform', fields: { x: 0.0, y: 0.0, z: 0.0, scale: 10.0 } },
                { name: 'Mesh',      fields: { path: 'mesh/court.glb', visible: true } },
              ],
            },
            {
              id: 6,
              name: 'Sun',
              children: [],
              components: [
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

// ─────────────────────────────────────────────────────────────
// Detect Tauri environment
// ─────────────────────────────────────────────────────────────
const isTauri = (): boolean => '__TAURI_INTERNALS__' in window;

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new Error(`Not in Tauri — command: ${cmd}`);
  }
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
  return tauriInvoke<T>(cmd, args);
}

// ─────────────────────────────────────────────────────────────
// API — each fn falls back to mock data outside Tauri
// ─────────────────────────────────────────────────────────────
export const api = {
  pressPlay: async (): Promise<void> => {
    if (!isTauri()) return;
    return invoke('press_play');
  },

  pressStop: async (): Promise<void> => {
    if (!isTauri()) return;
    return invoke('press_stop');
  },

  pressPause: async (): Promise<void> => {
    if (!isTauri()) return;
    return invoke('press_pause');
  },

  getSceneSnapshot: async (): Promise<SceneSnapshot> => {
    if (!isTauri()) return MOCK_SNAPSHOT;
    return invoke<SceneSnapshot>('get_scene_snapshot');
  },
};
