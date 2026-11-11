'use client';
import { useSyncExternalStore } from 'react';

export type ThemePref = 'system' | 'light' | 'dark';
export type ResolvedTheme = 'light' | 'dark';

const KEY = 'rust-ogren:theme';
const EVENT = 'rust-ogren:theme-change';

export function getThemePref(): ThemePref {
  try {
    const v = localStorage.getItem(KEY);
    if (v === 'light' || v === 'dark') return v;
  } catch {
    /* localStorage yok */
  }
  return 'system';
}

export function resolveTheme(pref: ThemePref): ResolvedTheme {
  if (pref === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  return pref;
}

export function applyTheme(pref: ThemePref): void {
  const root = document.documentElement;
  root.dataset.theme = resolveTheme(pref);
  root.dataset.themePref = pref;
}

export function setThemePref(pref: ThemePref): void {
  try {
    if (pref === 'system') localStorage.removeItem(KEY);
    else localStorage.setItem(KEY, pref);
  } catch {
    /* yok say */
  }
  applyTheme(pref);
  window.dispatchEvent(new Event(EVENT));
}

function subscribe(cb: () => void): () => void {
  const mq = window.matchMedia('(prefers-color-scheme: dark)');
  const onSystem = () => {
    if (getThemePref() === 'system') applyTheme('system');
    cb();
  };
  mq.addEventListener('change', onSystem);
  window.addEventListener(EVENT, cb);
  return () => {
    mq.removeEventListener('change', onSystem);
    window.removeEventListener(EVENT, cb);
  };
}

export function useResolvedTheme(): ResolvedTheme {
  return useSyncExternalStore(
    subscribe,
    () => (document.documentElement.dataset.theme === 'dark' ? 'dark' : 'light'),
    () => 'light',
  );
}

export function useThemePref(): ThemePref {
  return useSyncExternalStore(subscribe, getThemePref, () => 'system' as ThemePref);
}

/** <head> içinde, boyamadan önce çalışan satır içi betik (tema titremesini önler). */
export const themeInitScript = `(function(){try{var p=localStorage.getItem('${KEY}');var d=p==='dark'||(p!=='light'&&window.matchMedia('(prefers-color-scheme: dark)').matches);var r=document.documentElement;r.dataset.theme=d?'dark':'light';r.dataset.themePref=p==='dark'||p==='light'?p:'system';}catch(e){}})();`;
