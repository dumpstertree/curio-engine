import React, { useEffect, useRef, useState } from 'react';
import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { api } from '../../api';

export interface GlbInfo {
  triangles:  number;
  vertices:   number;
  materials:  number;
  meshes:     number;
  nodes:      number;
  sizeKb:     number;
}

interface Props {
  path:   string;
  onInfo: (info: GlbInfo | null) => void;
}

export function GlbViewport({ path, onInfo }: Props) {
  const mountRef   = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const frameRef    = useRef<number>(0);
  const [error,   setError]   = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!mountRef.current) return;

    setLoading(true);
    setError(null);
    onInfo(null);

    const mount = mountRef.current;
    const W = mount.clientWidth  || 600;
    const H = mount.clientHeight || 400;

    // Scene
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x141414);

    // Subtle grid floor
    const grid = new THREE.GridHelper(10, 20, 0x2a2a2a, 0x2a2a2a);
    scene.add(grid);

    // Lights
    const ambient = new THREE.AmbientLight(0xffffff, 0.6);
    scene.add(ambient);
    const dirLight = new THREE.DirectionalLight(0xffffff, 1.2);
    dirLight.position.set(5, 8, 5);
    scene.add(dirLight);
    const fillLight = new THREE.DirectionalLight(0x8888ff, 0.3);
    fillLight.position.set(-5, 2, -5);
    scene.add(fillLight);

    // Camera
    const camera = new THREE.PerspectiveCamera(45, W / H, 0.01, 1000);
    camera.position.set(0, 1, 3);

    // Renderer
    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(W, H);
    renderer.setPixelRatio(window.devicePixelRatio);
    renderer.outputEncoding = THREE.sRGBEncoding;
    mount.appendChild(renderer.domElement);
    rendererRef.current = renderer;

    // Orbit controls
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.08;
    controls.minDistance = 0.1;
    controls.maxDistance = 100;

    // Animate
    let alive = true;
    function animate() {
      if (!alive) return;
      frameRef.current = requestAnimationFrame(animate);
      controls.update();
      renderer.render(scene, camera);
    }

    // Resize observer
    const ro = new ResizeObserver(() => {
      const w = mount.clientWidth;
      const h = mount.clientHeight;
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
      renderer.setSize(w, h);
    });
    ro.observe(mount);

    // Load GLB
    api.readFileBytes(path).then(bytes => {
      const u8     = new Uint8Array(bytes);
      const buffer = u8.buffer.slice(u8.byteOffset, u8.byteOffset + u8.byteLength);
      const sizeKb = Math.round(bytes.length / 1024);

      const loader = new GLTFLoader();
      loader.parse(buffer, '', gltf => {
        const model = gltf.scene;

        // Collect stats
        let triangles = 0;
        let vertices  = 0;
        const materialSet = new Set<THREE.Material>();
        let meshCount = 0;
        let nodeCount = 0;

        model.traverse(obj => {
          nodeCount++;
          if ((obj as THREE.Mesh).isMesh) {
            meshCount++;
            const mesh = obj as THREE.Mesh;
            const geo  = mesh.geometry;
            if (geo.index) {
              triangles += geo.index.count / 3;
            } else if (geo.attributes.position) {
              triangles += geo.attributes.position.count / 3;
            }
            if (geo.attributes.position) {
              vertices += geo.attributes.position.count;
            }
            const mats = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
            mats.forEach(m => materialSet.add(m));
          }
        });

        onInfo({
          triangles: Math.round(triangles),
          vertices,
          materials: materialSet.size,
          meshes:    meshCount,
          nodes:     nodeCount,
          sizeKb,
        });

        // Fit camera to model
        const box = new THREE.Box3().setFromObject(model);
        const center = box.getCenter(new THREE.Vector3());
        const size   = box.getSize(new THREE.Vector3());
        const maxDim = Math.max(size.x, size.y, size.z);

        model.position.sub(center);
        scene.add(model);

        const dist = maxDim * 1.8;
        camera.position.set(dist * 0.6, dist * 0.5, dist);
        camera.near = maxDim * 0.001;
        camera.far  = maxDim * 100;
        camera.updateProjectionMatrix();
        controls.target.set(0, 0, 0);
        controls.update();

        setLoading(false);
        animate();
      },
      err => {
        setError(String(err));
        setLoading(false);
      });
    }).catch(e => {
      setError(String(e));
      setLoading(false);
    });

    return () => {
      alive = false;
      cancelAnimationFrame(frameRef.current);
      controls.dispose();
      renderer.dispose();
      ro.disconnect();
      if (mount.contains(renderer.domElement)) {
        mount.removeChild(renderer.domElement);
      }
    };
  }, [path]);

  return (
    <div className="asset-viewport-glb" ref={mountRef}>
      {loading && !error && (
        <div className="asset-viewport-overlay">Loading…</div>
      )}
      {error && (
        <div className="asset-viewport-overlay asset-error">{error}</div>
      )}
    </div>
  );
}
