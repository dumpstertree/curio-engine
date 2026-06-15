import React, { useEffect, useRef, useState } from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import type { PrefabGameObjectRaw } from './prefabTypes';
import { collectRenderEntries } from './prefabTransforms';
import { loadGlbObject, loadAnimMesh } from './assetLoaders';

const ASSET_ROOT = '/home/dumpstertree/Git/Rust/system_test/assets';

interface Props {
  /** The current (possibly edited) prefab tree. Re-render triggers a reload of changed assets. */
  root: PrefabGameObjectRaw;
}

export function PrefabViewport({ root }: Props) {
  const mountRef = useRef<HTMLDivElement>(null);
  const [error,   setError]   = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  // Re-create the whole scene whenever the prefab structure changes.
  // (Edits are infrequent and prefabs are small, so a full rebuild is simplest & most correct.)
  const structureKey = JSON.stringify(root);

  useEffect(() => {
    if (!mountRef.current) return;
    setLoading(true);
    setError(null);

    const mount = mountRef.current;
    let alive   = true;
    let frameId = 0;
    const animMeshes: any[] = []; // spine SkeletonMesh instances needing per-frame update

    async function init() {
      try {
        const entries = collectRenderEntries(root);

        const W = mount.clientWidth  || 600;
        const H = mount.clientHeight || 400;

        const scene = new THREE.Scene();
        scene.background = new THREE.Color(0x141414);

        const grid = new THREE.GridHelper(20, 20, 0x2a2a2a, 0x2a2a2a);
        scene.add(grid);

        const ambient = new THREE.AmbientLight(0xffffff, 0.6);
        scene.add(ambient);
        const dirLight = new THREE.DirectionalLight(0xffffff, 1.2);
        dirLight.position.set(5, 8, 5);
        scene.add(dirLight);
        const fillLight = new THREE.DirectionalLight(0x8888ff, 0.3);
        fillLight.position.set(-5, 2, -5);
        scene.add(fillLight);

        const camera = new THREE.PerspectiveCamera(45, W / H, 0.01, 10000);
        camera.position.set(2, 2, 4);

        const renderer = new THREE.WebGLRenderer({ antialias: true });
        renderer.setSize(W, H);
        renderer.setPixelRatio(window.devicePixelRatio);
        renderer.outputEncoding = THREE.sRGBEncoding;
        mount.appendChild(renderer.domElement);

        const orb = new OrbitControls(camera, renderer.domElement);
        orb.enableDamping = true;
        orb.dampingFactor = 0.08;

        // Load every renderer entry, placing it per its world matrix
        const group = new THREE.Group();
        scene.add(group);

        await Promise.all(entries.map(async entry => {
          try {
            let obj: THREE.Object3D;
            if (entry.rendererType === 'RendererStatic') {
              const fullPath = `${ASSET_ROOT}/${entry.assetPath}`;
              obj = await loadGlbObject(fullPath);
            } else {
              const fullPath = `${ASSET_ROOT}/${entry.assetPath}`;
              const skeletonMesh = await loadAnimMesh(fullPath);
              animMeshes.push(skeletonMesh);
              obj = skeletonMesh;
            }
            obj.matrixAutoUpdate = false;
            obj.matrix.copy(entry.worldMatrix);
            obj.matrix.decompose(obj.position, obj.quaternion, obj.scale);
            obj.matrixAutoUpdate = true;
            obj.name = entry.name;
            if (alive) group.add(obj);
          } catch (e) {
            console.error(`[PrefabViewport] failed to load asset for "${entry.name}" (${entry.assetPath}):`, e);
          }
        }));

        if (!alive) return;

        const ro = new ResizeObserver(() => {
          const w = mount.clientWidth, h = mount.clientHeight;
          camera.aspect = w / h;
          camera.updateProjectionMatrix();
          renderer.setSize(w, h);
        });
        ro.observe(mount);

        // Fit camera to the whole group, or show a default framing if empty
        const box = new THREE.Box3().setFromObject(group);
        if (!box.isEmpty()) {
          const center = box.getCenter(new THREE.Vector3());
          const size   = box.getSize(new THREE.Vector3());
          const maxDim = Math.max(size.x, size.y, size.z, 0.01);
          const fovRad = THREE.MathUtils.degToRad(camera.fov);
          const dist   = (maxDim / 2) / Math.tan(fovRad / 2) * 1.6;
          camera.position.set(center.x + dist * 0.5, center.y + dist * 0.4, center.z + dist);
          camera.near = Math.max(dist * 0.001, 0.001);
          camera.far  = dist * 1000;
          camera.updateProjectionMatrix();
          orb.target.copy(center);
          orb.update();
        }

        let lastTime = performance.now();
        function tick() {
          if (!alive) return;
          frameId = requestAnimationFrame(tick);

          const now   = performance.now();
          const delta = Math.min((now - lastTime) / 1000, 0.064);
          lastTime    = now;

          for (const m of animMeshes) m.update(delta);

          orb.update();
          renderer.render(scene, camera);
        }

        setLoading(false);
        tick();

        return () => {
          alive = false;
          cancelAnimationFrame(frameId);
          orb.dispose();
          ro.disconnect();
          renderer.dispose();
          for (const m of animMeshes) m.dispose?.();
          if (mount.contains(renderer.domElement)) mount.removeChild(renderer.domElement);
        };
      } catch (e) {
        if (alive) { setError(String(e)); setLoading(false); }
      }
    }

    let cleanup: (() => void) | undefined;
    init().then(fn => { cleanup = fn; });
    return () => { alive = false; cleanup?.(); };
  }, [structureKey]);

  return (
    <div className="asset-viewport-glb" ref={mountRef} style={{ width: '100%', height: '100%' }}>
      {loading && !error && <div className="asset-viewport-overlay">Loading…</div>}
      {error   && <div className="asset-viewport-overlay asset-error">{error}</div>}
    </div>
  );
}
