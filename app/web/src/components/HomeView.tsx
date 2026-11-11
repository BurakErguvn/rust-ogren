'use client';
import Link from 'next/link';
import { useEffect, useState } from 'react';
import { loadLastVisit } from '@/lib/storage';
import type { LastVisit } from '@/lib/storage';
import { useToc } from './TocContext';
import { Title } from './Title';
import { CheckIcon, ScoreBadge } from './ScoreBadge';

export function HomeView() {
  const { toc, error, refresh, pages } = useToc();
  const [last, setLast] = useState<LastVisit | null>(null);

  useEffect(() => {
    // localStorage yalnızca istemcide okunabilir.
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setLast(loadLastVisit());
  }, []);

  // Kayıtlı ziyaret yalnızca içindekilerde varsa (egzersiz ise hâlâ egzersizi varsa) geçerlidir.
  const lastPage = last ? pages.get(last.id) : undefined;
  const lastValid = last && lastPage && (last.kind === 'ders' || lastPage.exercise !== null) ? { last, page: lastPage } : null;
  const allPages = toc?.chapters.flatMap((c) => c.pages) ?? [];
  const firstId = allPages[0]?.id;
  const nextUp = toc && toc.progress.completed > 0 ? allPages.find((p) => p.exercise && !p.exercise.completed) : undefined;
  const pct = toc && toc.progress.total > 0 ? Math.round((toc.progress.completed / toc.progress.total) * 100) : 0;

  return (
    <div className="home">
      <section className="home__hero">
        <h1>Rust Öğren</h1>
        <p className="home__lead">
          Önce konunun anlatımını oku, sonra aynı konunun egzersizini tarayıcıdaki küçük Rust IDE&apos;sinde çöz. Testleri
          istediğin kadar çalıştır; sonunda teslim et ve puanını al.
        </p>
        <div className="home__actions">
          {lastValid ? (
            <Link href={`/${lastValid.last.kind}/${lastValid.last.id}/`} className="btn btn--primary btn--lg">
              Kaldığın yerden devam et
              <span className="btn__sub">
                {lastValid.last.kind === 'egzersiz' ? 'Egzersiz: ' : ''}
                <Title text={lastValid.page.title} />
              </span>
            </Link>
          ) : nextUp ? (
            <Link href={`/ders/${nextUp.id}/`} className="btn btn--primary btn--lg">
              Devam et
              <span className="btn__sub">
                <Title text={nextUp.title} />
              </span>
            </Link>
          ) : firstId ? (
            <Link href={`/ders/${firstId}/`} className="btn btn--primary btn--lg">
              Kursa başla
            </Link>
          ) : null}
        </div>
        {toc && (
          <div className="home__progress">
            <div className="sidebar__progress-text">
              <span>
                <strong>{toc.progress.completed}</strong> / {toc.progress.total} egzersiz tamamlandı
              </span>
              <span className="muted">{toc.progress.score_sum} puan</span>
            </div>
            <div className="progressbar" role="progressbar" aria-valuenow={pct} aria-valuemin={0} aria-valuemax={100}>
              <div className="progressbar__fill" style={{ width: `${pct}%` }} />
            </div>
          </div>
        )}
      </section>

      {error && !toc && (
        <div className="notice notice--error" role="alert">
          <strong>İçindekiler yüklenemedi.</strong>
          <p>{error}</p>
          <button type="button" className="btn" onClick={() => void refresh()}>
            Tekrar dene
          </button>
        </div>
      )}
      {!toc && !error && <p className="muted">Yükleniyor…</p>}

      <div className="home__toc">
        {toc?.chapters.map((c, i) => (
          <section key={c.id} className="home__chapter">
            <h2>
              <span className="home__chapter-no">{i + 1}</span>
              <Title text={c.title} />
            </h2>
            <ol className="home__pages">
              {c.pages.map((p) => (
                <li key={p.id}>
                  <Link href={`/ders/${p.id}/`} className="home__page">
                    <Title text={p.title} />
                    {!p.translated && <span className="tag">EN</span>}
                  </Link>
                  {p.exercise && (
                    <Link href={`/egzersiz/${p.id}/`} className="home__ex" title="Egzersiz">
                      {p.exercise.completed ? (
                        <>
                          <CheckIcon />
                          <ScoreBadge score={p.exercise.score} />
                        </>
                      ) : (
                        <span>{'</>'}</span>
                      )}
                    </Link>
                  )}
                </li>
              ))}
            </ol>
          </section>
        ))}
      </div>
    </div>
  );
}
