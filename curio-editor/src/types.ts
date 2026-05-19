export interface Component {
  name:   string;
  fields: Record<string, unknown>;
}

export interface Form {
  id:         number;
  name:       string;
  children:   Form[];
  components: Component[];
}

export interface FormsSnapshot {
  forms: Form[];
}

export type RecordPermission = 'read' | 'write' | 'readwrite';

export interface LedgerRecord {
  name:        string;
  record_type: string;
  permissions: RecordPermission;
  value:       Record<string, unknown>;
}

export interface GameInstance {
  id:      number;
  name:    string;
  role:    'host' | 'peer';
  records: LedgerRecord[];
}

export interface LedgerSnapshot {
  instances: GameInstance[];
}

export type PlayMode = 'stopped' | 'playing' | 'paused';
export type TopTab   = 'play' | 'asset' | 'input' | 'prefab'; // scene removed — play IS the scene
export type LeftTab  = 'ledger' | 'forms';
