import React, { useEffect, useRef } from 'react';
import { useEditorStore } from '../store';
import { api } from '../api';

// Must match CAPTURE_WIDTH / CAPTURE_HEIGHT in capture.rs
const FRAME_WIDTH = 1280;
const FRAME_HEIGHT = 720;

export function ViewportCanvas() {
  const { mode } = useEditorStore();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const imageDataRef = useRef<ImageData | null>(null);
  const bitmapPromiseRef = useRef<Promise<ImageBitmap> | null>(null);

  // ── Frame stream ──────────────────────────────────────────────────────────
  useEffect(() => {
    if (mode === 'stopped') return;

    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Allocate once — reused every frame via .data.set()
    imageDataRef.current = new ImageData(FRAME_WIDTH, FRAME_HEIGHT);

    // Frame counter for timing logs
    let frameCount = 0;
    let lastLog = performance.now();

    const onFrame = (bytes: Uint8ClampedArray) => {
      if (!imageDataRef.current) return;

      const t0 = performance.now();

      // Draw the previous frame's bitmap while we process this one —
      // bitmapPromiseRef holds the createImageBitmap started last call
      if (bitmapPromiseRef.current) {
        bitmapPromiseRef.current.then(bitmap => {
          ctx.drawImage(bitmap, 0, 0);
          bitmap.close();
        });
      }

      // Overwrite ImageData in place — no allocation
      imageDataRef.current.data.set(bytes);

      // Kick createImageBitmap for this frame — will be drawn next call
      bitmapPromiseRef.current = createImageBitmap(imageDataRef.current);

      // Log fps ~once per second
      frameCount++;
      const now = performance.now();
      if (now - lastLog >= 1000) {
        const fps = (frameCount / ((now - lastLog) / 1000)).toFixed(1);
        console.log(`[viewport] ${fps} fps  set+kick=${(performance.now() - t0).toFixed(1)}ms`);
        frameCount = 0;
        lastLog = now;
      }
    };

    // Establish the push channel — async because it awaits the Tauri import
    // and the stream_frames invoke before returning the cleanup fn.
    let stopStream: (() => void) | null = null;
    let unmounted = false;

    api.startFrameStream(onFrame).then(cleanup => {
      if (unmounted) {
        // Component already unmounted before stream started — clean up immediately
        cleanup();
      } else {
        stopStream = cleanup;
      }
    }).catch(e => {
      console.error('[ViewportCanvas] startFrameStream failed:', e);
    });

    return () => {
      unmounted = true;
      if (stopStream) stopStream();
      bitmapPromiseRef.current?.then(b => b.close()).catch(() => { });
      bitmapPromiseRef.current = null;
      imageDataRef.current = null;
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
      onPointerMove={e => {
        const rect = (e.target as HTMLCanvasElement).getBoundingClientRect();
        sendAxis(e.clientX - rect.left, e.clientY - rect.top);
      }}
      onPointerDown={e => {
        (e.target as HTMLCanvasElement).setPointerCapture(e.pointerId);
        sendButton(e.button, true);
      }}
      onPointerUp={e => sendButton(e.button, false)}
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
      onContextMenu={e => e.preventDefault()}
    />
  );
}