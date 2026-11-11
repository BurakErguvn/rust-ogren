'use client';
import Link from 'next/link';
import { useCallback, useEffect, useRef, useState } from 'react';
import { api, ApiError } from '@/lib/api';
import { clearResult, loadCreatedFiles, loadPanelRatio, loadResult, saveCreatedFiles, saveLastVisit, savePanelRatio, saveResult } from '@/lib/storage';
import type { CompilerDiagnostic, ExerciseFile, ExerciseFiles, ExerciseProgress, RunResult } from '@/lib/types';
import ExerciseEditor from './ExerciseEditor';
import type { EditorHandle } from './ExerciseEditor';
import { ResultsPanel } from './ResultsPanel';
import type { PanelTab } from './ResultsPanel';
import { ConfirmDialog } from './ConfirmDialog';
import { NewFileDialog } from './NewFileDialog';
import { ScoreBadge, CheckIcon } from './ScoreBadge';
import { Title } from './Title';
import { useToc } from './TocContext';

type SaveState = 'saved' | 'dirty' | 'saving' | 'error';
type Notice = { kind: 'error' | 'info'; text: string };

const SAVE_TEXT: Record<SaveState, string> = {
  saved: 'Kaydedildi',
  dirty: 'Kaydedilmedi',
  saving: 'Kaydediliyor…',
  error: 'Kaydedilemedi',
};

const DEFAULT_PANEL_RATIO = 0.28;
const MIN_PANEL_PX = 96;
const MIN_EDITOR_PX = 200;
const clamp = (v: number, lo: number, hi: number) => Math.min(Math.max(v, lo), hi);

/** Yeni dosyayı sunucunun sıralamasına benzer yere koyar: lib.rs/main.rs, diğer .rs (alfabetik), sonra toml. */
function insertFile(files: ExerciseFile[], f: ExerciseFile): ExerciseFile[] {
  const rank = (x: ExerciseFile) => (x.language === 'toml' ? 3 : x.path === 'src/lib.rs' ? 0 : x.path === 'src/main.rs' ? 1 : 2);
  const idx = files.findIndex((x) => rank(x) > rank(f) || (rank(x) === rank(f) && x.path.localeCompare(f.path) > 0));
  return idx < 0 ? [...files, f] : [...files.slice(0, idx), f, ...files.slice(idx)];
}

const isMac = () => typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform);

const potentialScore = (failed: number) => Math.max(100 - 10 * failed, 50);

export default function ExerciseView({ pageId }: { pageId: string }) {
  const { pages, refresh: refreshToc } = useToc();
  const [data, setData] = useState<ExerciseFiles | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [progress, setProgress] = useState<ExerciseProgress | null>(null);
  const [dirty, setDirty] = useState<ReadonlySet<string>>(new Set());
  const [saveState, setSaveState] = useState<SaveState>('saved');
  const [busy, setBusy] = useState<'test' | 'submit' | 'reset' | null>(null);
  const [result, setResult] = useState<RunResult | null>(null);
  const [summary, setSummary] = useState<string | null>(null);
  const [notice, setNotice] = useState<Notice | null>(null);
  const [tab, setTab] = useState<PanelTab>('tests');
  const [confirmReset, setConfirmReset] = useState(false);
  const [panelRatio, setPanelRatio] = useState(DEFAULT_PANEL_RATIO);
  const [restoredAt, setRestoredAt] = useState<number | null>(null);
  const [noExercise, setNoExercise] = useState(false);
  const [created, setCreated] = useState<readonly string[]>([]);
  const [newFileOpen, setNewFileOpen] = useState(false);
  const [newFileBusy, setNewFileBusy] = useState(false);
  const [newFileError, setNewFileError] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);
  const [saveErrors, setSaveErrors] = useState<ReadonlyMap<string, string>>(new Map());
  const [stale, setStale] = useState<string | null>(null);
  const lastSaveError = useRef<string | null>(null);

  const editorRef = useRef<EditorHandle>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  const dirtyRef = useRef(new Map<string, string>()); // path -> en son içerik (kaydedilmemiş)
  const autosaveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const saveChain = useRef<Promise<boolean>>(Promise.resolve(true));
  const idRef = useRef(pageId);
  const busyRef = useRef(false);
  const runRef = useRef<(kind: 'test' | 'submit') => Promise<void>>(async () => {});

  // Kayıtlı panel oranı (yalnızca tarayıcıda okunabilir).
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setPanelRatio(loadPanelRatio(DEFAULT_PANEL_RATIO));
  }, []);

  // ---- yükleme -------------------------------------------------------------------------------

  const load = useCallback(() => {
    let cancelled = false;
    api
      .files(pageId)
      .then((d) => {
        if (cancelled) return;
        setData(d);
        setProgress(d.progress);
        const known = new Set(d.files.map((f) => f.path));
        setCreated(loadCreatedFiles(pageId).filter((p) => known.has(p)));
        const prev = loadResult(pageId);
        if (prev) {
          setResult(prev.result);
          setRestoredAt(prev.savedAt);
          setTab(prev.result.status === 'compile_error' ? 'diagnostics' : 'tests');
        }
      })
      .catch((e: unknown) => {
        if (cancelled) return;
        if (e instanceof ApiError && e.status === 404) setNoExercise(true);
        else setLoadError(e instanceof Error ? e.message : 'Egzersiz yüklenemedi.');
      });
    return () => {
      cancelled = true;
    };
  }, [pageId]);

  useEffect(() => {
    idRef.current = pageId;
    saveLastVisit({ kind: 'egzersiz', id: pageId });
    return load();
  }, [pageId, load]);

  const pageTitle = pages.get(pageId)?.title;
  useEffect(() => {
    document.title = `Egzersiz: ${(pageTitle ?? pageId).replaceAll('`', '')} · Rust Öğren`;
  }, [pageTitle, pageId]);

  // ---- kaydetme ------------------------------------------------------------------------------

  const syncDirty = useCallback(() => setDirty(new Set(dirtyRef.current.keys())), []);

  const saveAll = useCallback((): Promise<boolean> => {
    if (autosaveTimer.current) {
      clearTimeout(autosaveTimer.current);
      autosaveTimer.current = null;
    }
    const id = idRef.current;
    saveChain.current = saveChain.current.then(async () => {
      if (dirtyRef.current.size === 0) return true;
      setSaveState('saving');
      let ok = true;
      lastSaveError.current = null;
      for (const [path, content] of [...dirtyRef.current]) {
        try {
          await api.saveFile(id, path, content);
          if (dirtyRef.current.get(path) === content) dirtyRef.current.delete(path);
          setSaveErrors((m) => {
            if (!m.has(path)) return m;
            const n = new Map(m);
            n.delete(path);
            return n;
          });
        } catch (e) {
          ok = false;
          const msg = e instanceof Error ? e.message : 'Dosya kaydedilemedi.';
          lastSaveError.current = msg;
          setSaveErrors((m) => new Map(m).set(path, msg));
          setNotice({ kind: 'error', text: msg });
        }
      }
      syncDirty();
      setSaveState(!ok ? 'error' : dirtyRef.current.size > 0 ? 'dirty' : 'saved');
      return ok;
    });
    return saveChain.current;
  }, [syncDirty]);

  const onEdit = useCallback(
    (path: string, content: string) => {
      const had = dirtyRef.current.has(path);
      dirtyRef.current.set(path, content);
      if (!had) syncDirty();
      setSaveState((s) => (s === 'saving' ? s : 'dirty'));
      if (autosaveTimer.current) clearTimeout(autosaveTimer.current);
      autosaveTimer.current = setTimeout(() => void saveAll(), 1000);
    },
    [saveAll, syncDirty],
  );

  // Sayfadan ayrılırken bekleyen değişiklikleri gönder.
  useEffect(() => {
    const dirtyFiles = dirtyRef.current;
    const flush = () => {
      for (const [path, content] of dirtyFiles) {
        const url = `/api/exercises/${idRef.current.split('/').map(encodeURIComponent).join('/')}/files/${path.split('/').map(encodeURIComponent).join('/')}`;
        void fetch(url, {
          method: 'PUT',
          keepalive: true,
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ content }),
        }).catch(() => {});
      }
    };
    window.addEventListener('pagehide', flush);
    return () => {
      window.removeEventListener('pagehide', flush);
      flush(); // istemci tarafı gezinme
      if (autosaveTimer.current) clearTimeout(autosaveTimer.current);
    };
  }, []);

  // ---- test / teslim ---------------------------------------------------------------------------

  const run = useCallback(
    async (kind: 'test' | 'submit') => {
      if (busyRef.current) return;
      busyRef.current = true;
      setNotice(null);
      setSummary(null);
      setBusy(kind);
      try {
        const saved = await saveAll();
        if (!saved) {
          setNotice({ kind: 'error', text: `${lastSaveError.current ?? 'Dosyalar kaydedilemedi'}\n(Çalıştırma iptal edildi.)` });
          return;
        }
        const before = progress;
        const r = await api.run(pageId, kind);
        setResult(r);
        setStale(null);
        setRestoredAt(null);
        saveResult(pageId, r);
        setProgress(r.progress);
        setTab(r.status === 'compile_error' ? 'diagnostics' : 'tests');
        setSummary(describeResult(kind, r, before));
        if (kind === 'submit') void refreshToc();
      } catch (e) {
        if (e instanceof ApiError && e.status === 409) {
          setNotice({ kind: 'info', text: e.message || 'Başka bir test çalışıyor, lütfen bekle.' });
        } else {
          setNotice({ kind: 'error', text: e instanceof Error ? e.message : 'Çalıştırma başarısız oldu.' });
        }
      } finally {
        busyRef.current = false;
        setBusy(null);
      }
    },
    [saveAll, pageId, progress, refreshToc],
  );

  useEffect(() => {
    runRef.current = run;
  });

  // Ctrl/Cmd+Enter = Test et (editörde Monaco komutu, dışarıda pencere dinleyicisi).
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Enter' && (e.ctrlKey || e.metaKey) && !e.defaultPrevented && !e.shiftKey && !e.altKey) {
        e.preventDefault();
        void runRef.current('test');
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  const doReset = useCallback(async () => {
    setConfirmReset(false);
    setNotice(null);
    if (busyRef.current) return;
    busyRef.current = true;
    setBusy('reset');
    try {
      await saveChain.current; // süren kaydetmeyi bekle
      if (autosaveTimer.current) clearTimeout(autosaveTimer.current);
      dirtyRef.current.clear();
      const d = await api.reset(pageId);
      setData(d);
      setProgress(d.progress);
      syncDirty();
      setSaveState('saved');
      setResult(null);
      setStale(null);
      setSaveErrors(new Map());
      setRestoredAt(null);
      clearResult(pageId);
      setCreated([]);
      saveCreatedFiles(pageId, []);
      setSummary(null);
      setNotice({ kind: 'info', text: 'Egzersiz başlangıç haline döndürüldü. Puanın değişmedi.' });
    } catch (e) {
      setNotice({ kind: 'error', text: e instanceof Error ? e.message : 'Sıfırlama başarısız oldu.' });
    } finally {
      busyRef.current = false;
      setBusy(null);
    }
  }, [pageId, syncDirty]);

  // ---- yeni dosya / silme --------------------------------------------------------------------------

  const createFile = useCallback(
    async (path: string) => {
      setNewFileBusy(true);
      setNewFileError(null);
      try {
        await saveChain.current;
        await api.saveFile(pageId, path, '');
        setData((d) => (d ? { ...d, files: insertFile(d.files, { path, content: '', editable: true, language: 'rust' }) } : d));
        setCreated((c) => {
          const next = [...c, path];
          saveCreatedFiles(pageId, next);
          return next;
        });
        setNewFileOpen(false);
        setStale(`${path} dosyası eklendi`);
      } catch (e) {
        setNewFileError(e instanceof Error ? e.message : 'Dosya oluşturulamadı.');
      } finally {
        setNewFileBusy(false);
      }
    },
    [pageId],
  );

  const deleteFile = useCallback(
    async (path: string) => {
      setDeleteTarget(null);
      setNotice(null);
      try {
        await saveChain.current;
        dirtyRef.current.delete(path);
        syncDirty();
        try {
          await api.deleteFile(pageId, path);
        } catch (e) {
          if (!(e instanceof ApiError && e.status === 404)) throw e;
        }
        setData((d) => (d ? { ...d, files: d.files.filter((f) => f.path !== path) } : d));
        setStale(`${path} dosyası silindi`);
        setSaveErrors((m) => {
          if (!m.has(path)) return m;
          const n = new Map(m);
          n.delete(path);
          return n;
        });
        setCreated((c) => {
          const next = c.filter((x) => x !== path);
          saveCreatedFiles(pageId, next);
          return next;
        });
      } catch (e) {
        setNotice({ kind: 'error', text: e instanceof Error ? `Dosya silinemedi: ${e.message}` : 'Dosya silinemedi.' });
      }
    },
    [pageId, syncDirty],
  );

  const jump = useCallback((d: CompilerDiagnostic & { line: number; column: number }) => {
    editorRef.current?.reveal(d.file, d.line, d.column);
  }, []);

  // ---- panel yüksekliği (oran olarak saklanır) ----------------------------------------------------

  const contentHeight = () => {
    const root = rootRef.current;
    if (!root) return 0;
    const cs = getComputedStyle(root);
    return root.clientHeight - parseFloat(cs.paddingTop) - parseFloat(cs.paddingBottom);
  };

  const applyPanelPx = (px: number, persist: boolean) => {
    const total = contentHeight();
    if (total <= 0) return;
    const maxPx = Math.max(MIN_PANEL_PX, total - MIN_EDITOR_PX - 60);
    const ratio = clamp(px, MIN_PANEL_PX, maxPx) / total;
    setPanelRatio(ratio);
    if (persist) savePanelRatio(ratio);
  };

  const startDrag = (e: React.PointerEvent) => {
    e.preventDefault();
    const root = rootRef.current;
    if (!root) return;
    const bottom = root.getBoundingClientRect().bottom - parseFloat(getComputedStyle(root).paddingBottom);
    const move = (ev: PointerEvent) => applyPanelPx(bottom - ev.clientY, false);
    const up = (ev: PointerEvent) => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
      applyPanelPx(bottom - ev.clientY, true);
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  };

  const onSplitterKey = (e: React.KeyboardEvent) => {
    const cur = panelRatio * contentHeight();
    if (e.key === 'ArrowUp') applyPanelPx(cur + 24, true);
    else if (e.key === 'ArrowDown') applyPanelPx(cur - 24, true);
    else if (e.key === 'Home' || e.key === 'Enter') applyPanelPx(DEFAULT_PANEL_RATIO * contentHeight(), true);
    else return;
    e.preventDefault();
  };

  // ---- görünüm -----------------------------------------------------------------------------------

  const title = pages.get(pageId)?.title;

  if (noExercise || (pages.size > 0 && pages.get(pageId) && pages.get(pageId)?.exercise === null)) {
    return (
      <div className="exercise-loading">
        <div className="notice notice--info" role="status">
          <strong>Bu sayfanın egzersizi yok.</strong>
          <p>Bu bölüm yalnızca anlatımdan oluşuyor.</p>
          <Link href={`/ders/${pageId}/`} className="btn">
            ← Anlatıma dön
          </Link>
        </div>
      </div>
    );
  }
  if (loadError) {
    return (
      <div className="exercise-loading">
        <div className="notice notice--error" role="alert">
          <strong>Egzersiz yüklenemedi.</strong>
          <p>{loadError}</p>
          <button type="button" className="btn" onClick={() => {
              setLoadError(null);
              load();
            }}>
            Tekrar dene
          </button>
        </div>
      </div>
    );
  }
  if (!data || !progress) {
    return (
      <div className="exercise-loading" aria-busy="true">
        Egzersiz yükleniyor…
      </div>
    );
  }

  const running = busy === 'test' || busy === 'submit' ? busy : null;
  const disabled = busy !== null;

  const mac = isMac();
  const testShortcut = mac ? '⌘↵' : 'Ctrl+Enter';

  return (
    <div className="exercise" ref={rootRef}>
      <div className="exercise__bar">
        <div className="exercise__title">
          <span className="exercise__kicker">Egzersiz</span>
          <h1>{title ? <Title text={title} /> : pageId}</h1>
        </div>
        <div className="toolbar__buttons">
          <button
            type="button"
            className="btn btn--primary"
            disabled={disabled}
            onClick={() => void run('test')}
            title={`Derle ve testleri çalıştır (puanı etkilemez) · ${mac ? '⌘+Enter' : 'Ctrl+Enter'}`}
            aria-keyshortcuts={mac ? 'Meta+Enter' : 'Control+Enter'}
          >
            {busy === 'test' ? 'Test ediliyor…' : 'Test et'}
            {busy !== 'test' && <kbd className="btn__kbd">{testShortcut}</kbd>}
          </button>
          <button
            type="button"
            className="btn btn--outline"
            disabled={disabled}
            onClick={() => void run('submit')}
            title="Derle, testleri çalıştır ve sonucu puana işle (başarısız teslim puanı düşürür)"
          >
            {busy === 'submit' ? 'Teslim ediliyor…' : 'Teslim et'}
          </button>
          <button type="button" className="btn btn--ghost" disabled={disabled} onClick={() => setConfirmReset(true)} title="Dosyaları başlangıç haline döndür (puan değişmez)">
            Sıfırla
          </button>
        </div>
        <div className="toolbar__status">
          {progress.completed ? (
            <span className="state state--done">
              <CheckIcon /> Tamamlandı <ScoreBadge score={progress.score} large />
            </span>
          ) : (
            <span className="state" title={`Şimdi başarıyla teslim edersen ${potentialScore(progress.failed_submits)} puan alırsın`}>
              Tamamlanmadı
            </span>
          )}
          <span className="state" title="Bu egzersiz için başarısız teslim sayısı">
            Başarısız teslim: <strong>{progress.failed_submits}</strong>
          </span>
          <span className={`save save--${saveState}`} role="status">
            {SAVE_TEXT[saveState]}
          </span>
        </div>
        <Link href={`/ders/${pageId}/`} className="btn btn--small exercise__back">
          ← Anlatıma dön
        </Link>
      </div>

      <div className="exercise__editor">
        <ExerciseEditor
          key={data.id}
          data={data}
          diagnostics={restoredAt === null && !stale ? (result?.diagnostics ?? []) : []}
          dirty={dirty}
          saveErrors={saveErrors}
          onEdit={onEdit}
          onSave={() => void saveAll()}
          onSwitchTab={() => void saveAll()}
          onRunTest={() => void runRef.current('test')}
          onNewFile={() => {
            setNewFileError(null);
            setNewFileOpen(true);
          }}
          canDelete={(path) => data.files.find((f) => f.path === path)?.deletable ?? created.includes(path)}
          onDeleteFile={setDeleteTarget}
          handleRef={editorRef}
        />
      </div>

      <div
        className="splitter"
        role="separator"
        aria-orientation="horizontal"
        aria-label="Editör ve sonuç paneli oranı (yukarı/aşağı ok ile ayarla)"
        tabIndex={0}
        onPointerDown={startDrag}
        onKeyDown={onSplitterKey}
        onDoubleClick={() => applyPanelPx(DEFAULT_PANEL_RATIO * contentHeight(), true)}
        title="Sürükleyerek yeniden boyutlandır; çift tıkla: varsayılan"
      />
      <div className="exercise__panel" style={{ height: `${(panelRatio * 100).toFixed(2)}%` }}>
        <ResultsPanel stale={stale} result={result} restoredAt={restoredAt} running={running} notice={notice} summary={summary} tab={tab} onTab={setTab} onJump={jump} />
      </div>

      <NewFileDialog
        open={newFileOpen}
        existing={data.files.map((f) => f.path)}
        busy={newFileBusy}
        error={newFileError}
        onCreate={(p) => void createFile(p)}
        onCancel={() => setNewFileOpen(false)}
      />
      <ConfirmDialog
        open={deleteTarget !== null}
        title="Dosyayı sil?"
        confirmLabel="Evet, sil"
        danger
        onConfirm={() => deleteTarget && void deleteFile(deleteTarget)}
        onCancel={() => setDeleteTarget(null)}
      >
        <p>
          <code>{deleteTarget}</code> dosyası silinecek. Yalnızca senin oluşturduğun dosyalar silinebilir; egzersizin özgün dosyaları silinemez.
        </p>
      </ConfirmDialog>
      <ConfirmDialog
        open={confirmReset}
        title="Egzersizi sıfırla?"
        confirmLabel="Evet, sıfırla"
        danger
        onConfirm={() => void doReset()}
        onCancel={() => setConfirmReset(false)}
      >
        <p>Bu egzersizdeki tüm dosyalar başlangıç haline döner; yaptığın değişiklikler kaybolur. Puanın ve başarısız teslim sayın değişmez.</p>
      </ConfirmDialog>
    </div>
  );
}

function describeResult(kind: 'test' | 'submit', r: RunResult, before: ExerciseProgress | null): string {
  const counts = r.tests_total > 0 ? `${r.tests_passed}/${r.tests_total} test geçti.` : '';
  if (kind === 'test') {
    if (r.status === 'passed') return `${counts} Hazırsan şimdi teslim edebilirsin.`;
    if (r.status === 'compile_error') return 'Kod derlenemedi.';
    if (r.status === 'timeout') return 'Çalışma süresi aşıldı.';
    return `${counts} Bu bir test çalıştırmasıydı, puanını etkilemedi.`;
  }
  if (before?.completed) return `${counts} Bu egzersizi zaten tamamlamıştın; puanın (${r.progress.score ?? '—'}) değişmedi.`;
  if (r.status === 'passed') return `${counts} Egzersizi tamamladın, puanın: ${r.progress.score ?? '—'}.`;
  return `${counts} Başarısız teslim sayısı: ${r.progress.failed_submits}. Şimdi başarıyla teslim edersen ${potentialScore(r.progress.failed_submits)} puan alırsın.`;
}
