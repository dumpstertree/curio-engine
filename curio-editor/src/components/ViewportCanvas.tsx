import React, { useEffect, useRef } from 'react';
import { useEditorStore } from '../store';
import { api } from '../api';

// Must match CAPTURE_WIDTH / CAPTURE_HEIGHT in capture.rs
const FRAME_WIDTH = 1280;
const FRAME_HEIGHT = 720;

export function ViewportCanvas() {
  const { mode } = useEditorStore();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rafRef = useRef<number | null>(null);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Persistent ImageData — allocated once, reused every frame.
  // .data.set() overwrites in place with no heap allocation.
  const imageDataRef = useRef<ImageData | null>(null);

  // Pipelining: bitmapPromise holds the createImageBitmap call for the
  // frame we just fetched so we can draw it next tick without extra awaiting.
  const bitmapPromiseRef = useRef<Promise<ImageBitmap> | null>(null);

  // ── Frame polling loop ────────────────────────────────────────────────────
  useEffect(() => {
    if (mode === 'stopped') return;

    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Allocate the reusable ImageData once for the lifetime of this effect
    imageDataRef.current = new ImageData(FRAME_WIDTH, FRAME_HEIGHT);

    let running = true;
    let missCount = 0;

    const scheduleNext = (delay = 0) => {
      if (!running) return;
      if (delay > 0) {
        timeoutRef.current = setTimeout(() => {
          rafRef.current = requestAnimationFrame(poll);
        }, delay);
      } else {
        rafRef.current = requestAnimationFrame(poll);
      }
    };

    const poll = async () => {
      if (!running) return;

      try {
        const t0 = performance.now();

        // ── Draw last frame's bitmap while fetching this frame ────────────
        // Await the bitmap we kicked off at the end of the previous iteration
        // concurrently with the IPC fetch below — but actually we draw it
        // first to get it on screen as early as possible.
        const prevBitmap = bitmapPromiseRef.current
          ? await bitmapPromiseRef.current
          : null;

        if (prevBitmap) {
          ctx.drawImage(prevBitmap, 0, 0);
          prevBitmap.close();
        }

        // ── Fetch this frame ──────────────────────────────────────────────
        const bytes = await api.getFrame();
        const t1 = performance.now();

        if (bytes && imageDataRef.current) {
          missCount = 0;

          // Overwrite reusable ImageData in place — no allocation
          imageDataRef.current.data.set(bytes);
          const t2 = performance.now();

          // Kick createImageBitmap for this frame — don't await it yet.
          // It will be drawn at the start of the next poll iteration,
          // overlapping with the next IPC fetch.
          bitmapPromiseRef.current = createImageBitmap(imageDataRef.current);
          const t3 = performance.now();

          // Log timing ~once per second
          if (Math.random() < 0.017) {
            console.log(
              `[viewport] fetch=${(t1 - t0).toFixed(1)}ms  ` +
              `set=${(t2 - t1).toFixed(1)}ms  ` +
              `kickBitmap=${(t3 - t2).toFixed(1)}ms`
            );
          }
        } else {
          // No new frame — clear the pipeline so we don't redraw stale bitmaps
          bitmapPromiseRef.current = null;
          missCount++;
        }
      } catch (e) {
        console.error('[ViewportCanvas] poll error:', e);
        bitmapPromiseRef.current = null;
      }

      // ── Schedule next iteration ───────────────────────────────────────
      // After 5 consecutive misses slow to ~30Hz to reduce IPC overhead
      // when the game is producing frames slower than the display rate.
      scheduleNext(missCount > 5 ? 33 : 0);
    };

    // Kick the first iteration
    bitmapPromiseRef.current = null;
    rafRef.current = requestAnimationFrame(poll);

    return () => {
      running = false;
      if (rafRef.current !== null) { cancelAnimationFrame(rafRef.current); rafRef.current = null; }
      if (timeoutRef.current !== null) { clearTimeout(timeoutRef.current); timeoutRef.current = null; }
      // Let any in-flight bitmap resolve and close to avoid leaking GPU memory
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