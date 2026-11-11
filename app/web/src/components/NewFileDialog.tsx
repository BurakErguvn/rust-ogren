'use client';
import { useEffect, useRef, useState } from 'react';

/** Kullanıcının yazdığı yolu `src/…​.rs` biçimine getirir; geçersizse Türkçe hata döner. */
export function normalizeNewFilePath(raw: string, existing: readonly string[]): { path: string } | { error: string } {
  let p = raw.trim().replaceAll('\\', '/').replace(/^\.\//, '');
  if (!p) return { error: 'Bir dosya adı yaz.' };
  if (p.startsWith('/')) return { error: 'Yol göreli olmalı (ör. lib.rs veya src/utils.rs).' };
  if (p.split('/').includes('..')) return { error: 'Yolda “..” kullanılamaz.' };
  if (!p.startsWith('src/')) p = `src/${p}`;
  if (!p.endsWith('.rs')) return { error: 'Dosya adı .rs ile bitmeli.' };
  const segments = p.split('/');
  if (segments.some((s) => s === '' || s === '.')) return { error: 'Geçersiz dosya yolu.' };
  if (!segments.every((s) => /^[A-Za-z0-9_.-]+$/.test(s))) return { error: 'Yalnızca harf, rakam, “_”, “-” ve “.” kullanabilirsin.' };
  if (segments[segments.length - 1] === '.rs') return { error: 'Dosya adı boş olamaz.' };
  if (existing.includes(p)) return { error: `${p} zaten var.` };
  return { path: p };
}

interface Props {
  open: boolean;
  existing: readonly string[];
  busy: boolean;
  error: string | null;
  onCreate: (path: string) => void;
  onCancel: () => void;
}

export function NewFileDialog({ open, existing, busy, error, onCreate, onCancel }: Props) {
  const ref = useRef<HTMLDialogElement>(null);
  const [value, setValue] = useState('');
  const [localError, setLocalError] = useState<string | null>(null);

  useEffect(() => {
    const d = ref.current;
    if (!d) return;
    if (open && !d.open) {
      // Diyalog her açılışta temiz başlar.
      setValue('');
      setLocalError(null);
      d.showModal();
    }
    if (!open && d.open) d.close();
  }, [open]);

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const r = normalizeNewFilePath(value, existing);
    if ('error' in r) setLocalError(r.error);
    else {
      setLocalError(null);
      onCreate(r.path);
    }
  };

  const shown = localError ?? error;
  return (
    <dialog
      ref={ref}
      className="dialog"
      onCancel={(e) => {
        e.preventDefault();
        onCancel();
      }}
      onClick={(e) => {
        if (e.target === ref.current) onCancel();
      }}
    >
      <form className="dialog__body" onSubmit={submit}>
        <h2>Yeni dosya</h2>
        <p>
          Dosya <code>src/</code> altında oluşturulur (ör. <code>lib.rs</code> veya <code>utils.rs</code>).
        </p>
        <input
          className="dialog__input"
          type="text"
          value={value}
          onChange={(e) => {
            setValue(e.target.value);
            setLocalError(null);
          }}
          placeholder="src/lib.rs"
          aria-label="Dosya yolu"
          aria-invalid={shown ? true : undefined}
          spellCheck={false}
          autoComplete="off"
          autoFocus
        />
        {shown && (
          <p className="dialog__error" role="alert">
            {shown}
          </p>
        )}
        <div className="dialog__actions">
          <button type="button" className="btn" onClick={onCancel}>
            Vazgeç
          </button>
          <button type="submit" className="btn btn--primary" disabled={busy}>
            {busy ? 'Oluşturuluyor…' : 'Oluştur'}
          </button>
        </div>
      </form>
    </dialog>
  );
}
