import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import JSZip from 'jszip';
import { spine } from '../../../vendor/spine38/spine-threejs.mjs';
import { api } from '../../../api';

/** Loads a .glb file and returns its root scene Object3D, ready to add to a scene. */
export async function loadGlbObject(path: string): Promise<THREE.Object3D> {
  const bytes  = await api.readFileBytes(path);
  const u8     = new Uint8Array(bytes);
  const buffer = u8.buffer.slice(u8.byteOffset, u8.byteOffset + u8.byteLength);

  const loader = new GLTFLoader();
  return new Promise((resolve, reject) => {
    loader.parse(buffer, '', gltf => resolve(gltf.scene), err => reject(err));
  });
}

/** Loads a .anim (Spine 3.8 zip) file and returns a live SkeletonMesh.
 *  Caller is responsible for calling `.update(delta)` each frame and `.dispose()` on cleanup. */
export async function loadAnimMesh(path: string): Promise<any> {
  const bytes = await api.readFileBytes(path);
  const u8    = new Uint8Array(bytes);

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

  const pngBlob     = new Blob([pngBytes], { type: 'image/png' });
  const imageBitmap = await createImageBitmap(pngBlob);

  const atlas = new spine.TextureAtlas(atlasText, (_p: string) => {
    return new spine.threejs.ThreeJsTexture(imageBitmap as any);
  });

  const loader   = new spine.AtlasAttachmentLoader(atlas);
  const skelJson = new spine.SkeletonJson(loader);
  const skelData = skelJson.readSkeletonData(jsonText);

  const skeletonMesh = new spine.threejs.SkeletonMesh(skelData, (_params: any) => {});

  const animNames: string[] = skelData.animations.map((a: any) => a.name);
  if (animNames.length > 0) {
    skeletonMesh.state.setAnimation(0, animNames[0], true);
  }
  skeletonMesh.skeleton.setToSetupPose();
  skeletonMesh.skeleton.updateWorldTransform();

  return skeletonMesh;
}
