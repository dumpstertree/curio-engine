import * as THREE from 'three';
import type { PrefabGameObjectRaw } from './prefabTypes';
import { isTransform, isRenderer, readTransformFields, readRendererAsset, eulerDegToQuat } from './prefabTypes';
import { api } from '../../../api';

import { getProjectRoot } from '../../../paths';
const PROJECT_ROOT = getProjectRoot();

export interface RenderEntry {
  path:         number[];
  name:         string;
  worldMatrix:  THREE.Matrix4;
  rendererType: 'RendererStatic' | 'RendererDynamic';
  /** Absolute filesystem path to the asset file */
  assetAbsPath: string;
}

/** Walks the hierarchy collecting renderer entries, resolving asset IDs to
 *  absolute paths via the manifest. */
export async function collectRenderEntries(root: PrefabGameObjectRaw): Promise<RenderEntry[]> {
  // Load manifest once for all ID lookups
  const manifest = await api.readManifest();
  const idToUri  = new Map(manifest.map(e => [String(e.id), e.uri]));

  const out: RenderEntry[] = [];

  function walk(node: PrefabGameObjectRaw, parentMatrix: THREE.Matrix4, path: number[]) {
    let localMatrix = new THREE.Matrix4();
    const transformComp = node.components.find(c => isTransform(c.type));
    if (transformComp) {
      const t = readTransformFields(transformComp);
      const q = eulerDegToQuat(t.rotation);
      localMatrix = new THREE.Matrix4().compose(
        new THREE.Vector3(t.position.x, t.position.y, t.position.z),
        new THREE.Quaternion(q.x, q.y, q.z, q.w),
        new THREE.Vector3(t.scale.x, t.scale.y, t.scale.z),
      );
    }

    const worldMatrix = new THREE.Matrix4().multiplyMatrices(parentMatrix, localMatrix);

    if (node.enabled) {
      for (const comp of node.components) {
        if (isRenderer(comp.type)) {
          const rawVal = readRendererAsset(comp);
          if (!rawVal || !rawVal.trim()) continue;

          const trimmed = rawVal.trim();
          // Resolve: numeric ID → URI → abs path; otherwise treat as legacy relative path
          let absPath: string;
          if (/^\d+$/.test(trimmed)) {
            const uri = idToUri.get(trimmed);
            if (!uri) continue; // unknown ID, skip
            absPath = `${PROJECT_ROOT}/${uri}`;
          } else {
            // Legacy path format — relative to assets/
            absPath = `${PROJECT_ROOT}/assets/${trimmed}`;
          }

          out.push({
            path,
            name:         node.name,
            worldMatrix:  worldMatrix.clone(),
            rendererType: comp.type as 'RendererStatic' | 'RendererDynamic',
            assetAbsPath: absPath,
          });
        }
      }
      node.children.forEach((child, i) => walk(child, worldMatrix, [...path, i]));
    }
  }

  walk(root, new THREE.Matrix4(), []);
  return out;
}
