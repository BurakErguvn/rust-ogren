'use client';
import { useCallback, useSyncExternalStore } from 'react';
import { loadBool, saveBool } from './storage';

const EVENT = 'rust-ogren:local-bool';

/** localStorage'da saklanan boolean; SSR/hydration ile uyumlu. */
export function useLocalBool(key: string, fallback: boolean): [boolean, (v: boolean) => void] {
  const subscribe = useCallback((cb: () => void) => {
    window.addEventListener(EVENT, cb);
    return () => window.removeEventListener(EVENT, cb);
  }, []);
  const value = useSyncExternalStore(
    subscribe,
    () => loadBool(key, fallback),
    () => fallback,
  );
  const set = useCallback(
    (v: boolean) => {
      saveBool(key, v);
      window.dispatchEvent(new Event(EVENT));
    },
    [key],
  );
  return [value, set];
}
