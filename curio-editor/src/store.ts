import { create } from 'zustand';
import type {
  FormsSnapshot, LedgerSnapshot, LedgerRecord,
  PlayMode, TopTab, LeftTab,
} from './types';
import { api } from './api';

interface EditorStore {
  // ── Tab navigation ───────────────────────────────────────
  activeTab:    TopTab;
  setActiveTab: (tab: TopTab) => void;
  leftTab:      LeftTab;
  setLeftTab:   (tab: LeftTab) => void;

  // ── Play state ───────────────────────────────────────────
  mode:  PlayMode;
  play:  () => Promise<void>;
  stop:  () => Promise<void>;
  pause: () => Promise<void>;

  // ── Ledger ───────────────────────────────────────────────
  ledger:           LedgerSnapshot | null;
  selectedInstance: number;
  selectInstance:   (id: number) => void;
  refreshLedger:    () => Promise<void>;

  // ── Forms ────────────────────────────────────────────────
  forms:         FormsSnapshot | null;
  expandedForms: Set<number>;
  toggleForm:    (id: number) => void;
  refreshForms:  () => Promise<void>;

  // ── Inspector — shared selection ─────────────────────────
  selectedForm:   number | null;
  selectedRecord: LedgerRecord | null;
  selectForm:     (id: number | null) => void;
  selectRecord:   (record: LedgerRecord | null) => void;
}

export const useEditorStore = create<EditorStore>((set, get) => ({
  // ── Tab navigation ───────────────────────────────────────
  activeTab:    'play',
  setActiveTab: (tab) => set({ activeTab: tab }),
  leftTab:      'ledger',
  setLeftTab:   (tab) => set({ leftTab: tab }),

  // ── Play state ───────────────────────────────────────────
  mode: 'stopped',

  play: async () => {
    try {
      await api.pressPlay();
      set({ mode: 'playing' });
      get().refreshForms();
    } catch (e) { console.error('[store] play failed:', e); }
  },

  stop: async () => {
    try {
      await api.pressStop();
      set({ mode: 'stopped' });
      get().refreshForms();
    } catch (e) { console.error('[store] stop failed:', e); }
  },

  pause: async () => {
    try {
      await api.pressPause();
      set(s => ({ mode: s.mode === 'paused' ? 'playing' : 'paused' }));
    } catch (e) { console.error('[store] pause failed:', e); }
  },

  // ── Ledger ───────────────────────────────────────────────
  ledger:           null,
  selectedInstance: 0,
  selectInstance:   (id) => set({ selectedInstance: id }),

  refreshLedger: async () => {
    try {
      const ledger = await api.getLedgerSnapshot();
      set({ ledger });
    } catch (e) { console.error('[store] refreshLedger failed:', e); }
  },

  // ── Forms ────────────────────────────────────────────────
  forms:         null,
  expandedForms: new Set<number>(),

  toggleForm: (id) => {
    const next = new Set(get().expandedForms);
    if (next.has(id)) next.delete(id); else next.add(id);
    set({ expandedForms: next });
  },

  refreshForms: async () => {
    try {
      const forms = await api.getForms();
      set({ forms });
    } catch (e) { console.error('[store] refreshForms failed:', e); }
  },

  // ── Inspector ────────────────────────────────────────────
  selectedForm:   null,
  selectedRecord: null,

  // selecting a form clears record selection and vice versa
  selectForm:   (id)     => set({ selectedForm: id, selectedRecord: null }),
  selectRecord: (record) => set({ selectedRecord: record, selectedForm: null }),
}));
