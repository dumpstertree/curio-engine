import type { TabGroupState } from './types';

// ─────────────────────────────────────────────────────────────
// Mock data — matches Rust serialization exactly
// ─────────────────────────────────────────────────────────────

const MOCK: TabGroupState = {
  id_for_tabs: {
    'Host': [
      {
        tab_name: 'Ledger',
        objects: [
          {
            object_name: 'Time',
            children: [],
            components: [{ component_name: 'SysRecordTime', fields: [{ field_name: 'delta', data: 0.016 }, { field_name: 'elapsed', data: 12.4 }] }],
          },
          {
            object_name: 'Camera',
            children: [],
            components: [{ component_name: 'SysRecordCamera', fields: [{ field_name: 'fov', data: 60 }, { field_name: 'near', data: 0.1 }, { field_name: 'far', data: 1000 }] }],
          },
          {
            object_name: 'Rendering',
            children: [],
            components: [{ component_name: 'SysRecordRendering', fields: [{ field_name: 'draw_calls', data: 12 }] }],
          },
        ],
      },
      {
        tab_name: 'Forms',
        objects: [
          {
            object_name: 'World',
            components: [],
            children: [
              {
                object_name: 'Player',
                children: [],
                components: [
                  { component_name: 'Transform', fields: [{ field_name: 'x', data: 0 }, { field_name: 'y', data: 1.2 }, { field_name: 'z', data: 0 }] },
                  { component_name: 'Health', fields: [{ field_name: 'max', data: 100 }, { field_name: 'current', data: 87 }] },
                ],
              },
            ],
          },
        ],
      },
    ],
  },
};

// ─────────────────────────────────────────────────────────────
// Tauri detection
// ─────────────────────────────────────────────────────────────

const isTauri = (): boolean => '__TAURI_INTERNALS__' in window;

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) throw new Error(`Not in Tauri: ${cmd}`);
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
  return tauriInvoke<T>(cmd, args);
}

// ─────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────

export interface DirEntry {
  name: string;
  path: string;
  is_dir: boolean;
}

// ─────────────────────────────────────────────────────────────
// API
// ─────────────────────────────────────────────────────────────

export const api = {
  pressPlay:  (): Promise<void> => isTauri() ? invoke('press_play')  : Promise.resolve(),
  pressStop:  (): Promise<void> => isTauri() ? invoke('press_stop')  : Promise.resolve(),
  pressPause: (): Promise<void> => isTauri() ? invoke('press_pause') : Promise.resolve(),

  getTabGroupState: async (): Promise<TabGroupState> => {
    if (!isTauri()) return MOCK;
    return invoke<TabGroupState>('get_tab_group_state');
  },

  listDir: async (path: string): Promise<DirEntry[]> => {
    return invoke<DirEntry[]>('list_dir', { path });
  },

  readFileBytes: async (path: string): Promise<number[]> => {
    return invoke<number[]>('read_file_bytes', { path });
  },

  writeFileText: async (path: string, contents: string): Promise<void> => {
    return invoke<void>('write_file_text', { path, contents });
  },

  copyFile: async (src: string, dst: string): Promise<void> => {
    return invoke<void>('copy_file', { src, dst });
  },

  createCompFile: async (path: string): Promise<void> => {
    return invoke<void>('create_comp_file', { path });
  },

  deletePath: async (path: string): Promise<void> => {
    return invoke<void>('delete_path', { path });
  },

  renamePath: async (oldPath: string, newPath: string): Promise<void> => {
    return invoke<void>('rename_path', { oldPath, newPath });
  },

  movePath: async (src: string, dst: string): Promise<void> => {
    return invoke<void>('move_path', { src, dst });
  },

  pickFile: async (): Promise<string | null> => {
    if (!isTauri()) return null;
    const { open } = await import('@tauri-apps/plugin-dialog');
    const result = await open({ multiple: false, directory: false });
    return typeof result === 'string' ? result : null;
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
