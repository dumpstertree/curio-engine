import React, { useEffect, useRef } from 'react';
import { useEditorStore } from '../store';
import { api, InputEvent } from '../api';

// These must match CAPTURE_WIDTH / CAPTURE_HEIGHT in capture.rs
const FRAME_WIDTH = 1280;
const FRAME_HEIGHT = 720;

export function ViewportCanvas() {
  const { mode } = useEditorStore();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rafRef = useRef<number | null>(null);

  // ── Frame polling loop ────────────────────────────────────────────────────
  useEffect(() => {
    if (mode === 'stopped') return;

    let running = true;

    const poll = async () => {
      if (!running) return;

      try {
        const bytes = await api.getFrame();

        if (bytes && canvasRef.current) {
          const canvas = canvasRef.current;
          const ctx = canvas.getContext('2d');
          if (ctx) {
            // Raw RGBA — build ImageData directly, no PNG decode
            const uint8 = new Uint8ClampedArray(bytes);
            const imageData = new ImageData(uint8, FRAME_WIDTH, FRAME_HEIGHT);
            ctx.putImageData(imageData, 0, 0);
          }
        }
      } catch (e) {
        console.error('[ViewportCanvas] getFrame error:', e);
      }

      rafRef.current = requestAnimationFrame(poll);
    };

    rafRef.current = requestAnimationFrame(poll);

    return () => {
      running = false;
      if (rafRef.current !== null) {
        cancelAnimationFrame(rafRef.current);
        rafRef.current = null;
      }
    };
  }, [mode]);

  // ── Clear when stopped ────────────────────────────────────────────────────
  useEffect(() => {
    if (mode !== 'stopped') return;
    const canvas = canvasRef.current;
    if (!canvas) return;
    canvas.getContext('2d')?.clearRect(0, 0, canvas.width, canvas.height);
  }, [mode]);

  // ── Input forwarding ──────────────────────────────────────────────────────
  const sendButton = (button: number, pressed: boolean) => {
    if (mode !== 'playing') return;
    api.sendInput({ type: 'Button', code: button, pressed });
  };

  const sendAxis = (x: number, y: number) => {
    if (mode !== 'playing') return;
    api.sendInput({ type: 'Axis', code: 0, x, y });
  };

  return (
    <canvas
      ref={canvasRef}
      width={FRAME_WIDTH}
      height={FRAME_HEIGHT}
      className="viewport-canvas"
      // Pointer events — position relative to canvas top-left
      onPointerMove={e => {
        const rect = (e.target as HTMLCanvasElement).getBoundingClientRect();
        sendAxis(e.clientX - rect.left, e.clientY - rect.top);
      }}
      onPointerDown={e => {
        (e.target as HTMLCanvasElement).setPointerCapture(e.pointerId);
        sendButton(e.button, true);
      }}
      onPointerUp={e => sendButton(e.button, false)}
      // Keyboard events — canvas needs tabIndex to receive these
      tabIndex={0}
      onKeyDown={e => {
        e.preventDefault();
        if (mode !== 'playing') return;
        api.sendInput({ type: 'Button', code: e.keyCode, pressed: true });
      }}
      onKeyUp={e => {
        e.preventDefault();
        if (mode !== 'playing') return;
        api.sendInput({ type: 'Button', code: e.keyCode, pressed: false });
      }}
      // Prevent right-click context menu stealing events
      onContextMenu={e => e.preventDefault()}
    />
  );
}