import type { ExerciseFiles, PageData, RunResult, Toc } from './types';

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  let res: Response;
  try {
    res = await fetch(path, init);
  } catch {
    throw new ApiError(0, 'Sunucuya ulaşılamıyor. Uygulama sunucusunun çalıştığından emin ol.');
  }
  if (!res.ok) {
    let message = `Sunucu hatası (${res.status})`;
    try {
      const body = (await res.json()) as { error?: string };
      if (body.error) message = body.error;
    } catch {
      /* gövde JSON değil */
    }
    throw new ApiError(res.status, message);
  }
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

/** Yol parçalarını tek tek kodlar, `/` ayraçlarını korur. */
function encodePath(p: string): string {
  return p.split('/').map(encodeURIComponent).join('/');
}

export const api = {
  toc: () => request<Toc>('/api/toc'),
  page: (id: string) => request<PageData>(`/api/pages/${encodePath(id)}`),
  files: (id: string) => request<ExerciseFiles>(`/api/exercises/${encodePath(id)}/files`),
  saveFile: (id: string, path: string, content: string) =>
    request<void>(`/api/exercises/${encodePath(id)}/files/${encodePath(path)}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ content }),
    }),
  deleteFile: (id: string, path: string) =>
    request<void>(`/api/exercises/${encodePath(id)}/files/${encodePath(path)}`, { method: 'DELETE' }),
  run: (id: string, kind: 'test' | 'submit') =>
    request<RunResult>(`/api/exercises/${encodePath(id)}/${kind}`, { method: 'POST' }),
  reset: (id: string) =>
    request<ExerciseFiles>(`/api/exercises/${encodePath(id)}/reset`, { method: 'POST' }),
};

/** rust-analyzer WebSocket adresi. */
export function lspUrl(exerciseId: string): string {
  const explicit = process.env.NEXT_PUBLIC_API_ORIGIN;
  let origin: string;
  if (explicit) {
    origin = explicit.replace(/^http/, 'ws');
  } else if (process.env.NODE_ENV === 'development') {
    origin = 'ws://localhost:3000';
  } else {
    origin = window.location.origin.replace(/^http/, 'ws');
  }
  return `${origin.replace(/\/$/, '')}/api/lsp/${encodePath(exerciseId)}`;
}
