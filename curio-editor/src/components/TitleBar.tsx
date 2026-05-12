import React from 'react';
import { useEditorStore } from '../store';

export function TitleBar() {
  const { mode, play, stop, pause } = useEditorStore();

  return (
    <div className="titlebar">
      {/* Menu */}
      <div className="titlebar-menu">
        {['File', 'Edit', 'View', 'Run'].map((item) => (
          <div key={item} className="titlebar-menu-item">{item}</div>
        ))}
      </div>

      {/* Play controls — center */}
      <div className="titlebar-center">
        {/* Play */}
        <button
          className={`play-btn ${mode === 'playing' ? 'play-active' : ''}`}
          onClick={() => mode === 'playing' ? undefined : play()}
          title="Play"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
            <polygon points="2,1 9,5 2,9" />
          </svg>
          Play
        </button>

        {/* Pause */}
        <button
          className={`play-btn ${mode === 'paused' ? 'pause-active' : ''}`}
          onClick={() => mode === 'playing' ? pause() : undefined}
          disabled={mode === 'stopped'}
          title="Pause"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
            <rect x="2" y="1" width="2.5" height="8" />
            <rect x="5.5" y="1" width="2.5" height="8" />
          </svg>
          Pause
        </button>

        {/* Stop */}
        <button
          className="play-btn"
          onClick={() => mode !== 'stopped' ? stop() : undefined}
          disabled={mode === 'stopped'}
          title="Stop"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
            <rect x="1.5" y="1.5" width="7" height="7" />
          </svg>
          Stop
        </button>
      </div>

      {/* Right — project name */}
      <div className="titlebar-right">
        curio editor
      </div>
    </div>
  );
}
