'use client';
import { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react';
import type { ReactNode } from 'react';
import { api } from '@/lib/api';
import type { Toc, TocPage } from '@/lib/types';

interface TocState {
  toc: Toc | null;
  error: string | null;
  refresh: () => Promise<void>;
  pages: Map<string, TocPage>;
}

const Ctx = createContext<TocState | null>(null);

export function TocProvider({ children }: { children: ReactNode }) {
  const [toc, setToc] = useState<Toc | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const t = await api.toc();
      setToc(t);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'İçindekiler yüklenemedi.');
    }
  }, []);

  useEffect(() => {
    // İlk yükleme: dış sistemden (API) veri çekip state'e yazar.
    // eslint-disable-next-line react-hooks/set-state-in-effect
    void refresh();
  }, [refresh]);

  const pages = useMemo(() => {
    const m = new Map<string, TocPage>();
    toc?.chapters.forEach((c) => c.pages.forEach((p) => m.set(p.id, p)));
    return m;
  }, [toc]);

  const value = useMemo(() => ({ toc, error, refresh, pages }), [toc, error, refresh, pages]);
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useToc(): TocState {
  const v = useContext(Ctx);
  if (!v) throw new Error('useToc, TocProvider içinde kullanılmalı');
  return v;
}
