import React, { useState, useRef, useEffect } from 'react';

interface Option {
  value: string;
  label: string;
}

interface Props {
  value:    string;
  options:  Option[];
  onChange: (value: string) => void;
  className?: string;
}

export function CustomSelect({ value, options, onChange, className = '' }: Props) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  const selected = options.find(o => o.value === value);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, []);

  return (
    <div ref={ref} className={`custom-select ${className} ${open ? 'open' : ''}`}>
      <button
        className="custom-select-trigger"
        onClick={() => setOpen(o => !o)}
      >
        <span className="custom-select-value">{selected?.label ?? value}</span>
        <svg
          className={`custom-select-arrow ${open ? 'open' : ''}`}
          width="10" height="10" viewBox="0 0 10 10" fill="currentColor"
        >
          <polygon points="1,3 9,3 5,8" />
        </svg>
      </button>

      {open && (
        <div className="custom-select-menu">
          {options.map(opt => (
            <div
              key={opt.value}
              className={`custom-select-option ${opt.value === value ? 'selected' : ''}`}
              onClick={() => { onChange(opt.value); setOpen(false); }}
            >
              {opt.label}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
