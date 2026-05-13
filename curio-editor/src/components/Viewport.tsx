import React, { useEffect, useRef } from 'react';
import { useEditorStore } from '../store';
import { api } from '../api';

export function Viewport() {
  const { mode, play } = useEditorStore();
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (mode !== 'playing' && mode !== 'paused') return;

    const unlisten = api.onViewportFrame((dataUrl) => {
      const canvas = canvasRef.current;
      if (!canvas) return;
      const ctx = canvas.getContext('2d');
      if (!ctx) return;

      const img = new Image();
      img.onload = () => ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
      img.src = dataUrl;
    });

    return unlisten;
  }, [mode]);

  return (
    <div className="viewport-area">
      <div className="viewport-tabs">
        <div className="viewport-tab active">Viewport</div>
      </div>

      <div className="viewport-content">
        <div className="viewport-grid" />

        {/* game canvas — always present, hidden when stopped */}
        <canvas
          ref={canvasRef}
          width={1280}
          height={720}
          className="viewport-canvas"
          style={{ display: mode !== 'stopped' ? 'block' : 'none' }}
        />

        {mode === 'stopped' && (
          <div className="viewport-label">
            <div className="viewport-label-title">No active game window</div>
            <div className="viewport-label-sub">
              Press{' '}
              <span className="viewport-play-hint" onClick={play}>
                ▶ Play
              </span>{' '}
              to launch
            </div>
          </div>
        )}

        {mode === 'paused' && (
          <div className="viewport-paused-overlay">⏸ Paused</div>
        )}
      </div>
    </div>
  );
}