import React, { useEffect, useRef, useState } from 'react';
import * as THREE from 'three';
import { OrbitControls }    from 'three/examples/jsm/controls/OrbitControls.js';
import { TransformControls } from 'three/examples/jsm/controls/TransformControls.js';
import type { ResolvedGameObject } from './prefabResolver';
import { resolvedToRawFull } from './prefabResolver';
import { collectRenderEntries } from './prefabTransforms';
import { loadGlbObject, loadAnimMesh } from './assetLoaders';
import type { PrefabGameObjectRaw } from './prefabTypes';
import {
  formatTuple,
  getNodeAtPath,
  setNodeAtPath,
  setComponentField,
  isTransform,
} from './prefabTypes';
const PROJECT_ROOT = '/home/dumpstertree/Git/Rust/system_test';

// ── Structure key: only asset paths + hierarchy, NOT transform values ─────────
// This prevents camera resets when you move/rotate/scale an object.
function structureKey(root: ResolvedGameObject): string {
  function walk(node: any): any {
    return {
      name:     node.name,
      enabled:  node.enabled,
      base:     node.base,
      // only the renderer asset paths, not transform fields
      assets: (node.components ?? [])
        .filter((c: any) => c.type === 'RendererStatic' || c.type === 'RendererDynamic')
        .map((c: any) => ({ type: c.type, fields: c.fields })),
      children: (node.children ?? []).map(walk),
    };
  }
  return JSON.stringify(walk(root));
}

// ── Quaternion → Euler degrees (XYZ intrinsic, matching eulerDegToQuat) ──────
function quatToEulerDeg(q: THREE.Quaternion): { x: number; y: number; z: number } {
  const euler = new THREE.Euler().setFromQuaternion(q, 'XYZ');
  const r2d   = 180 / Math.PI;
  return { x: euler.x * r2d, y: euler.y * r2d, z: euler.z * r2d };
}

interface Props {
  root:        ResolvedGameObject;
  raw:         PrefabGameObjectRaw;
  selectedPath: number[] | null;
  onSelect:    (path: number[] | null) => void;
  onRawChange: (next: PrefabGameObjectRaw) => void;
}

export function PrefabViewport({ root, raw, selectedPath, onSelect, onRawChange }: Props) {
  const mountRef = useRef<HTMLDivElement>(null);
  const [error,   setError]   = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [mode,    setMode]    = useState<'translate' | 'rotate' | 'scale'>('translate');

  const modeRef         = useRef<'translate' | 'rotate' | 'scale'>('translate');
  modeRef.current       = mode;

  const selectedPathRef = useRef<number[] | null>(null);
  selectedPathRef.current = selectedPath;

  // Keep raw in a ref so mouseUp always reads current value without re-running the effect
  const rawRef = useRef<PrefabGameObjectRaw>(raw);
  rawRef.current = raw;

  const onRawChangeRef = useRef(onRawChange);
  onRawChangeRef.current = onRawChange;

  const selectObjectRef      = useRef<((path: number[] | null) => void) | null>(null);
  // Called whenever raw changes non-structurally — updates Three.js object transforms in place
  const applyTransformsRef   = useRef<((raw: PrefabGameObjectRaw) => Promise<void>) | null>(null);

  // Only rebuild the scene when asset paths or hierarchy change — NOT on transform edits
  const sceneKey = structureKey(root);

  useEffect(() => {
    if (!mountRef.current) return;
    setLoading(true);
    setError(null);

    const mount = mountRef.current;
    let alive   = true;
    let frameId = 0;
    const animMeshes: any[] = [];

    const objPathMap = new Map<string, number[]>();
    const pathObjMap = new Map<string, THREE.Object3D>();

    async function init() {
      try {
        const fullRaw = resolvedToRawFull(root);
        const entries = await collectRenderEntries(fullRaw);

        const W = mount.clientWidth  || 600;
        const H = mount.clientHeight || 400;

        const scene = new THREE.Scene();
        scene.background = new THREE.Color(0x141414);

        const grid = new THREE.GridHelper(20, 20, 0x2a2a2a, 0x2a2a2a);
        scene.add(grid);

        const ambient  = new THREE.AmbientLight(0xffffff, 0.6);
        const dirLight = new THREE.DirectionalLight(0xffffff, 1.2);
        dirLight.position.set(5, 8, 5);
        const fillLight = new THREE.DirectionalLight(0x8888ff, 0.3);
        fillLight.position.set(-5, 2, -5);
        scene.add(ambient, dirLight, fillLight);

        const camera = new THREE.PerspectiveCamera(45, W / H, 0.01, 10000);
        camera.position.set(2, 2, 4);

        const renderer = new THREE.WebGLRenderer({ antialias: true });
        renderer.setSize(W, H);
        renderer.setPixelRatio(window.devicePixelRatio);
        renderer.outputEncoding = (THREE as any).sRGBEncoding;
        mount.appendChild(renderer.domElement);

        const orb = new OrbitControls(camera, renderer.domElement);
        orb.enableDamping = true;
        orb.dampingFactor = 0.08;

        const tc = new TransformControls(camera, renderer.domElement);
        tc.setMode('translate');
        scene.add(tc);

        tc.addEventListener('dragging-changed', (e: any) => {
          orb.enabled = !e.value;
        });

        // ── mouseUp: write position / rotation / scale based on current gizmo mode ──
        tc.addEventListener('mouseUp', () => {
          const attached = tc.object;
          if (!attached) return;
          const path = objPathMap.get(attached.uuid);
          if (!path) return;

          // Read current raw from ref so we always have latest value
          const currentRaw  = rawRef.current;
          let node = getNodeAtPath(currentRaw, path);
          if (!node) return;

          let transformComp = node.components.find(c => isTransform(c.type));
          let updatedComp   = transformComp ?? { type: 'Transform3D', fields: [] };

          const m = modeRef.current;

          if (m === 'translate') {
            const p   = attached.position;
            updatedComp = setComponentField(updatedComp, 'position', formatTuple([p.x, p.y, p.z]));
          } else if (m === 'rotate') {
            const e   = quatToEulerDeg(attached.quaternion);
            updatedComp = setComponentField(updatedComp, 'rotation', formatTuple([e.x, e.y, e.z]));
          } else if (m === 'scale') {
            const s   = attached.scale;
            updatedComp = setComponentField(updatedComp, 'scale', formatTuple([s.x, s.y, s.z]));
          }

          const newNode: PrefabGameObjectRaw = {
            ...node,
            components: transformComp
              ? node.components.map(c => c === transformComp ? updatedComp : c)
              : [...node.components, updatedComp],
          };

          onRawChangeRef.current(setNodeAtPath(currentRaw, path, newNode));
        });

        function selectObject(path: number[] | null) {
          if (!path) { tc.detach(); onSelect(null); return; }
          const key = path.join(',');
          const obj = pathObjMap.get(key);
          if (obj) { tc.attach(obj); tc.setMode(modeRef.current); }
          onSelect(path);
        }
        selectObjectRef.current = selectObject;

        const group = new THREE.Group();
        group.name  = '__prefab_group__';
        scene.add(group);

        await Promise.all(entries.map(async entry => {
          try {
            let obj: THREE.Object3D;
            if (entry.rendererType === 'RendererStatic') {
              obj = await loadGlbObject(entry.assetAbsPath);
            } else {
              const sm = await loadAnimMesh(entry.assetAbsPath);
              animMeshes.push(sm);
              obj = sm;
            }
            obj.matrix.copy(entry.worldMatrix);
            obj.matrix.decompose(obj.position, obj.quaternion, obj.scale);
            obj.name = entry.name;

            const pathKey = entry.path.join(',');
            pathObjMap.set(pathKey, obj);
            obj.traverse(child => { objPathMap.set(child.uuid, entry.path); });
            objPathMap.set(obj.uuid, entry.path);

            if (alive) group.add(obj);
          } catch (e) {
            console.error(`[PrefabViewport] failed to load "${entry.name}":`, e);
          }
        }));

        if (!alive) return;

        // Apply live transform updates from inspector edits without rebuilding the scene
        applyTransformsRef.current = async (updatedRaw: PrefabGameObjectRaw) => {
          const resolvedFull = resolvedToRawFull(root);
          function mergeTransforms(resolved: any, updated: any): any {
            return {
              ...resolved,
              components: resolved.components.map((rc: any) => {
                if (!isTransform(rc.type)) return rc;
                const uc = (updated.components ?? []).find((c: any) => c.type === rc.type);
                if (!uc) return rc;
                const fieldMap = new Map(rc.fields.map((f: string) => [f.split(':')[0].trim(), f]));
                for (const f of uc.fields) fieldMap.set(f.split(':')[0].trim(), f);
                return { ...rc, fields: [...fieldMap.values()] };
              }),
              children: resolved.children.map((rc: any, i: number) =>
                mergeTransforms(rc, updated.children?.[i] ?? rc)
              ),
            };
          }
          const merged   = mergeTransforms(resolvedFull, updatedRaw);
          const entries2 = await collectRenderEntries(merged);
          for (const entry of entries2) {
            const key = entry.path.join(',');
            const obj = pathObjMap.get(key);
            if (!obj) continue;
            const pos = new THREE.Vector3();
            const rot = new THREE.Quaternion();
            const scl = new THREE.Vector3();
            entry.worldMatrix.decompose(pos, rot, scl);
            obj.position.copy(pos);
            obj.quaternion.copy(rot);
            obj.scale.copy(scl);
          }
        };

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

        if (selectedPathRef.current) selectObject(selectedPathRef.current);

        const raycaster = new THREE.Raycaster();
        const mouse     = new THREE.Vector2();

        function onPointerDown(e: PointerEvent) {
          (onPointerDown as any)._startX = e.clientX;
          (onPointerDown as any)._startY = e.clientY;
        }

        function onPointerUp(e: PointerEvent) {
          const dx = e.clientX - ((onPointerDown as any)._startX ?? e.clientX);
          const dy = e.clientY - ((onPointerDown as any)._startY ?? e.clientY);
          if (Math.sqrt(dx * dx + dy * dy) > 4) return;
          if (tc.dragging) return;

          const rect = renderer.domElement.getBoundingClientRect();
          mouse.x =  ((e.clientX - rect.left) / rect.width)  * 2 - 1;
          mouse.y = -((e.clientY - rect.top)  / rect.height) * 2 + 1;

          raycaster.setFromCamera(mouse, camera);
          const targets: THREE.Object3D[] = [];
          group.traverse(obj => { if ((obj as THREE.Mesh).isMesh) targets.push(obj); });

          const hits = raycaster.intersectObjects(targets, false);
          if (hits.length === 0) { selectObject(null); return; }

          let found: number[] | null = null;
          let cur: THREE.Object3D | null = hits[0].object;
          while (cur && !found) {
            found = objPathMap.get(cur.uuid) ?? null;
            cur   = cur.parent;
          }
          selectObject(found);
        }

        renderer.domElement.addEventListener('pointerdown', onPointerDown);
        renderer.domElement.addEventListener('pointerup',   onPointerUp);

        function onKey(e: KeyboardEvent) {
          if (e.code === 'Space') {
            // Space only when canvas/body focused
            if (e.target !== document.body && e.target !== renderer.domElement) return;
            e.preventDefault();
            tc.setSpace(tc.space === 'world' ? 'local' : 'world');
          }
          if (e.code === 'KeyW' || e.code === 'KeyE' || e.code === 'KeyR') {
            // Blur any focused input so the mode switch registers
            (document.activeElement as HTMLElement)?.blur?.();
            if (e.code === 'KeyW') { tc.setMode('translate'); setMode('translate'); }
            if (e.code === 'KeyE') { tc.setMode('rotate');    setMode('rotate'); }
            if (e.code === 'KeyR') { tc.setMode('scale');     setMode('scale'); }
          }
        }
        window.addEventListener('keydown', onKey);

        const ro = new ResizeObserver(() => {
          const w = mount.clientWidth, h = mount.clientHeight;
          camera.aspect = w / h;
          camera.updateProjectionMatrix();
          renderer.setSize(w, h);
        });
        ro.observe(mount);

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
          tc.dispose();
          ro.disconnect();
          renderer.dispose();
          window.removeEventListener('keydown', onKey);
          renderer.domElement.removeEventListener('pointerdown', onPointerDown);
          renderer.domElement.removeEventListener('pointerup',   onPointerUp);
          for (const m of animMeshes) m.dispose?.();
          if (mount.contains(renderer.domElement)) mount.removeChild(renderer.domElement);
          selectObjectRef.current      = null;
          applyTransformsRef.current   = null;
        };

      } catch (e) {
        if (alive) { setError(String(e)); setLoading(false); }
      }
    }

    let cleanup: (() => void) | undefined;
    init().then(fn => { cleanup = fn; });
    return () => { alive = false; cleanup?.(); };
  }, [sceneKey]); // Only rebuild when scene structure changes, not on transform edits

  useEffect(() => {
    selectObjectRef.current?.(selectedPath);
  }, [selectedPath]);

  // When raw changes non-structurally (transform edits), update Three.js objects in place
  useEffect(() => {
    applyTransformsRef.current?.(raw);
  }, [raw]);

  return (
    <div className="asset-viewport-glb" ref={mountRef}
      style={{ width: '100%', height: '100%', position: 'relative' }}>
      {loading && !error && <div className="asset-viewport-overlay">Loading…</div>}
      {error   && <div className="asset-viewport-overlay asset-error">{error}</div>}
      {!loading && !error && (
        <div className="prefab-gizmo-hud">
          <span className={mode === 'translate' ? 'active' : ''}>W Move</span>
          <span className={mode === 'rotate'    ? 'active' : ''}>E Rotate</span>
          <span className={mode === 'scale'     ? 'active' : ''}>R Scale</span>
          <span className="prefab-gizmo-sep">·</span>
          <span>Space = toggle space</span>
        </div>
      )}
    </div>
  );
}
