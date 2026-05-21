import { create } from 'zustand';
import type { TabGroupState, ObjectState, PlayMode, TopTab } from './types';
import { api } from './api';

interface EditorStore {
  activeTab:    TopTab;
  setActiveTab: (tab: TopTab) => void;

  mode:  PlayMode;
  play:  () => Promise<void>;
  stop:  () => Promise<void>;
  pause: () => Promise<void>;

  tabGroupState:      TabGroupState | null;
  refreshTabGroup:    () => Promise<void>;

  // selectedInstance is the HashMap key (instance name string)
  selectedInstance:   string;
  selectInstance:     (key: string) => void;

  activeLeftTab:      number;
  setActiveLeftTab:   (idx: number) => void;

  selectedObject:     ObjectState | null;
  expandedNodes:      Set<string>;
  selectObject:       (obj: ObjectState | null) => void;
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
      // if no instance selected yet, default to first key
      const currentKey = get().selectedInstance;
      const keys = Object.keys(tabGroupState.id_for_tabs);
      const validKey = keys.includes(currentKey) ? currentKey : (keys[0] ?? '');
      set({ tabGroupState, selectedInstance: validKey });
    } catch (e) { console.error('[store] refreshTabGroup failed:', e); }
  },

  selectedInstance:  '',
  selectInstance:    (key) => set({ selectedInstance: key, activeLeftTab: 0, selectedObject: null }),

  activeLeftTab:     0,
  setActiveLeftTab:  (idx) => set({ activeLeftTab: idx, selectedObject: null }),

  selectedObject:    null,
  expandedNodes:     new Set<string>(),

  selectObject: (obj) => set({ selectedObject: obj }),

  toggleNode: (path) => {
    const next = new Set(get().expandedNodes);
    if (next.has(path)) next.delete(path); else next.add(path);
    set({ expandedNodes: next });
  },
}));
