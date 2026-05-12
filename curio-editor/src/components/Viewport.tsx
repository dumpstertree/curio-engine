import React from 'react';
import { useEditorStore } from '../store';

export function Viewport() {
  const { mode, play } = useEditorStore();

  return (
    <div className="viewport-area">
      {/* tabs */}
      <div className="viewport-tabs">
        <div className="viewport-tab active">Viewport</div>
      </div>

      {/* content */}
      <div className="viewport-content">
        <div className="viewport-grid" />

        {mode === 'stopped' && (
          <div className="viewport-label">
            <div className="viewport-label-title">No active game window</div>
            <div className="viewport-label-sub">
              Press{' '}
              <span
                className="viewport-play-hint"
                onClick={play}
              >
                ▶ Play
              </span>{' '}
              to launch
            </div>
          </div>
        )}

        {mode === 'playing' && (
          <div className="viewport-playing">
            <div className="viewport-playing-indicator">
              <div className="play-dot" />
              Game running in separate window
            </div>
            <div style={{ fontSize: 11, color: 'var(--text-muted)' }}>
              Viewport streaming coming soon
            </div>
          </div>
        )}

        {mode === 'paused' && (
          <div className="viewport-playing">
            <div className="viewport-playing-indicator" style={{ color: 'var(--pause)' }}>
              <div className="play-dot" style={{ background: 'var(--pause)', animationPlayState: 'paused' }} />
              Paused
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
