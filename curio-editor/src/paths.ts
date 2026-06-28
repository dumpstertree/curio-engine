import { useEditorStore } from './store';

export function getProjectRoot(): string {
  const path = useEditorStore.getState().projectPath;
  return path || '/home/dumpstertree/Git/Rust/system_test';
}

export function getAssetsRoot(): string {
  return `${getProjectRoot()}/assets`;
}
