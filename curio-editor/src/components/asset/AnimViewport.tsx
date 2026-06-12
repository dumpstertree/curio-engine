import React, { useEffect, useRef, useState } from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import JSZip from 'jszip';
import { spine } from '../../vendor/spine38/spine-threejs.mjs';
import { api } from '../../api';

// ─── Exported types ───────────────────────────────────────────────────────────

export interface AnimInfo {
  animations: string[];
  bones:      number;
  slots:      number;
  sizeKb:     number;
}

export interface AnimPlaybackState {
  current:  string | null;
  time:     number;
  duration: number;
}

interface Props {
  path:       string;
  onInfo:     (info: AnimInfo | null) => void;
  onPlayback: (state: AnimPlaybackState) => void;
  playAnim:   string | null;
}

// ─── Component ────────────────────────────────────────────────────────────────

export function AnimViewport({ path, onInfo, onPlayback, playAnim }: Props) {
  const mountRef = useRef<HTMLDivElement>(null);
  const meshRef   = useRef<any>(null); // spine.threejs.SkeletonMesh
  const [error,   setError]   = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  // Switch animation when inspector requests it
  useEffect(() => {
    if (!meshRef.current || !playAnim) return;
    meshRef.current.state.setAnimation(0, playAnim, true);
  }, [playAnim]);

  useEffect(() => {
    if (!mountRef.current) return;
    setLoading(true);
    setError(null);
    onInfo(null);

    const mount = mountRef.current;
    let alive   = true;
    let frameId = 0;

    async function init() {
      try {
        // 1. Read and unzip .anim
        const bytes  = await api.readFileBytes(path);
        const u8     = new Uint8Array(bytes);
        const sizeKb = Math.round(bytes.length / 1024);

        const zip     = await JSZip.loadAsync(u8);
        const entries = Object.keys(zip.files);
        const find    = (suffix: string) => {
          const k = entries.find(e => e.endsWith(suffix));
          if (!k) throw new Error(`Missing ${suffix} in zip. Found: ${entries.join(', ')}`);
          return zip.files[k];
        };

        const atlasText = await find('skeleton.atlas').async('string');
        const jsonText  = await find('skeleton.json').async('string');
        const pngBytes  = await find('skeleton.png').async('uint8array');

        // 2. Decode PNG
        const pngBlob     = new Blob([pngBytes], { type: 'image/png' });
        const imageBitmap = await createImageBitmap(pngBlob);

        // 3. Build atlas — textureLoader is synchronous, image already decoded
        const atlas = new spine.TextureAtlas(atlasText, (_path: string) => {
          return new spine.threejs.ThreeJsTexture(imageBitmap as any);
        });

        // 4. Build skeleton data (3.8 native format — no shim needed)
        const loader   = new spine.AtlasAttachmentLoader(atlas);
        const skelJson = new spine.SkeletonJson(loader);
        const skelData = skelJson.readSkeletonData(jsonText);

        const animNames = skelData.animations.map((a: any) => a.name);
        onInfo({ animations: animNames, bones: skelData.bones.length, slots: skelData.slots.length, sizeKb });

        // 5. SkeletonMesh — handles geometry batching internally
        const skeletonMesh = new spine.threejs.SkeletonMesh(skelData, (_params: any) => {});
        meshRef.current = skeletonMesh;

        if (animNames.length > 0) {
          skeletonMesh.state.setAnimation(0, animNames[0], true);
        }
        skeletonMesh.skeleton.setToSetupPose();
        skeletonMesh.skeleton.updateWorldTransform();

        // 6. Three.js scene
        const W = mount.clientWidth  || 600;
        const H = mount.clientHeight || 400;

        const scene = new THREE.Scene();
        scene.background = new THREE.Color(0x141414);

        const grid = new THREE.GridHelper(2000, 40, 0x2a2a2a, 0x2a2a2a);
        grid.rotation.x = Math.PI / 2;
        scene.add(grid);
        scene.add(skeletonMesh);

        const camera = new THREE.PerspectiveCamera(45, W / H, 0.1, 100000);
        camera.position.set(0, 0, 1000);

        const renderer = new THREE.WebGLRenderer({ antialias: true });
        renderer.setSize(W, H);
        renderer.setPixelRatio(window.devicePixelRatio);
        mount.appendChild(renderer.domElement);

        const orb = new OrbitControls(camera, renderer.domElement);
        orb.enableDamping      = true;
        orb.dampingFactor      = 0.08;
        orb.screenSpacePanning = true;

        const ro = new ResizeObserver(() => {
          const w = mount.clientWidth, h = mount.clientHeight;
          camera.aspect = w / h;
          camera.updateProjectionMatrix();
          renderer.setSize(w, h);
        });
        ro.observe(mount);

        let fitted   = false;
        let lastTime = performance.now();
        const box    = new THREE.Box3();

        function tick() {
          if (!alive) return;
          frameId = requestAnimationFrame(tick);

          const now   = performance.now();
          const delta = Math.min((now - lastTime) / 1000, 0.064);
          lastTime    = now;

          skeletonMesh.update(delta);

          // Report playback
          const track = skeletonMesh.state.getCurrent(0);
          if (track?.animation) {
            onPlayback({
              current:  track.animation.name,
              time:     track.trackTime % track.animation.duration,
              duration: track.animation.duration,
            });
          }

          // Auto-fit camera once geometry exists
          if (!fitted) {
            box.setFromObject(skeletonMesh);
            if (!box.isEmpty()) {
              fitted = true;
              const center = box.getCenter(new THREE.Vector3());
              const size   = box.getSize(new THREE.Vector3());
              const maxDim = Math.max(size.x, size.y, 1);
              const fovRad = THREE.MathUtils.degToRad(camera.fov);
              const dist   = (maxDim / 2) / Math.tan(fovRad / 2) * 1.4;
              camera.position.set(center.x, center.y, dist);
              camera.near = dist * 0.001;
              camera.far  = dist * 100;
              camera.updateProjectionMatrix();
              orb.target.copy(center);
              orb.update();
            }
          }

          orb.update();
          renderer.render(scene, camera);
        }

        setLoading(false);
        tick();

        return () => {
          alive = false;
          cancelAnimationFrame(frameId);
          orb.dispose();
          renderer.dispose();
          ro.disconnect();
          skeletonMesh.dispose();
          if (mount.contains(renderer.domElement)) mount.removeChild(renderer.domElement);
        };

      } catch (e) {
        if (alive) { setError(String(e)); setLoading(false); }
      }
    }

    let cleanup: (() => void) | undefined;
    init().then(fn => { cleanup = fn; });
    return () => { alive = false; cleanup?.(); };
  }, [path]);

  return (
    <div className="asset-viewport-glb" ref={mountRef} style={{ width: '100%', height: '100%' }}>
      {loading && !error && <div className="asset-viewport-overlay">Loading…</div>}
      {error   && <div className="asset-viewport-overlay asset-error">{error}</div>}
    </div>
  );
}
