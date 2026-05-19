import React from 'react';
import { useEditorStore } from '../../store';
import { ViewportCanvas } from '../ViewportCanvas';

const RESOLUTIONS = ['1280 × 720', '1920 × 1080', '2560 × 1440'];

export function PlayModeTab() {
  const { mode, play, stop, pause } = useEditorStore();

  return (
    <div className="play-mode-tab">
      {/* Viewport */}
      <div className="play-viewport-container">
        <div className="play-viewport-inner">
          {mode === 'stopped' ? (
            <div className="play-mode-idle">
              <div className="play-mode-idle-icon">
                <svg width="48" height="48" viewBox="0 0 48 48" fill="none" stroke="currentColor" strokeWidth="1.5">
                  <circle cx="24" cy="24" r="20" />
                  <polygon points="20,16 36,24 20,32" fill="currentColor" stroke="none" />
                </svg>
              </div>
              <div className="play-mode-idle-label">Press Play to launch</div>
              <button className="play-mode-launch-btn" onClick={play}>
                ▶ Play
              </button>
            </div>
          ) : (
            <>
              <ViewportCanvas />
              {mode === 'paused' && (
                <div className="play-mode-paused-overlay">
                  <span>⏸ Paused</span>
                </div>
              )}
            </>
          )}
        </div>
      </div>

      {/* Controls bar */}
      <div className="play-controls-bar">
        {/* Resolution */}
        <div className="play-controls-left">
          <label className="control-label">Resolution</label>
          <select className="resolution-select">
            {RESOLUTIONS.map(r => (
              <option key={r}>{r}</option>
            ))}
          </select>
        </div>

        {/* Play / Pause / Stop */}
        <div className="play-controls-center">
          <button
            className={`ctrl-btn ${mode === 'playing' ? 'active-play' : ''}`}
            onClick={play}
            disabled={mode === 'playing'}
            title="Play"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
              <polygon points="3,1 11,6 3,11" />
            </svg>
            Play
          </button>

          <button
            className={`ctrl-btn ${mode === 'paused' ? 'active-pause' : ''}`}
            onClick={pause}
            disabled={mode === 'stopped'}
            title="Pause"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
              <rect x="2" y="1" width="3" height="10" />
              <rect x="7" y="1" width="3" height="10" />
            </svg>
            Pause
          </button>

          <button
            className="ctrl-btn"
            onClick={stop}
            disabled={mode === 'stopped'}
            title="Stop"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
              <rect x="1" y="1" width="10" height="10" />
            </svg>
            Stop
          </button>
        </div>

        {/* Mode indicator */}
        <div className="play-controls-right">
          <span className={`mode-badge mode-${mode}`}>
            {mode === 'playing' && <><span className="mode-dot" />Playing</>}
            {mode === 'paused'  && '⏸ Paused'}
            {mode === 'stopped' && '■ Stopped'}
          </span>
        </div>
      </div>
    </div>
  );
}
