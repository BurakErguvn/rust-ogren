'use client';
import Link from 'next/link';
import { useEffect, useState } from 'react';
import { api } from '@/lib/api';
import { saveLastVisit } from '@/lib/storage';
import type { PageData } from '@/lib/types';
import { Markdown } from './Markdown';
import { useToc } from './TocContext';
import { Title } from './Title';
import { CheckIcon, ScoreBadge } from './ScoreBadge';

type State = { id: string; page: PageData } | { id: string; error: string } | null;

export function LessonView({ pageId }: { pageId: string }) {
  const [state, setState] = useState<State>(null);
  const { pages } = useToc();

  useEffect(() => {
    let cancelled = false;
    saveLastVisit({ kind: 'ders', id: pageId });
    api
      .page(pageId)
      .then((page) => !cancelled && setState({ id: pageId, page }))
      .catch((e: unknown) => !cancelled && setState({ id: pageId, error: e instanceof Error ? e.message : 'Sayfa yüklenemedi.' }));
    return () => {
      cancelled = true;
    };
  }, [pageId]);

  const loaded = state && state.id === pageId ? state : null;
  const page = loaded && 'page' in loaded ? loaded.page : null;
  const error = loaded && 'error' in loaded ? loaded.error : null;

  useEffect(() => {
    if (page) document.title = `${page.title.replaceAll('`', '')} · Rust Öğren`;
  }, [page]);

  // Sayfa yüklendikten sonra URL'deki #slug'a kaydır, değilse en üste çık.
  useEffect(() => {
    if (!page) return;
    const hash = decodeURIComponent(window.location.hash.slice(1));
    const el = hash ? document.getElementById(hash) : null;
    if (el) el.scrollIntoView();
    else document.getElementById('main')?.scrollTo({ top: 0 });
  }, [page]);

  if (error) {
    return (
      <article className="lesson">
        <div className="notice notice--error" role="alert">
          <strong>Sayfa yüklenemedi.</strong>
          <p>{error}</p>
          <button type="button" className="btn" onClick={() => window.location.reload()}>
            Yeniden dene
          </button>
        </div>
      </article>
    );
  }
  if (!page) {
    return (
      <article className="lesson" aria-busy="true">
        <div className="skeleton skeleton--title" />
        <div className="skeleton" />
        <div className="skeleton" />
        <div className="skeleton skeleton--short" />
      </article>
    );
  }

  const prev = page.prev ? pages.get(page.prev) : null;
  const next = page.next ? pages.get(page.next) : null;
  const ex = pages.get(pageId)?.exercise ?? page.exercise;

  return (
    <article className="lesson">
      {!page.translated && (
        <div className="notice notice--info">Bu sayfa henüz Türkçeye çevrilmedi; İngilizce özgün metin gösteriliyor.</div>
      )}
      <Markdown source={page.markdown} pageId={page.id} notes={page.notes ?? []} />

      {page.exercise && (
        <aside className="exercise-cta">
          <div>
            <h2 className="exercise-cta__title">Sıra sende</h2>
            <p className="muted">
              {ex?.completed ? (
                <>
                  Bu bölümün egzersizini tamamladın <CheckIcon className="inline-check" /> <ScoreBadge score={ex.score} />
                </>
              ) : (
                'Öğrendiklerini küçük bir egzersizle pekiştir.'
              )}
            </p>
          </div>
          <Link href={`/egzersiz/${page.id}/`} className="btn btn--primary btn--lg">
            Egzersize geç →
          </Link>
        </aside>
      )}

      <nav className="pager" aria-label="Sayfa gezintisi">
        {page.prev ? (
          <Link href={`/ders/${page.prev}/`} className="pager__link pager__link--prev" rel="prev">
            <span className="pager__dir">← Önceki</span>
            <span className="pager__title">{prev ? <Title text={prev.title} /> : page.prev}</span>
          </Link>
        ) : (
          <span />
        )}
        {page.next ? (
          <Link href={`/ders/${page.next}/`} className="pager__link pager__link--next" rel="next">
            <span className="pager__dir">Sonraki →</span>
            <span className="pager__title">{next ? <Title text={next.title} /> : page.next}</span>
          </Link>
        ) : (
          <span />
        )}
      </nav>
    </article>
  );
}
