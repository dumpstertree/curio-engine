import React, { useRef, useEffect, useState } from 'react';
import { useEditorStore } from '../store';
import { ViewportCanvas }  from './ViewportCanvas';
import { CustomSelect }    from './CustomSelect';
import type { LogLine }    from '../store';

const RESOLUTION_OPTIONS = [
  { value: '1280x720',  label: '1280 × 720'  },
  { value: '1920x1080', label: '1920 × 1080' },
  { value: '2560x1440', label: '2560 × 1440' },
];

// ── ANSI colour parser ────────────────────────────────────────────────────────
// The `colored` crate emits ESC[<codes>m ... ESC[0m sequences.

interface Span { text: string; color?: string; bold?: boolean; underline?: boolean; }

function parseAnsi(raw: string): Span[] {
  const spans: Span[] = [];
  // Split on ESC[ ... m sequences
  const parts = raw.split(/\x1b\[([0-9;]*)m/);
  let color: string | undefined;
  let bold       = false;
  let underline  = false;

  for (let i = 0; i < parts.length; i++) {
    if (i % 2 === 0) {
      // Plain text segment
      if (parts[i]) spans.push({ text: parts[i], color, bold, underline });
    } else {
      // Code segment
      const codes = parts[i].split(';').map(Number);
      for (const code of codes) {
        if (code === 0)  { color = undefined; bold = false; underline = false; }
        else if (code === 1)  bold      = true;
        else if (code === 4)  underline = true;
        // Standard 8-colour foreground
        else if (code === 30) color = '#555';
        else if (code === 31) color = '#f87171'; // red
        else if (code === 32) color = '#4ade80'; // green
        else if (code === 33) color = '#fbbf24'; // yellow
        else if (code === 34) color = '#60a5fa'; // blue
        else if (code === 35) color = '#c084fc'; // magenta
        else if (code === 36) color = '#34d399'; // cyan
        else if (code === 37) color = '#e5e7eb'; // white
        // Bright variants
        else if (code === 90) color = '#6b7280';
        else if (code === 91) color = '#fca5a5';
        else if (code === 92) color = '#86efac';
        else if (code === 93) color = '#fde68a';
        else if (code === 94) color = '#93c5fd';
        else if (code === 95) color = '#d8b4fe';
        else if (code === 96) color = '#6ee7b7';
        else if (code === 97) color = '#f3f4f6';
      }
    }
  }
  return spans;
}

// truecolor: ESC[38;2;R;G;Bm
function parseTruecolor(raw: string): Span[] {
  const result: Span[] = [];
  // Handle truecolor sequences first, then fall through to standard ANSI
  const tc = raw.replace(/\x1b\[38;2;(\d+);(\d+);(\d+)m/g, (_, r, g, b) => {
    return `\x1b[38;2;${r};${g};${b}m_TC_${r}_${g}_${b}_`;
  });
  // Parse standard sequences
  const spans = parseAnsi(tc);
  // Resolve _TC_ markers back to rgb colors
  for (const span of spans) {
    const m = span.text.match(/^_TC_(\d+)_(\d+)_(\d+)_(.*)$/s);
    if (m) {
      const rgb = `rgb(${m[1]},${m[2]},${m[3]})`;
      if (m[4]) result.push({ ...span, color: rgb, text: m[4] });
    } else {
      result.push(span);
    }
  }
  return result;
}

function AnsiLine({ raw }: { raw: string }) {
  // Handle truecolor (colored crate uses this for .truecolor())
  const spans: Span[] = [];
  const re = /(\x1b\[(?:38;2;\d+;\d+;\d+|\d+(?:;\d+)*)m)/g;
  const parts = raw.split(re);

  let color: string | undefined;
  let bold = false;
  let underline = false;

  for (const part of parts) {
    // Check if it's a control sequence
    const tcMatch = part.match(/^\x1b\[38;2;(\d+);(\d+);(\d+)m$/);
    const stdMatch = part.match(/^\x1b\[(\d+(?:;\d+)*)m$/);

    if (tcMatch) {
      color = `rgb(${tcMatch[1]},${tcMatch[2]},${tcMatch[3]})`;
    } else if (stdMatch) {
      const codes = stdMatch[1].split(';').map(Number);
      for (const code of codes) {
        if (code === 0)  { color = undefined; bold = false; underline = false; }
        else if (code === 1)  bold      = true;
        else if (code === 4)  underline = true;
        else if (code === 31) color = '#f87171';
        else if (code === 32) color = '#4ade80';
        else if (code === 33) color = '#fbbf24';
        else if (code === 34) color = '#60a5fa';
        else if (code === 35) color = '#c084fc';
        else if (code === 36) color = '#34d399';
        else if (code === 37) color = '#e5e7eb';
        else if (code === 90) color = '#6b7280';
        else if (code === 91) color = '#fca5a5';
        else if (code === 92) color = '#86efac';
        else if (code === 93) color = '#fde68a';
        else if (code === 94) color = '#93c5fd';
        else if (code === 95) color = '#d8b4fe';
        else if (code === 96) color = '#6ee7b7';
        else if (code === 97) color = '#f3f4f6';
      }
    } else if (part) {
      spans.push({ text: part, color, bold, underline });
    }
  }

  return (
    <>
      {spans.map((s, i) => (
        <span key={i} style={{
          color:           s.color,
          fontWeight:      s.bold      ? 700       : undefined,
          textDecoration:  s.underline ? 'underline': undefined,
        }}>
          {s.text}
        </span>
      ))}
    </>
  );
}

// ── Compile error modal ───────────────────────────────────────────────────────

function CompileErrorModal() {
  const { compileStatus, compileError, compileStatus: cs } = useEditorStore();
  const [dismissed, setDismissed] = useState(false);

  // Reset dismissed when a new error comes in
  useEffect(() => { if (compileStatus === 'error') setDismissed(false); }, [compileStatus]);

  if (compileStatus !== 'error' || dismissed) return null;

  return (
    <div className="compile-modal-backdrop" onClick={() => setDismissed(true)}>
      <div className="compile-modal" onClick={e => e.stopPropagation()}>
        <div className="compile-modal-header">
          <span>Compile Failed</span>
          <button onClick={() => setDismissed(true)}>✕</button>
        </div>
        <pre className="compile-modal-body">{compileError}</pre>
        <div className="compile-modal-footer">
          <button className="compile-modal-dismiss" onClick={() => setDismissed(true)}>
            Dismiss
          </button>
        </div>
      </div>
    </div>
  );
}

// ── Compile status indicator (non-blocking, shown in viewport) ────────────────

function CompileIndicator() {
  const { compileStatus } = useEditorStore();
  if (compileStatus !== 'compiling') return null;
  return (
    <div className="compile-indicator compile-compiling">
      <span className="compile-spinner" /> Compiling…
    </div>
  );
}

// ── Console overlay ───────────────────────────────────────────────────────────

function ConsoleOverlay() {
  const { consoleOpen, logs } = useEditorStore();
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  if (!consoleOpen) return null;

  function levelColor(level: LogLine['level']): string {
    if (level === 'error') return '#f87171';
    if (level === 'warn')  return '#fbbf24';
    return '#9cdcfe';
  }

  return (
    <div className="console-overlay">
      <div className="console-lines">
        {logs.length === 0 && (
          <div className="console-empty">No output yet</div>
        )}
        {logs.map((line, i) => (
          <div key={i} className="console-line">
            <span className="console-time">{line.time}</span>
            <span className="console-msg">
              <AnsiLine raw={line.message} />
            </span>
          </div>
        ))}
        <div ref={bottomRef} />
      </div>
    </div>
  );
}

// ── Viewport toolbar ──────────────────────────────────────────────────────────

function ViewportToolbar() {
  const { consoleOpen, toggleConsole, unreadLogs } = useEditorStore();
  const [resolution, setResolution] = useState('1280x720');

  return (
    <div className="viewport-toolbar">
      <div className="viewport-toolbar-left">
        <label className="vt-label">Resolution</label>
        <CustomSelect
          value={resolution}
          options={RESOLUTION_OPTIONS}
          onChange={setResolution}
          className="resolution-dropdown"
        />
      </div>
      <div className="viewport-toolbar-right">
        <button
          className={`vt-console-btn ${consoleOpen ? 'active' : ''}`}
          onClick={toggleConsole}
          title="Toggle console"
        >
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.3">
            <rect x="1" y="1" width="10" height="10" rx="1"/>
            <polyline points="3,4 5,6 3,8"/>
            <line x1="6" y1="8" x2="9" y2="8"/>
          </svg>
          Console
          {!consoleOpen && unreadLogs > 0 && (
            <span className="vt-badge">{unreadLogs > 99 ? '99+' : unreadLogs}</span>
          )}
        </button>
      </div>
    </div>
  );
}

// ── CenterPanel ───────────────────────────────────────────────────────────────

export function CenterPanel() {
  const { mode, play, stop, pause } = useEditorStore();

  return (
    <div className="center-panel">
      <ViewportToolbar />

      <div className="center-viewport" style={{ position: 'relative' }}>
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

        <CompileIndicator />
        <ConsoleOverlay />
        <CompileErrorModal />
      </div>

      <div className="play-bar">
        <div className="play-bar-left" />

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
