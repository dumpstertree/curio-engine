import type { TabGroupState } from './types';
import { useEditorStore } from './store';

// ─────────────────────────────────────────────────────────────────────────────
// Mock data — matches Rust serialization exactly
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Tauri detection
// ─────────────────────────────────────────────────────────────────────────────

const isTauri = (): boolean => '__TAURI_INTERNALS__' in window;

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) throw new Error(`Not in Tauri: ${cmd}`);
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
  return tauriInvoke<T>(cmd, args);
}

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

export interface DirEntry {
  name: string;
  path: string;
  is_dir: boolean;
}

export interface MetaFile {
  id: number;
  included: boolean;
}

export interface ManifestEntry {
  id: number;
  name: string;
  type: string;
  uri: string;
}

export type EntryType =
  | { Asset: string }
  | 'Float'
  | 'Int'
  | 'Bool'
  | 'Vector2'
  | 'Vector3'
  | 'Vector4';

export interface FacetField {
  name: string;
  data: EntryType;
}

export interface FacetComponent {
  name: string;
  data: FacetField[];
}

export interface FacetManifest {
  manifest: FacetComponent[];
}

export type InputEvent =
  | { type: 'Button'; code: number; pressed: boolean }
  | { type: 'Axis'; code: number; x: number; y: number };

// ─────────────────────────────────────────────────────────────────────────────
// API
// ─────────────────────────────────────────────────────────────────────────────

export const api = {
  // Lifecycle
  initialize: (): Promise<void> =>
    isTauri() ? invoke('initialize') : Promise.resolve(),

  // Playback
  pressPlay: (): Promise<void> => isTauri() ? invoke('press_play') : Promise.resolve(),
  pressStop: (): Promise<void> => isTauri() ? invoke('press_stop') : Promise.resolve(),
  pressPause: (): Promise<void> => isTauri() ? invoke('press_pause') : Promise.resolve(),
  pressPlayStart: (): Promise<void> => isTauri() ? invoke('press_play_start') : Promise.resolve(),

  // Compile
  compile: (): Promise<void> => isTauri() ? invoke('compile') : Promise.resolve(),
  getCompileStatus: (): Promise<string> => isTauri() ? invoke('get_compile_status') : Promise.resolve('idle'),
  cancelCompile: (): Promise<void> => isTauri() ? invoke('cancel_compile') : Promise.resolve(),

  // Viewport — poll for raw RGBA frame bytes, null if no new frame
  getFrame: (): Promise<number[] | null> =>
    isTauri() ? invoke<number[] | null>('get_frame') : Promise.resolve(null),

  // Input
  sendInput: (event: InputEvent): Promise<void> =>
    isTauri() ? invoke('send_input', { event }) : Promise.resolve(),

  // Inspector / tab group
  getTabGroupState: async (): Promise<TabGroupState> => {
    if (!isTauri()) return MOCK;
    return invoke<TabGroupState>('get_tab_group_state');
  },

  // Project
  getProjectPath: (): Promise<string> =>
    invoke<string>('get_project_path'),

  // Logs
  getLogs: async (): Promise<[string, string][]> => {
    if (!isTauri()) return [];
    return invoke<[string, string][]>('get_logs');
  },

  // Facets
  getFacets: async (): Promise<FacetManifest> => {
    if (!isTauri()) return { manifest: [] };
    return invoke<FacetManifest>('get_facets');
  },

  // File system
  listDir: (path: string): Promise<DirEntry[]> =>
    invoke<DirEntry[]>('list_dir', { path }),

  readFileBytes: (path: string): Promise<number[]> =>
    invoke<number[]>('read_file_bytes', { path }),

  writeFileText: (path: string, contents: string): Promise<void> =>
    invoke<void>('write_file_text', { path, contents }),

  copyFile: (src: string, dst: string): Promise<void> =>
    invoke<void>('copy_file', { src, dst }),

  createCompFile: (path: string): Promise<void> =>
    invoke<void>('create_comp_file', { path }),

  createFolder: (path: string): Promise<void> =>
    invoke<void>('create_folder', { path }),

  deletePath: (path: string): Promise<void> =>
    invoke<void>('delete_path', { path }),

  renamePath: (oldPath: string, newPath: string): Promise<void> =>
    invoke<void>('rename_path', { oldPath, newPath }),

  movePath: (src: string, dst: string): Promise<void> =>
    invoke<void>('move_path', { src, dst }),

  // Meta files
  readMeta: async (assetPath: string): Promise<MetaFile | null> => {
    try {
      const bytes = await invoke<number[]>('read_file_bytes', { path: assetPath + '.meta' });
      const text = new TextDecoder('utf-8').decode(new Uint8Array(bytes));
      const { load } = await import('js-yaml');
      return load(text) as MetaFile;
    } catch {
      return null;
    }
  },

  writeMeta: async (assetPath: string, meta: MetaFile): Promise<void> => {
    const { dump } = await import('js-yaml');
    return invoke<void>('write_file_text', { path: assetPath + '.meta', contents: dump(meta) });
  },

  // Manifest
  rebuildManifest: (): Promise<void> =>
    invoke<void>('rebuild_manifest'),

  readManifest: async (): Promise<ManifestEntry[]> => {
    try {
      const bytes = await invoke<number[]>('read_file_bytes', {
        path: `${useEditorStore.getState().projectPath || '/home/dumpstertree/Git/Rust/system_test'}/asset.manifest`
      });
      const text = new TextDecoder('utf-8').decode(new Uint8Array(bytes));
      const { load } = await import('js-yaml');
      const parsed = load(text) as any;
      return Array.isArray(parsed?.manifest) ? parsed.manifest : [];
    } catch {
      return [];
    }
  },

  // File picker
  pickFile: async (): Promise<string | null> => {
    if (!isTauri()) return null;
    const { open } = await import('@tauri-apps/plugin-dialog');
    const result = await open({ multiple: false, directory: false });
    return typeof result === 'string' ? result : null;
  },
};