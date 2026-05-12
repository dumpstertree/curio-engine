export interface Component {
  name: string;
  fields: Record<string, unknown>;
}

export interface Entity {
  id: number;
  name: string;
  children: Entity[];
  components: Component[];
}

export interface SceneSnapshot {
  entities: Entity[];
}

export type PlayMode = 'stopped' | 'playing' | 'paused';
