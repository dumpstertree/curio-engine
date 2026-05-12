import { create } from 'zustand';
import type { SceneSnapshot, PlayMode } from './types';
import { api } from './api';

interface EditorStore {
  // scene
  snapshot: SceneSnapshot | null;
  selected: number | null;
  expanded: Set<number>;

  // play state
  mode: PlayMode;

  // actions
  refreshSnapshot: () => Promise<void>;
  selectEntity: (id: number | null) => void;
  toggleExpand: (id: number) => void;
  play: () => Promise<void>;
  stop: () => Promise<void>;
  pause: () => Promise<void>;
}

export const useEditorStore = create<EditorStore>((set, get) => ({
  snapshot: null,
  selected: null,
  expanded: new Set<number>(),
  mode: 'stopped',

  refreshSnapshot: async () => {
    try {
      const snapshot = await api.getSceneSnapshot();
      set({ snapshot });
    } catch (e) {
      console.error('get_scene_snapshot failed:', e);
    }
  },

  selectEntity: (id) => set({ selected: id }),

  toggleExpand: (id) => {
    const next = new Set(get().expanded);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    set({ expanded: next });
  },

  play: async () => {
    await api.pressPlay();
    set({ mode: 'playing' });
    get().refreshSnapshot();
  },

  stop: async () => {
    await api.pressStop();
    set({ mode: 'stopped' });
    get().refreshSnapshot();
  },

  pause: async () => {
    await api.pressPause();
    set({ mode: 'paused' });
  },


}));

