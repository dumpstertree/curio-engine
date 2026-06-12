import React, { useEffect, useState } from 'react';
import { api } from '../../api';

export interface PngInfo {
  width:      number;
  height:     number;
  aspectRatio: string;
  sizeKb:     number;
  dataUrl:    string;
}

interface Props {
  path: string;
  onInfo: (info: PngInfo | null) => void;
}

export function PngViewport({ path, onInfo }: Props) {
  const [dataUrl, setDataUrl] = useState<string | null>(null);
  const [error,   setError]   = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    setError(null);
    setDataUrl(null);
    onInfo(null);

    api.readFileBytes(path)
      .then(bytes => {
        const u8 = new Uint8Array(bytes);
        const blob = new Blob([u8], { type: 'image/png' });
        const url = URL.createObjectURL(blob);
        setDataUrl(url);

        // Read PNG dimensions from IHDR chunk (bytes 16–23)
        const w = (u8[16] << 24 | u8[17] << 16 | u8[18] << 8 | u8[19]) >>> 0;
        const h = (u8[20] << 24 | u8[21] << 16 | u8[22] << 8 | u8[23]) >>> 0;
        const sizeKb = Math.round(bytes.length / 1024);
        const gcd = (a: number, b: number): number => b === 0 ? a : gcd(b, a % b);
        const g = gcd(w, h);
        const aspectRatio = `${w / g}:${h / g}`;

        onInfo({ width: w, height: h, aspectRatio, sizeKb, dataUrl: url });
        setLoading(false);
        return () => URL.revokeObjectURL(url);
      })
      .catch(e => {
        setError(String(e));
        setLoading(false);
      });
  }, [path]);

  if (loading) return <div className="asset-viewport-placeholder">Loading…</div>;
  if (error)   return <div className="asset-viewport-placeholder asset-error">{error}</div>;

  return (
    <div className="asset-viewport-png">
      <img
        src={dataUrl!}
        className="asset-png-img"
        alt="asset preview"
        draggable={false}
      />
    </div>
  );
}
