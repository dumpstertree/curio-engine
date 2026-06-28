import { create } from 'zustand';
import type { TabGroupState, ObjectState, PlayMode, TopTab } from './types';
import { api } from './api';

export type ObjectPath = number[];

export type CompileStatus = 'idle' | 'compiling' | 'success' | 'error';

export interface LogLine {
  level:   'info' | 'warn' | 'error';
  message: string;
  time:    string;
}

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

function parseLogLevel(line: string): LogLine['level'] {
  const l = line.toLowerCase();
  if (l.includes('[error]') || l.includes('error:') || l.includes('panicked')) return 'error';
  if (l.includes('[warn]')  || l.includes('warning:')) return 'warn';
  return 'info';
}

function timestamp(): string {
  return new Date().toLocaleTimeString('en-US', { hour12: false });
}

interface EditorStore {
  activeTab:    TopTab;
  setActiveTab: (tab: TopTab) => void;

  mode:          PlayMode;
  compileStatus: CompileStatus;
  compileError:  string;
  play:          () => Promise<void>;
  stop:          () => Promise<void>;
  pause:         () => Promise<void>;

  logs:          LogLine[];
  unreadLogs:    number;
  consoleOpen:   boolean;
  addLog:        (line: string) => void;
  clearLogs:     () => void;
  toggleConsole: () => void;
  markLogsRead:  () => void;
  startLogPolling: () => void;
  stopLogPolling:  () => void;

  projectPath:    string;
  loadProjectPath: () => Promise<void>;

  tabGroupState:      TabGroupState | null;
  refreshTabGroup:    () => Promise<void>;
  startPolling:       () => void;
  stopPolling:        () => void;

  selectedInstance:   string;
  selectInstance:     (key: string) => void;

  activeLeftTab:      number;
  setActiveLeftTab:   (idx: number) => void;

  selectedObjectPath: ObjectPath | null;
  selectedObject:     ObjectState | null;
  expandedNodes:      Set<string>;
  selectObjectByPath: (path: ObjectPath | null) => void;
  toggleNode:         (path: string) => void;
}

export const useEditorStore = create<EditorStore>((set, get) => ({
  activeTab:    'play',
  setActiveTab: (tab) => set({ activeTab: tab }),

  mode:          'stopped',
  compileStatus: 'idle',
  compileError:  '',

  play: async () => {
    set({ compileStatus: 'compiling', compileError: '' });
    get().clearLogs();
    get().startLogPolling();
    try {
      await api.compile();
      // Poll compile status until done
      const pollCompile = setInterval(async () => {
        const status = await api.getCompileStatus();
        if (status === 'success') {
          clearInterval(pollCompile);
          try {
            await api.pressPlayStart();
            set({ mode: 'playing', compileStatus: 'success' });
            get().refreshTabGroup();
          } catch (e: any) {
            set({ compileStatus: 'error', compileError: String(e) });
            get().stopLogPolling();
          }
        } else if (status === 'error') {
          clearInterval(pollCompile);
          set({ compileStatus: 'error', compileError: 'Build failed — see console for details' });
          // keep log polling running so user can read errors
        }
      }, 500);
    } catch (e: any) {
      set({ compileStatus: 'error', compileError: String(e) });
      get().stopLogPolling();
    }
  },

  stop: async () => {
    try {
      // Cancel compile if in progress
      const status = await api.getCompileStatus();
      if (status === 'compiling') await api.cancelCompile();
      await api.pressStop();
      get().stopLogPolling();
      set({ mode: 'stopped', compileStatus: 'idle', compileError: '' });
      get().clearLogs();
      get().refreshTabGroup();
    } catch (e) { console.error('[store] stop failed:', e); }
  },

  pause: async () => {
    try {
      await api.pressPause();
      set(s => ({ mode: s.mode === 'paused' ? 'playing' : 'paused' }));
    } catch (e) { console.error('[store] pause failed:', e); }
  },

  logs:       [],
  unreadLogs: 0,
  consoleOpen: false,

  addLog: (line: string) => {
    const entry: LogLine = {
      level:   parseLogLevel(line),
      message: line,
      time:    timestamp(),
    };
    set(s => ({
      logs:       [...s.logs.slice(-500), entry], // keep last 500
      unreadLogs: s.consoleOpen ? 0 : s.unreadLogs + 1,
    }));
  },

  clearLogs: () => set({ logs: [], unreadLogs: 0 }),

  toggleConsole: () => set(s => ({
    consoleOpen: !s.consoleOpen,
    unreadLogs:  !s.consoleOpen ? 0 : s.unreadLogs, // clear badge when opening
  })),

  markLogsRead: () => set({ unreadLogs: 0 }),

  startLogPolling: () => {
    if ((get() as any)._logPollHandle) return;
    const handle = setInterval(async () => {
      try {
        const entries = await api.getLogs();
        if (entries.length === 0) return;
        const { consoleOpen } = get();
        const newLines: LogLine[] = entries.map(([level, message]) => ({
          level: level.includes('ERROR') ? 'error' : level.includes('WARN') ? 'warn' : 'info',
          message: `${level}: ${message}`,
          time: new Date().toLocaleTimeString('en-US', { hour12: false }),
        }));
        set(s => ({
          logs:       [...s.logs.slice(-500), ...newLines],
          unreadLogs: consoleOpen ? 0 : s.unreadLogs + newLines.length,
        }));
      } catch {}
    }, 250);
    (get() as any)._logPollHandle = handle;
  },

  stopLogPolling: () => {
    const handle = (get() as any)._logPollHandle;
    if (handle) { clearInterval(handle); (get() as any)._logPollHandle = null; }
  },

  projectPath: '',
  loadProjectPath: async () => {
    try {
      const path = await api.getProjectPath();
      set({ projectPath: path });
    } catch (e) { console.error('[store] loadProjectPath failed:', e); }
  },

  tabGroupState: null,

  refreshTabGroup: async () => {
    try {
      const tabGroupState = await api.getTabGroupState();
      const currentKey = get().selectedInstance;
      const keys = Object.keys(tabGroupState.id_for_tabs);
      const validKey = keys.includes(currentKey) ? currentKey : (keys[0] ?? '');
      const { selectedObjectPath, activeLeftTab } = get();
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
