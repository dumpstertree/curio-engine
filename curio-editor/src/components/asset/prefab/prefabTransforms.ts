import * as THREE from 'three';
import type { PrefabGameObjectRaw } from './prefabTypes';
import { isTransform, isRenderer, readTransformFields, readRendererAsset, eulerDegToQuat } from './prefabTypes';

export interface RenderEntry {
  /** Path through the hierarchy by child index, e.g. [0, 2] = root.children[0].children[2] */
  path:        number[];
  name:        string;
  worldMatrix: THREE.Matrix4;
  rendererType: 'RendererStatic' | 'RendererDynamic';
  assetPath:   string; // relative to assets/ root
}

/** Walks the prefab hierarchy, composing local transforms (transform2d/transform3d)
 *  into world matrices, and collects every RendererStatic/RendererDynamic with a
 *  non-empty asset path along with its world matrix. */
export function collectRenderEntries(root: PrefabGameObjectRaw): RenderEntry[] {
  const out: RenderEntry[] = [];

  function walk(node: PrefabGameObjectRaw, parentMatrix: THREE.Matrix4, path: number[]) {
    // Find this node's transform component (transform2d or transform3d), if any
    let localMatrix = new THREE.Matrix4(); // identity
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
          const assetPath = readRendererAsset(comp);
          if (assetPath && assetPath.trim() !== '') {
            out.push({
              path,
              name: node.name,
              worldMatrix: worldMatrix.clone(),
              rendererType: comp.type as 'RendererStatic' | 'RendererDynamic',
              assetPath: assetPath.trim(),
            });
          }
        }
      }

      node.children.forEach((child, i) => walk(child, worldMatrix, [...path, i]));
    }
  }

  walk(root, new THREE.Matrix4(), []);
  return out;
}
