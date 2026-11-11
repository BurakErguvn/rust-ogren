'use client';
import { useCallback, useEffect, useId, useRef, useState } from 'react';

/**
 * Puanlama kuralları için "?" düğmesi. Üzerine gelince (fare), odaklanınca (klavye) veya
 * dokununca (dokunmatik) açılır; Esc veya dışarı tıklama kapatır.
 */
export function ScoreInfo() {
  const id = useId();
  const [hover, setHover] = useState(false);
  const [focus, setFocus] = useState(false);
  const [pinned, setPinned] = useState(false);
  const wrapRef = useRef<HTMLSpanElement>(null);
  const open = hover || focus || pinned;

  const close = useCallback(() => {
    setHover(false);
    setFocus(false);
    setPinned(false);
  }, []);

  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') close();
    };
    const onDown = (e: PointerEvent) => {
      if (!wrapRef.current?.contains(e.target as Node)) close();
    };
    window.addEventListener('keydown', onKey);
    window.addEventListener('pointerdown', onDown);
    return () => {
      window.removeEventListener('keydown', onKey);
      window.removeEventListener('pointerdown', onDown);
    };
  }, [open, close]);

  return (
    <span
      className="scoreinfo"
      ref={wrapRef}
      onPointerEnter={(e) => e.pointerType === 'mouse' && setHover(true)}
      onPointerLeave={(e) => e.pointerType === 'mouse' && setHover(false)}
    >
      <button
        type="button"
        className="scoreinfo__btn"
        aria-label="Puanlama nasıl çalışır?"
        aria-expanded={open}
        aria-controls={id}
        onFocus={() => setFocus(true)}
        onBlur={() => setFocus(false)}
        onClick={() => setPinned((p) => !p)}
      >
        ?
      </button>
      <div id={id} role="tooltip" className="scoreinfo__pop" hidden={!open}>
        <strong className="scoreinfo__title">Puanlama</strong>
        <ul>
          <li>
            Yalnızca <strong>Teslim et</strong> puana işlenir.
          </li>
          <li>
            Puan = <code>max(100 − 10 × başarısız teslim, 50)</code>.
          </li>
          <li>
            <strong>Test et</strong> ücretsizdir; puanı etkilemez.
          </li>
          <li>Egzersiz tamamlanınca puanı sabitlenir; sonraki teslimler değiştirmez.</li>
          <li>
            <strong>Sıfırla</strong> puanı değiştirmez.
          </li>
        </ul>
      </div>
    </span>
  );
}
