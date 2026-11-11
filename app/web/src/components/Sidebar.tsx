'use client';
import Link from 'next/link';
import { useEffect, useRef, useState } from 'react';
import { useToc } from './TocContext';
import { Title } from './Title';
import { CheckIcon, ScoreBadge } from './ScoreBadge';
import type { TocChapter } from '@/lib/types';

export function currentRoute(pathname: string): { kind: 'ders' | 'egzersiz' | null; pageId: string | null } {
  const m = /^\/(ders|egzersiz)\/(.+?)\/?$/.exec(pathname);
  if (!m) return { kind: null, pageId: null };
  return { kind: m[1] as 'ders' | 'egzersiz', pageId: m[2] };
}

export function Sidebar({ pathname, onNavigate }: { pathname: string; onNavigate: () => void }) {
  const { toc, error, refresh } = useToc();
  const { kind, pageId } = currentRoute(pathname);
  const currentChapter = pageId ? pageId.split('/')[0] : null;
  const [overrides, setOverrides] = useState<Record<string, boolean>>({});
  const activeRef = useRef<HTMLLIElement | null>(null);

  const tocLoaded = toc !== null;
  useEffect(() => {
    activeRef.current?.scrollIntoView({ block: 'nearest' });
  }, [pageId, tocLoaded]);

  const isOpen = (c: TocChapter) => overrides[c.id] ?? c.id === currentChapter;
  const pct = toc && toc.progress.total > 0 ? Math.round((toc.progress.completed / toc.progress.total) * 100) : 0;

  return (
    <nav className="sidebar__inner" aria-label="İçindekiler">
      <div className="sidebar__progress">
        {toc ? (
          <>
            <div className="sidebar__progress-text">
              <span>
                <strong>{toc.progress.completed}</strong> / {toc.progress.total} egzersiz
              </span>
              <span className="muted">{toc.progress.score_sum} puan</span>
            </div>
            <div className="progressbar" role="progressbar" aria-valuenow={pct} aria-valuemin={0} aria-valuemax={100} aria-label="Genel ilerleme">
              <div className="progressbar__fill" style={{ width: `${pct}%` }} />
            </div>
          </>
        ) : (
          <div className="muted small">{error ? 'İçindekiler yüklenemedi' : 'Yükleniyor…'}</div>
        )}
      </div>
      {error && !toc && (
        <div className="sidebar__error">
          <p>{error}</p>
          <button type="button" className="btn btn--small" onClick={() => void refresh()}>
            Tekrar dene
          </button>
        </div>
      )}
      <div className="sidebar__list">
        {toc?.chapters.map((c) => {
          const open = isOpen(c);
          const done = c.pages.filter((p) => p.exercise?.completed).length;
          const total = c.pages.filter((p) => p.exercise).length;
          return (
            <section key={c.id} className="chapter">
              <button
                type="button"
                className="chapter__head"
                aria-expanded={open}
                onClick={() => setOverrides((o) => ({ ...o, [c.id]: !open }))}
              >
                <svg className={`chapter__chev${open ? ' chapter__chev--open' : ''}`} width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
                  <path d="M4 2l4 4-4 4" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
                <span className="chapter__title">
                  <Title text={c.title} />
                </span>
                {total > 0 && (
                  <span className="chapter__count" title="Tamamlanan egzersizler">
                    {done}/{total}
                  </span>
                )}
              </button>
              {open && (
                <ul className="chapter__pages">
                  {c.pages.map((p) => {
                    const active = p.id === pageId;
                    const onLesson = active && kind === 'ders';
                    const onExercise = active && kind === 'egzersiz';
                    const ex = p.exercise;
                    return (
                      <li key={p.id} ref={active ? activeRef : undefined}>
                        <div className={`item${onLesson ? ' item--active' : ''}${onExercise ? ' item--parent' : ''}`}>
                          <Link
                            href={`/ders/${p.id}/`}
                            className="item__link"
                            aria-current={onLesson ? 'page' : undefined}
                            onClick={onNavigate}
                          >
                            <span className="item__title">
                              <Title text={p.title} />
                            </span>
                            {!p.translated && (
                              <span className="tag" title="Bu sayfa henüz Türkçeye çevrilmedi">
                                EN
                              </span>
                            )}
                          </Link>
                          {ex && !active && (
                            <Link
                              href={`/egzersiz/${p.id}/`}
                              className={`item__ex${ex.completed ? ' item__ex--done' : ''}`}
                              title={ex.completed ? `Egzersize git (tamamlandı, ${ex.score} puan)` : 'Egzersize git'}
                              aria-label={ex.completed ? `Egzersiz: tamamlandı, ${ex.score} puan` : 'Egzersiz'}
                              onClick={onNavigate}
                            >
                              {ex.completed ? (
                                <>
                                  <CheckIcon />
                                  <ScoreBadge score={ex.score} />
                                </>
                              ) : (
                                <>
                                  <span className="item__ex-icon" aria-hidden="true">
                                    {'</>'}
                                  </span>
                                  <span className="item__ex-label">Egzersiz</span>
                                </>
                              )}
                            </Link>
                          )}
                        </div>
                        {ex && active && (
                          <div className="item__sub" role="group" aria-label="Bu sayfanın bölümleri">
                            <Link
                              href={`/ders/${p.id}/`}
                              className={`item__subtab${onLesson ? ' item__subtab--active' : ''}`}
                              aria-current={onLesson ? 'page' : undefined}
                              onClick={onNavigate}
                            >
                              Anlatım
                            </Link>
                            <Link
                              href={`/egzersiz/${p.id}/`}
                              className={`item__subtab${onExercise ? ' item__subtab--active' : ''}${ex.completed ? ' item__subtab--done' : ''}`}
                              aria-current={onExercise ? 'page' : undefined}
                              onClick={onNavigate}
                            >
                              <span aria-hidden="true">{'</>'}</span> Egzersiz
                              {ex.completed && (
                                <>
                                  <CheckIcon />
                                  <ScoreBadge score={ex.score} />
                                </>
                              )}
                            </Link>
                          </div>
                        )}
                      </li>
                    );
                  })}
                </ul>
              )}
            </section>
          );
        })}
      </div>
    </nav>
  );
}
