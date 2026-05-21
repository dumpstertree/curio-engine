import React from 'react';
import { useEditorStore } from '../store';
import { ViewportCanvas } from './ViewportCanvas';
import { CustomSelect }   from './CustomSelect';

const RESOLUTION_OPTIONS = [
  { value: '1280x720',  label: '1280 × 720'  },
  { value: '1920x1080', label: '1920 × 1080' },
  { value: '2560x1440', label: '2560 × 1440' },
];

export function CenterPanel() {
  const { mode, play, stop, pause } = useEditorStore();
  const [resolution, setResolution] = React.useState('1280x720');

  return (
    <div className="center-panel">

      {/* Viewport */}
      <div className="center-viewport">
        {mode === 'stopped' ? (
          <div className="viewport-idle">
            <div className="viewport-idle-icon">
              <svg width="40" height="40" viewBox="0 0 40 40" fill="none" stroke="currentColor" strokeWidth="1.2">
                <circle cx="20" cy="20" r="17" />
                <polygon points="16,13 30,20 16,27" fill="currentColor" stroke="none" />
              </svg>
            </div>
            <div className="viewport-idle-label">Press Play to launch</div>
          </div>
        ) : (
          <>
            <ViewportCanvas />
            {mode === 'paused' && (
              <div className="viewport-paused-badge">⏸ Paused</div>
            )}
          </>
        )}
      </div>

      {/* Play controls bar */}
      <div className="play-bar">
        <div className="play-bar-left">
          <label className="play-bar-label">Resolution</label>
          <CustomSelect
            value={resolution}
            options={RESOLUTION_OPTIONS}
            onChange={setResolution}
            className="resolution-dropdown"
          />
        </div>

        <div className="play-bar-center">
          <button
            className={`ctrl-btn ${mode === 'playing' ? 'active-play' : ''}`}
            onClick={play}
            disabled={mode === 'playing'}
          >
            <svg width="11" height="11" viewBox="0 0 11 11" fill="currentColor">
              <polygon points="2,1 10,5.5 2,10" />
            </svg>
            Play
          </button>

          <button
            className={`ctrl-btn ${mode === 'paused' ? 'active-pause' : ''}`}
            onClick={pause}
            disabled={mode === 'stopped'}
          >
            <svg width="11" height="11" viewBox="0 0 11 11" fill="currentColor">
              <rect x="1.5" y="1" width="3" height="9" />
              <rect x="6.5" y="1" width="3" height="9" />
            </svg>
            Pause
          </button>

          <button
            className="ctrl-btn"
            onClick={stop}
            disabled={mode === 'stopped'}
          >
            <svg width="11" height="11" viewBox="0 0 11 11" fill="currentColor">
              <rect x="1" y="1" width="9" height="9" />
            </svg>
            Stop
          </button>
        </div>

        <div className="play-bar-right">
          <span className={`mode-pill mode-${mode}`}>
            {mode === 'playing' && <><span className="mode-dot" /> Playing</>}
            {mode === 'paused'  && '⏸ Paused'}
            {mode === 'stopped' && '■ Stopped'}
          </span>
        </div>
      </div>
    </div>
  );
}
