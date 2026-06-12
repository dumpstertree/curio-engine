import React from 'react';
import type { PngInfo }        from './PngViewport';
import type { GlbInfo }        from './GlbViewport';
import type { AnimInfo, AnimPlaybackState } from './AnimViewport';

// ─── shared field row ─────────────────────────────────────────

function InfoRow({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="field-row" style={{ paddingLeft: 10 }}>
      <span className="field-key">{label}</span>
      <span className="field-val fv-num">{value}</span>
    </div>
  );
}

function SectionHeader({ title }: { title: string }) {
  return (
    <div className="comp-header" style={{ cursor: 'default' }}>
      <span className="comp-name">{title}</span>
    </div>
  );
}

// ─── PNG inspector ────────────────────────────────────────────

function PngInspector({ info }: { info: PngInfo }) {
  return (
    <div className="comp-block">
      <SectionHeader title="Texture" />
      <div className="comp-fields-list">
        <InfoRow label="width"  value={`${info.width} px`} />
        <InfoRow label="height" value={`${info.height} px`} />
        <InfoRow label="ratio"  value={info.aspectRatio} />
        <InfoRow label="size"   value={`${info.sizeKb} KB`} />
      </div>
    </div>
  );
}

// ─── GLB inspector ────────────────────────────────────────────

function GlbInspector({ info }: { info: GlbInfo }) {
  return (
    <div className="comp-block">
      <SectionHeader title="Mesh" />
      <div className="comp-fields-list">
        <InfoRow label="triangles" value={info.triangles.toLocaleString()} />
        <InfoRow label="vertices"  value={info.vertices.toLocaleString()} />
        <InfoRow label="meshes"    value={info.meshes} />
        <InfoRow label="materials" value={info.materials} />
        <InfoRow label="nodes"     value={info.nodes} />
        <InfoRow label="size"      value={`${info.sizeKb} KB`} />
      </div>
    </div>
  );
}

// ─── ANIM inspector ───────────────────────────────────────────

interface AnimInspectorProps {
  info:       AnimInfo;
  playback:   AnimPlaybackState | null;
  onPlay:     (name: string) => void;
}

function fmt(t: number) {
  return t.toFixed(2) + 's';
}

function AnimInspector({ info, playback, onPlay }: AnimInspectorProps) {
  const progress = playback && playback.duration > 0
    ? playback.time / playback.duration
    : 0;

  return (
    <>
      {/* Stats */}
      <div className="comp-block">
        <SectionHeader title="Skeleton" />
        <div className="comp-fields-list">
          <InfoRow label="bones"  value={info.bones} />
          <InfoRow label="slots"  value={info.slots} />
          <InfoRow label="anims"  value={info.animations.length} />
          <InfoRow label="size"   value={`${info.sizeKb} KB`} />
        </div>
      </div>

      {/* Playback progress */}
      <div className="comp-block">
        <SectionHeader title="Playback" />
        <div className="anim-progress-block">
          <div className="anim-progress-label">
            <span className="anim-playing-name">{playback?.current ?? '—'}</span>
            <span className="anim-time-readout">
              {playback ? `${fmt(playback.time)} / ${fmt(playback.duration)}` : ''}
            </span>
          </div>
          <div className="anim-progress-bar-track">
            <div
              className="anim-progress-bar-fill"
              style={{ width: `${Math.min(progress * 100, 100)}%` }}
            />
          </div>
        </div>
      </div>

      {/* Animation list */}
      <div className="comp-block">
        <SectionHeader title="Animations" />
        <div className="anim-list">
          {info.animations.map(name => (
            <div
              key={name}
              className={`anim-list-row${playback?.current === name ? ' active' : ''}`}
              onClick={() => onPlay(name)}
            >
              <span className="anim-list-icon">
                {playback?.current === name
                  ? <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor"><rect x="1" y="1" width="2" height="6"/><rect x="5" y="1" width="2" height="6"/></svg>
                  : <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor"><polygon points="1,1 7,4 1,7"/></svg>
                }
              </span>
              <span className="anim-list-name">{name}</span>
            </div>
          ))}
        </div>
      </div>
    </>
  );
}

// ─── main panel ──────────────────────────────────────────────

interface Props {
  fileName:   string | null;
  pngInfo:    PngInfo | null;
  glbInfo:    GlbInfo | null;
  animInfo:   AnimInfo | null;
  animPlayback: AnimPlaybackState | null;
  onPlayAnim: (name: string) => void;
}

export function AssetInspectorView({ fileName, pngInfo, glbInfo, animInfo, animPlayback, onPlayAnim }: Props) {
  const subtitle = pngInfo ? 'PNG Texture' : glbInfo ? 'GLB Mesh' : animInfo ? 'Spine Animation' : 'Loading…';

  return (
    <div className="inspector-view">
      <div className="panel-header">
        <span className="panel-title">Inspector</span>
      </div>

      {!fileName ? (
        <div className="panel-empty">Select an asset</div>
      ) : (
        <>
          <div className="inspector-header">
            <div className="inspector-subject-name">{fileName}</div>
            <div className="inspector-subject-meta">{subtitle}</div>
          </div>

          <div className="inspector-content">
            {pngInfo  && <PngInspector  info={pngInfo} />}
            {glbInfo  && <GlbInspector  info={glbInfo} />}
            {animInfo && (
              <AnimInspector
                info={animInfo}
                playback={animPlayback}
                onPlay={onPlayAnim}
              />
            )}
            {!pngInfo && !glbInfo && !animInfo && (
              <div className="panel-empty">Loading asset data…</div>
            )}
          </div>
        </>
      )}
    </div>
  );
}
