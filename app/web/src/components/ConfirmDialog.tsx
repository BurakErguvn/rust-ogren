'use client';
import { useEffect, useRef } from 'react';
import type { ReactNode } from 'react';

interface Props {
  open: boolean;
  title: string;
  children: ReactNode;
  confirmLabel: string;
  cancelLabel?: string;
  danger?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({ open, title, children, confirmLabel, cancelLabel = 'Vazgeç', danger, onConfirm, onCancel }: Props) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const d = ref.current;
    if (!d) return;
    if (open && !d.open) d.showModal();
    if (!open && d.open) d.close();
  }, [open]);
  return (
    <dialog ref={ref} className="dialog" onCancel={(e) => { e.preventDefault(); onCancel(); }} onClick={(e) => { if (e.target === ref.current) onCancel(); }}>
      <div className="dialog__body">
        <h2>{title}</h2>
        <div>{children}</div>
        <div className="dialog__actions">
          <button type="button" className="btn" onClick={onCancel} autoFocus>
            {cancelLabel}
          </button>
          <button type="button" className={`btn ${danger ? 'btn--danger' : 'btn--primary'}`} onClick={onConfirm}>
            {confirmLabel}
          </button>
        </div>
      </div>
    </dialog>
  );
}
