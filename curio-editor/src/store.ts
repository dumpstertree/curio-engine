import { create } from 'zustand';
import type { TabGroupState, ObjectState, PlayMode, TopTab } from './types';
import { api } from './api';

// Path uniquely identifies an object: [tabIndex, ...childIndices]
export type ObjectPath = number[];

function resolveObject(tabGroupState: TabGroupState | null, selectedInstance: string, activeLeftTab: number, path: ObjectPath | null): ObjectState | null {
  if (!tabGroupState || !path) return null;
  const tabs = tabGroupState.id_for_tabs[selectedInstance] ?? [];
  const objects = tabs[activeLeftTab]?.objects ?? [];
  let nodes = objects;
  let obj: ObjectState | null = null;
  for (const idx of path) {
    obj = nodes[idx] ?? null;
    if (!obj) return null;
    nodes = obj.children;
  }
  return obj;
}

interface EditorStore {
  activeTab:    TopTab;
  setActiveTab: (tab: TopTab) => void;

  mode:  PlayMode;
  play:  () => Promise<void>;
  stop:  () => Promise<void>;
  pause: () => Promise<void>;

  tabGroupState:      TabGroupState | null;
  refreshTabGroup:    () => Promise<void>;
  startPolling:       () => void;
  stopPolling:        () => void;

  selectedInstance:   string;
  selectInstance:     (key: string) => void;

  activeLeftTab:      number;
  setActiveLeftTab:   (idx: number) => void;

  // Path-based selection — survives tabGroupState refreshes
  selectedObjectPath: ObjectPath | null;
  selectedObject:     ObjectState | null;          // derived, kept in sync
  expandedNodes:      Set<string>;
  selectObjectByPath: (path: ObjectPath | null) => void;
  toggleNode:         (path: string) => void;
}

export const useEditorStore = create<EditorStore>((set, get) => ({
  activeTab:    'play',
  setActiveTab: (tab) => set({ activeTab: tab }),

  mode: 'stopped',

  play: async () => {
    try {
      await api.pressPlay();
      set({ mode: 'playing' });
      get().refreshTabGroup();
    } catch (e) { console.error('[store] play failed:', e); }
  },

  stop: async () => {
    try {
      await api.pressStop();
      set({ mode: 'stopped' });
      get().refreshTabGroup();
    } catch (e) { console.error('[store] stop failed:', e); }
  },

  pause: async () => {
    try {
      await api.pressPause();
      set(s => ({ mode: s.mode === 'paused' ? 'playing' : 'paused' }));
    } catch (e) { console.error('[store] pause failed:', e); }
  },

  tabGroupState: null,

  refreshTabGroup: async () => {
    try {
      const tabGroupState = await api.getTabGroupState();
      const currentKey = get().selectedInstance;
      const keys = Object.keys(tabGroupState.id_for_tabs);
      const validKey = keys.includes(currentKey) ? currentKey : (keys[0] ?? '');
      const { selectedObjectPath, activeLeftTab } = get();
      // Re-derive selectedObject from fresh state
      const selectedObject = resolveObject(tabGroupState, validKey, activeLeftTab, selectedObjectPath);
      set({ tabGroupState, selectedInstance: validKey, selectedObject });
    } catch (e) { console.error('[store] refreshTabGroup failed:', e); }
  },

  _pollHandle: null as ReturnType<typeof setInterval> | null,

  startPolling: () => {
    const existing = (get() as any)._pollHandle;
    if (existing) return;
    const handle = setInterval(() => { get().refreshTabGroup(); }, 500);
    (get() as any)._pollHandle = handle;
  },

  stopPolling: () => {
    const handle = (get() as any)._pollHandle;
    if (handle) { clearInterval(handle); (get() as any)._pollHandle = null; }
  },

  selectedInstance:  '',
  selectInstance:    (key) => set({ selectedInstance: key, activeLeftTab: 0, selectedObjectPath: null, selectedObject: null }),

  activeLeftTab:     0,
  setActiveLeftTab:  (idx) => set({ activeLeftTab: idx, selectedObjectPath: null, selectedObject: null }),

  selectedObjectPath: null,
  selectedObject:     null,
  expandedNodes:      new Set<string>(),

  selectObjectByPath: (path) => {
    const { tabGroupState, selectedInstance, activeLeftTab } = get();
    const selectedObject = resolveObject(tabGroupState, selectedInstance, activeLeftTab, path);
    set({ selectedObjectPath: path, selectedObject });
  },

  toggleNode: (path) => {
    const next = new Set(get().expandedNodes);
    if (next.has(path)) next.delete(path); else next.add(path);
    set({ expandedNodes: next });
  },
}));
