import React, { useEffect, useRef } from 'react';
import { useEditorStore } from '../store';
import { api } from '../api';

export function ViewportCanvas() {
  const { mode } = useEditorStore();
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (mode === 'stopped') return;

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

  // clear canvas when stopped
  useEffect(() => {
    if (mode !== 'stopped') return;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    ctx?.clearRect(0, 0, canvas.width, canvas.height);
  }, [mode]);

  return (
    <canvas
      ref={canvasRef}
      width={1280}
      height={720}
      className="viewport-canvas"
    />
  );
}
