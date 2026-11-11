'use client';
import type { CompilerDiagnostic, RunResult } from '@/lib/types';

/** Süre: derleme dahil toplam. */
export function formatDuration(ms: number): string {
  if (ms < 100) return '< 0.1 sn';
  if (ms < 60_000) return `${(ms / 1000).toFixed(1)} sn`;
  const total = Math.round(ms / 1000);
  return `${Math.floor(total / 60)} dk ${total % 60} sn`;
}

function formatWhen(ts: number): string {
  return new Date(ts).toLocaleString('tr-TR', { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' });
}

export type PanelTab = 'tests' | 'diagnostics' | 'log';

const STATUS_LABEL: Record<RunResult['status'], string> = {
  passed: 'Geçti',
  failed: 'Başarısız',
  compile_error: 'Derleme hatası',
  timeout: 'Zaman aşımı',
};

interface Props {
  /** Sonuçtan sonra dosya eklendi/silindi: sonuç eski. Değer, değişikliğin açıklaması. */
  stale: string | null;
  result: RunResult | null;
  /** Sonuç önceki bir oturumdan geri yüklendi (bu çalıştırma değil). */
  restoredAt: number | null;
  running: 'test' | 'submit' | null;
  notice: { kind: 'error' | 'info'; text: string } | null;
  summary: string | null;
  tab: PanelTab;
  onTab: (t: PanelTab) => void;
  onJump: (d: CompilerDiagnostic & { line: number; column: number }) => void;
}

function SeverityIcon({ severity }: { severity: string }) {
  const cls = severity === 'error' ? 'sev sev--error' : severity === 'warning' ? 'sev sev--warning' : 'sev sev--info';
  return <span className={cls} aria-label={severity} title={severity} />;
}

export function ResultsPanel({ stale, result, restoredAt, running, notice, summary, tab, onTab, onJump }: Props) {
  const diagCount = result?.diagnostics.length ?? 0;
  const errCount = result?.diagnostics.filter((d) => d.severity === 'error').length ?? 0;

  return (
    <div className={`panel${stale && result ? ' panel--stale' : ''}`}>
      <div className="panel__tabs" role="tablist">
        <button type="button" role="tab" aria-selected={tab === 'tests'} className={`ptab${tab === 'tests' ? ' ptab--active' : ''}`} onClick={() => onTab('tests')}>
          Test sonuçları
          {result && result.tests_total > 0 && (
            <span className={`ptab__count${result.tests_passed === result.tests_total ? ' ptab__count--ok' : ' ptab__count--bad'}`}>
              {result.tests_passed}/{result.tests_total}
            </span>
          )}
        </button>
        <button type="button" role="tab" aria-selected={tab === 'diagnostics'} className={`ptab${tab === 'diagnostics' ? ' ptab--active' : ''}`} onClick={() => onTab('diagnostics')}>
          Derleyici
          {diagCount > 0 && <span className={`ptab__count${errCount > 0 ? ' ptab__count--bad' : ' ptab__count--warn'}`}>{diagCount}</span>}
        </button>
        <button type="button" role="tab" aria-selected={tab === 'log'} className={`ptab${tab === 'log' ? ' ptab--active' : ''}`} onClick={() => onTab('log')}>
          Ham çıktı
        </button>
        <div className="panel__spacer" />
        {running && (
          <span className="panel__running" role="status">
            <span className="spinner" aria-hidden="true" />
            {running === 'submit' ? 'Teslim ediliyor…' : 'Test çalışıyor…'}
          </span>
        )}
        {result && !running && (
          <span className="panel__meta" title="Derleme dahil toplam süre">
            {restoredAt !== null && <span className="panel__restored">önceki çalıştırma · </span>}
            {stale && <span className="panel__restored">eski sonuç · </span>}
            {result.kind === 'submit' ? 'Teslim' : 'Test'} · {formatDuration(result.duration_ms)}
          </span>
        )}
      </div>

      <div className="panel__body">
        {notice && (
          <div className={`notice notice--${notice.kind}`} role={notice.kind === 'error' ? 'alert' : 'status'}>
            {notice.text}
          </div>
        )}
        {stale && result && !running && (
          <div className="result-banner result-banner--stale" role="status">
            <strong>Kod değişti, tekrar çalıştır.</strong>{' '}
            <span className="result-banner__note">{stale}; aşağıdaki sonuç eski kodun sonucu ({STATUS_LABEL[result.status].toLowerCase()}).</span>
          </div>
        )}
        {restoredAt !== null && !stale && result && !running && (
          <div className={`result-banner result-banner--${result.status === 'passed' ? 'ok' : 'bad'} result-banner--restored`} role="status">
            <strong>{STATUS_LABEL[result.status]}.</strong>{' '}
            {result.tests_total > 0 && <>{result.tests_passed}/{result.tests_total} test geçti. </>}
            <span className="result-banner__note">
              Önceki çalıştırma ({result.kind === 'submit' ? 'teslim' : 'test'}, {formatWhen(restoredAt)}); kodu o zamandan beri değiştirmiş olabilirsin.
            </span>
          </div>
        )}
        {summary && result && !running && restoredAt === null && !stale && (
          <div className={`result-banner result-banner--${result.status === 'passed' ? 'ok' : 'bad'}`} role="status">
            <strong>{STATUS_LABEL[result.status]}.</strong> {summary}
          </div>
        )}

        {!result && !running && !notice && (
          <p className="muted panel__empty">
            Henüz bir şey çalıştırmadın. <strong>Test et</strong> ile testleri dene; hazır olduğunda <strong>Teslim et</strong>.
          </p>
        )}

        {tab === 'tests' && result && (
          <div className="tests">
            {result.status === 'compile_error' && (
              <p className="muted">Kod derlenemedi, bu yüzden testler çalıştırılmadı. Ayrıntılar için «Derleyici» sekmesine bak.</p>
            )}
            {result.status === 'timeout' && (
              <p className="muted">Çalışma süresi aşıldı. Sonsuz döngü veya çok yavaş bir kod olabilir.</p>
            )}
            {result.status === 'failed' && result.tests.length === 0 && (
              <p className="muted">Hiç test çalışmadı.</p>
            )}
            <ul className="tests__list">
              {result.tests.map((t) => (
                <li key={t.name} className={`test ${t.passed ? 'test--ok' : 'test--bad'}`}>
                  {t.passed ? (
                    <div className="test__row">
                      <span className="test__mark" aria-label="geçti">✓</span>
                      <code>{t.name}</code>
                    </div>
                  ) : (
                    <details open>
                      <summary className="test__row">
                        <span className="test__mark" aria-label="başarısız">✗</span>
                        <code>{t.name}</code>
                      </summary>
                      {t.output && <pre className="test__output">{t.output}</pre>}
                    </details>
                  )}
                </li>
              ))}
            </ul>
          </div>
        )}

        {tab === 'diagnostics' && result && (
          <div className="diags">
            {diagCount === 0 && <p className="muted">Derleyiciden mesaj yok.</p>}
            <ul className="diags__list">
              {result.diagnostics.map((d, i) => {
                const { line, column } = d;
                return (
                <li key={i} className="diag">
                  {d.hidden || line === null || column === null ? (
                    <div className="diag__btn diag__btn--static">
                      <SeverityIcon severity={d.severity} />
                      <span className="diag__msg">{d.message}</span>
                      <span className="diag__loc diag__loc--hidden" title="Bu hata gizli test kodunda; koda gidilemez">
                        gizli test kodunda
                      </span>
                    </div>
                  ) : (
                    <button type="button" className="diag__btn" onClick={() => onJump({ ...d, line, column })} title="Koda git">
                      <SeverityIcon severity={d.severity} />
                      <span className="diag__msg">{d.message}</span>
                      <span className="diag__loc">
                        {d.file}:{d.line}:{d.column}
                      </span>
                    </button>
                  )}
                  {d.rendered && (
                    <details className="diag__more">
                      <summary>Tam derleyici çıktısı</summary>
                      <pre>{d.rendered}</pre>
                    </details>
                  )}
                </li>
                );
              })}
            </ul>
          </div>
        )}

        {tab === 'log' && result && <pre className="rawlog">{result.log || '(boş)'}</pre>}
        {tab !== 'tests' && !result && null}
      </div>
    </div>
  );
}
