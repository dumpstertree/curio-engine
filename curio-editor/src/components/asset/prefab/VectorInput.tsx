import React, { useState, useEffect } from 'react';
import type { Vec3 } from './prefabTypes';

interface NumberFieldProps {
  label: string;
  value: number;
  onCommit: (v: number) => void;
}

function NumberField({ label, value, onCommit }: NumberFieldProps) {
  const [text, setText] = useState(String(value));

  useEffect(() => { setText(String(value)); }, [value]);

  function commit() {
    const n = parseFloat(text);
    if (Number.isFinite(n)) onCommit(n);
    else setText(String(value));
  }

  return (
    <label className="vec-axis">
      <span className="vec-axis-label">{label}</span>
      <input
        className="vec-axis-input"
        type="text"
        inputMode="decimal"
        value={text}
        onChange={e => setText(e.target.value)}
        onBlur={commit}
        onKeyDown={e => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
      />
    </label>
  );
}

interface Vec3InputProps {
  value:    Vec3;
  onChange: (v: Vec3) => void;
  /** Hide the Z axis (e.g. for transform2d position/scale) */
  is2d?: boolean;
}

export function Vec3Input({ value, onChange, is2d }: Vec3InputProps) {
  return (
    <div className="vec3-input">
      <NumberField label="X" value={value.x} onCommit={x => onChange({ ...value, x })} />
      <NumberField label="Y" value={value.y} onCommit={y => onChange({ ...value, y })} />
      {!is2d && <NumberField label="Z" value={value.z} onCommit={z => onChange({ ...value, z })} />}
    </div>
  );
}
