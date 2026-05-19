import React, { useState } from 'react';

const FileMenu = () => {
  const [open, setOpen] = useState(false);

  const items = [
    { label: 'New',  shortcut: 'Ctrl+N' },
    { label: 'Load', shortcut: 'Ctrl+O' },
    null,
    { label: 'Undo', shortcut: 'Ctrl+Z' },
    { label: 'Redo', shortcut: 'Ctrl+Y' },
  ];

  return (
    <div className="file-menu" onBlur={() => setOpen(false)} tabIndex={-1}>
      <button className="toolbar-btn" onClick={() => setOpen(o => !o)}>
        File
        <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor" style={{ marginLeft: 4 }}>
          <polygon points="0,2 8,2 4,7" />
        </svg>
      </button>
      {open && (
        <div className="file-menu-dropdown" onMouseDown={e => e.preventDefault()}>
          {items.map((item, i) =>
            item === null
              ? <div key={i} className="menu-divider" />
              : (
                <div key={item.label} className="menu-item" onClick={() => setOpen(false)}>
                  <span>{item.label}</span>
                  {item.shortcut && <span className="menu-shortcut">{item.shortcut}</span>}
                </div>
              )
          )}
        </div>
      )}
    </div>
  );
};

export function Toolbar() {
  return (
    <div className="toolbar">
      <FileMenu />
    </div>
  );
}
