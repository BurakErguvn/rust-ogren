import type { RunResult } from './types';

// "Kaldığın yerden devam" için son ziyaret edilen sayfa.
export interface LastVisit {
  kind: 'ders' | 'egzersiz';
  id: string;
}

const KEY = 'rust-ogren:last';

export function saveLastVisit(v: LastVisit): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(v));
  } catch {
    /* yok say */
  }
}

export function loadLastVisit(): LastVisit | null {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return null;
    const v = JSON.parse(raw) as Partial<LastVisit>;
    if ((v.kind === 'ders' || v.kind === 'egzersiz') && typeof v.id === 'string') {
      return { kind: v.kind, id: v.id };
    }
  } catch {
    /* bozuk kayıt */
  }
  return null;
}

export function loadBool(key: string, fallback: boolean): boolean {
  try {
    const v = localStorage.getItem(key);
    if (v === '1') return true;
    if (v === '0') return false;
  } catch {
    /* yok say */
  }
  return fallback;
}

export function saveBool(key: string, value: boolean): void {
  try {
    localStorage.setItem(key, value ? '1' : '0');
  } catch {
    /* yok say */
  }
}

// ---- Son çalıştırma sonucu (egzersiz başına) ----------------------------------------------------

export interface StoredResult {
  result: RunResult;
  savedAt: number;
}

const resultKey = (exerciseId: string) => `rust-ogren:result:${exerciseId}`;

export function saveResult(exerciseId: string, result: RunResult): void {
  try {
    const v: StoredResult = { result, savedAt: Date.now() };
    localStorage.setItem(resultKey(exerciseId), JSON.stringify(v));
  } catch {
    /* kota dolu / erişim yok: yok say */
  }
}

export function loadResult(exerciseId: string): StoredResult | null {
  try {
    const raw = localStorage.getItem(resultKey(exerciseId));
    if (!raw) return null;
    const v = JSON.parse(raw) as Partial<StoredResult>;
    const r = v.result;
    if (r && typeof v.savedAt === 'number' && Array.isArray(r.tests) && Array.isArray(r.diagnostics) && typeof r.status === 'string') {
      return { result: r, savedAt: v.savedAt };
    }
  } catch {
    /* bozuk kayıt */
  }
  return null;
}

export function clearResult(exerciseId: string): void {
  try {
    localStorage.removeItem(resultKey(exerciseId));
  } catch {
    /* yok say */
  }
}

// ---- Editör / panel oranı ---------------------------------------------------------------------

const PANEL_KEY = 'rust-ogren:panel-ratio';

export function loadPanelRatio(fallback: number): number {
  try {
    const v = Number(localStorage.getItem(PANEL_KEY));
    if (Number.isFinite(v) && v > 0.05 && v < 0.9) return v;
  } catch {
    /* yok say */
  }
  return fallback;
}

export function savePanelRatio(ratio: number): void {
  try {
    localStorage.setItem(PANEL_KEY, String(ratio));
  } catch {
    /* yok say */
  }
}

// ---- Öğrencinin oluşturduğu dosyalar (silinebilir olanlar) ------------------------------------

const createdKey = (exerciseId: string) => `rust-ogren:created:${exerciseId}`;

export function loadCreatedFiles(exerciseId: string): string[] {
  try {
    const v: unknown = JSON.parse(localStorage.getItem(createdKey(exerciseId)) ?? '[]');
    return Array.isArray(v) ? v.filter((x): x is string => typeof x === 'string') : [];
  } catch {
    return [];
  }
}

export function saveCreatedFiles(exerciseId: string, paths: readonly string[]): void {
  try {
    if (paths.length === 0) localStorage.removeItem(createdKey(exerciseId));
    else localStorage.setItem(createdKey(exerciseId), JSON.stringify(paths));
  } catch {
    /* yok say */
  }
}
