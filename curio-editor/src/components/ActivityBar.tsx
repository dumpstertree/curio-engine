import React, { useState } from 'react';

type PanelId = 'hierarchy' | 'search' | 'settings';

interface Props {
  active: PanelId;
  onChange: (id: PanelId) => void;
}

const icons: { id: PanelId; label: string; svg: React.ReactNode }[] = [
  {
    id: 'hierarchy',
    label: 'Scene Explorer',
    svg: (
      <svg width="22" height="22" viewBox="0 0 22 22" fill="none" stroke="currentColor" strokeWidth="1.5">
        <rect x="3" y="3" width="5" height="5" rx="1" />
        <rect x="3" y="14" width="5" height="5" rx="1" />
        <rect x="14" y="8" width="5" height="5" rx="1" />
        <line x1="8" y1="5.5" x2="14" y2="10.5" />
        <line x1="8" y1="16.5" x2="14" y2="12.5" />
      </svg>
    ),
  },
  {
    id: 'search',
    label: 'Search',
    svg: (
      <svg width="22" height="22" viewBox="0 0 22 22" fill="none" stroke="currentColor" strokeWidth="1.5">
        <circle cx="10" cy="10" r="6" />
        <line x1="14.5" y1="14.5" x2="19" y2="19" />
      </svg>
    ),
  },
];

const bottomIcons: { id: PanelId; label: string; svg: React.ReactNode }[] = [
  {
    id: 'settings',
    label: 'Settings',
    svg: (
      <svg width="22" height="22" viewBox="0 0 22 22" fill="none" stroke="currentColor" strokeWidth="1.5">
        <circle cx="11" cy="11" r="3" />
        <path d="M11 2v2M11 18v2M2 11h2M18 11h2M4.2 4.2l1.4 1.4M16.4 16.4l1.4 1.4M4.2 17.8l1.4-1.4M16.4 5.6l1.4-1.4" />
      </svg>
    ),
  },
];

export function ActivityBar({ active, onChange }: Props) {
  return (
    <div className="activity-bar">
      {icons.map(({ id, label, svg }) => (
        <div
          key={id}
          className={`activity-icon ${active === id ? 'active' : ''}`}
          onClick={() => onChange(id)}
          data-tooltip={label}
          title={label}
        >
          {svg}
        </div>
      ))}
      <div className="activity-bottom">
        {bottomIcons.map(({ id, label, svg }) => (
          <div
            key={id}
            className={`activity-icon ${active === id ? 'active' : ''}`}
            onClick={() => onChange(id)}
            title={label}
          >
            {svg}
          </div>
        ))}
      </div>
    </div>
  );
}
