'use client';
import { setThemePref, useThemePref } from '@/lib/theme';
import type { ThemePref } from '@/lib/theme';

const ORDER: ThemePref[] = ['system', 'light', 'dark'];
const LABEL: Record<ThemePref, string> = { system: 'Sistem', light: 'Açık', dark: 'Koyu' };

function Icon({ pref }: { pref: ThemePref }) {
  const common = { width: 18, height: 18, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 2, strokeLinecap: 'round' as const, strokeLinejoin: 'round' as const, 'aria-hidden': true };
  if (pref === 'light')
    return (
      <svg {...common}>
        <circle cx="12" cy="12" r="4" />
        <path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4" />
      </svg>
    );
  if (pref === 'dark')
    return (
      <svg {...common}>
        <path d="M21 12.8A9 9 0 1111.2 3a7 7 0 009.8 9.8z" />
      </svg>
    );
  return (
    <svg {...common}>
      <rect x="3" y="4" width="18" height="12" rx="2" />
      <path d="M8 20h8M12 16v4" />
    </svg>
  );
}

export function ThemeToggle() {
  const pref = useThemePref();
  const next = ORDER[(ORDER.indexOf(pref) + 1) % ORDER.length];
  return (
    <button
      type="button"
      className="icon-btn"
      onClick={() => setThemePref(next)}
      title={`Tema: ${LABEL[pref]} (değiştirmek için tıkla: ${LABEL[next]})`}
      aria-label={`Tema: ${LABEL[pref]}`}
    >
      <Icon pref={pref} />
      <span className="icon-btn__label">{LABEL[pref]}</span>
    </button>
  );
}
