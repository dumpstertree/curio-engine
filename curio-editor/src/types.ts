// ─────────────────────────────────────────────────────────────
// Matches Rust structs exactly
// ─────────────────────────────────────────────────────────────

export interface FieldState {
  field_name: string;
  data:       unknown;
}

export interface ComponentState {
  component_name: string;
  fields:         FieldState[];
}

export interface ObjectState {
  object_name: string;
  children:    ObjectState[];
  components:  ComponentState[];
}

export interface TabState {
  tab_name: string;
  objects:  ObjectState[];
}

// id_for_tabs: HashMap<String, Vec<TabState>>
// serializes as { "Host": [...], "Peer 1": [...] }
export interface TabGroupState {
  id_for_tabs: Record<string, TabState[]>;
}

// ─────────────────────────────────────────────────────────────
// Editor state
// ─────────────────────────────────────────────────────────────

export type PlayMode = 'stopped' | 'playing' | 'paused';
export type TopTab   = 'play' | 'asset' | 'input' | 'prefab';
