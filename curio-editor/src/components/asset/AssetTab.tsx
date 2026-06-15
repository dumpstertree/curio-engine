import React, { useState } from 'react';
import { AssetFileTree }      from './AssetFileTree';
import { PngViewport }        from './PngViewport';
import { GlbViewport }        from './GlbViewport';
import { AnimViewport }       from './AnimViewport';
import { AssetInspectorView } from './AssetInspectorView';
import { PrefabLoader }       from './prefab/PrefabLoader';
import type { DirEntry }      from '../../api';
import type { PngInfo }       from './PngViewport';
import type { GlbInfo }       from './GlbViewport';
import type { AnimInfo, AnimPlaybackState } from './AnimViewport';

function fileExt(name: string): string {
  const dot = name.lastIndexOf('.');
  return dot >= 0 ? name.slice(dot).toLowerCase() : '';
}

export function AssetTab() {
  const [selectedEntry, setSelectedEntry] = useState<DirEntry | null>(null);
  const [pngInfo,       setPngInfo]       = useState<PngInfo | null>(null);
  const [glbInfo,       setGlbInfo]       = useState<GlbInfo | null>(null);
  const [animInfo,      setAnimInfo]      = useState<AnimInfo | null>(null);
  const [animPlayback,  setAnimPlayback]  = useState<AnimPlaybackState | null>(null);
  // Name of animation the inspector wants to play (drives AnimViewport)
  const [requestedAnim, setRequestedAnim] = useState<string | null>(null);

  function handleSelect(entry: DirEntry) {
    setPngInfo(null);
    setGlbInfo(null);
    setAnimInfo(null);
    setAnimPlayback(null);
    setRequestedAnim(null);
    setSelectedEntry(entry);
  }

  const ext   = selectedEntry ? fileExt(selectedEntry.name) : null;
  const isPng  = ext === '.png';
  const isGlb  = ext === '.glb';
  const isAnim = ext === '.anim';
  const isComp = ext === '.comp';

  return (
    <>
      {/* Left: file tree */}
      <div className="left-panel">
        <div className="panel-header">
          <span className="panel-title">Assets</span>
        </div>
        <div className="left-panel-content">
          <AssetFileTree
            selectedPath={selectedEntry?.path ?? null}
            onSelect={handleSelect}
          />
        </div>
      </div>

      {/* Center + Inspector: prefab gets its own combined component */}
      {selectedEntry && isComp ? (
        <PrefabLoader key={selectedEntry.path} path={selectedEntry.path} name={selectedEntry.name} />
      ) : (
        <>
          {/* Center: viewport */}
          <div className="center-panel">
            <div className="center-viewport">
              {!selectedEntry && (
                <div className="viewport-idle">
                  <div className="viewport-idle-icon">
                    <svg width="40" height="40" viewBox="0 0 40 40" fill="none" stroke="currentColor" strokeWidth="1.2">
                      <rect x="4" y="8" width="32" height="24" rx="2" />
                      <circle cx="14" cy="18" r="3" />
                      <polyline points="4,28 12,20 18,26 26,16 36,28" />
                    </svg>
                  </div>
                  <div className="viewport-idle-label">Select an asset to preview</div>
                </div>
              )}

              {selectedEntry && isPng && (
                <PngViewport
                  key={selectedEntry.path}
                  path={selectedEntry.path}
                  onInfo={setPngInfo}
                />
              )}

              {selectedEntry && isGlb && (
                <GlbViewport
                  key={selectedEntry.path}
                  path={selectedEntry.path}
                  onInfo={setGlbInfo}
                />
              )}

              {selectedEntry && isAnim && (
                <AnimViewport
                  key={selectedEntry.path}
                  path={selectedEntry.path}
                  onInfo={setAnimInfo}
                  onPlayback={setAnimPlayback}
                  playAnim={requestedAnim}
                />
              )}
            </div>
          </div>

          {/* Right: inspector */}
          <AssetInspectorView
            fileName={selectedEntry?.name ?? null}
            pngInfo={pngInfo}
            glbInfo={glbInfo}
            animInfo={animInfo}
            animPlayback={animPlayback}
            onPlayAnim={name => setRequestedAnim(name)}
          />
        </>
      )}
    </>
  );
}
