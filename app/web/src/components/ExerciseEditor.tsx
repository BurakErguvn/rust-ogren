'use client';
// Monaco tabanlı çok dosyalı editör + LSP oturumu. Yalnızca istemcide yüklenir (ExerciseView dinamik import eder).
import { useCallback, useEffect, useImperativeHandle, useRef, useState } from 'react';
import type { Ref } from 'react';
import type * as Monaco from 'monaco-editor/editor/editor.api.js';
import { monaco } from '@/lib/monaco-setup';
import { LspSession } from '@/lib/lsp-monaco';
import type { LspStatus } from '@/lib/lsp';
import { lspUrl } from '@/lib/api';
import { useResolvedTheme } from '@/lib/theme';
import type { CompilerDiagnostic, ExerciseFile, ExerciseFiles } from '@/lib/types';

export const COMPILER_MARKER_OWNER = 'rustc';
const SAVE_MARKER_OWNER = 'save';

export interface EditorHandle {
  reveal: (path: string, line: number, column: number) => void;
}

interface Props {
  data: ExerciseFiles;
  diagnostics: CompilerDiagnostic[];
  dirty: ReadonlySet<string>;
  /** Dosya yolu -> sunucunun kaydetme hatası mesajı. */
  saveErrors: ReadonlyMap<string, string>;
  onEdit: (path: string, content: string) => void;
  onSave: () => void;
  onSwitchTab: () => void;
  onRunTest: () => void;
  onNewFile: () => void;
  /** Silinebilir (öğrencinin oluşturduğu) dosya mı? */
  canDelete: (path: string) => boolean;
  onDeleteFile: (path: string) => void;
  handleRef: Ref<EditorHandle>;
}

const STATUS_SHORT: Record<LspStatus, string> = {
  connecting: '…',
  indexing: '…',
  ready: '✓',
  disconnected: '✗',
};

const STATUS_TEXT: Record<LspStatus, string> = {
  connecting: 'bağlanıyor',
  indexing: 'indeksleniyor',
  ready: 'hazır',
  disconnected: 'bağlantı yok',
};

let themesDefined = false;
function defineThemes() {
  if (themesDefined) return;
  themesDefined = true;
  monaco.editor.defineTheme('ro-light', {
    base: 'vs',
    inherit: true,
    rules: [],
    colors: { 'editor.background': '#fbfaf7', 'editorGutter.background': '#fbfaf7' },
  });
  monaco.editor.defineTheme('ro-dark', {
    base: 'vs-dark',
    inherit: true,
    rules: [],
    colors: { 'editor.background': '#14171c', 'editorGutter.background': '#14171c' },
  });
}

export const WORKSPACE_PREFIX = '@workspace/';

/** Sekmede görünen kısa ad. */
export function fileLabel(path: string): string {
  return path.startsWith(WORKSPACE_PREFIX) ? `${path.slice(WORKSPACE_PREFIX.length)} (kök)` : path;
}

/** İpucu (tooltip) için tam ad. */
export function fileTitle(path: string): string {
  return path.startsWith(WORKSPACE_PREFIX) ? `${path.slice(WORKSPACE_PREFIX.length)} (çalışma alanı kökü)` : path;
}

/** Kaydetme hatası mesajından satır/sütun numarasını çıkarır ("at line 3", "--> line 3", "satır 3"). */
export function parseErrorLocation(message: string): { line: number; column: number | null } | null {
  const l = /(?:\bline\b|\bsatır\b)\s*:?\s*(\d+)/i.exec(message);
  if (!l) return null;
  const c = /(?:\bcolumn\b|\bsütun\b)\s*:?\s*(\d+)/i.exec(message);
  return { line: Number(l[1]), column: c ? Number(c[1]) : null };
}

const languageOf = (f: ExerciseFile) => f.language ?? (f.path.endsWith('.toml') ? 'toml' : 'rust');

function LockIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" aria-hidden="true" className="editor__hidden-icon">
      <rect x="3" y="7" width="10" height="7" rx="1.5" fill="currentColor" />
      <path d="M5.5 7V5a2.5 2.5 0 0 1 5 0v2" fill="none" stroke="currentColor" strokeWidth="1.6" />
    </svg>
  );
}

export default function ExerciseEditor({ data, diagnostics, dirty, saveErrors, onEdit, onSave, onSwitchTab, onRunTest, onNewFile, canDelete, onDeleteFile, handleRef }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<Monaco.editor.IStandaloneCodeEditor | null>(null);
  const modelsRef = useRef(new Map<string, Monaco.editor.ITextModel>());
  const viewStates = useRef(new Map<string, Monaco.editor.ICodeEditorViewState | null>());
  const sessionRef = useRef<LspSession | null>(null);
  const programmatic = useRef(false);
  const [active, setActive] = useState(data.files[0]?.path ?? '');
  const activeRef = useRef(active);
  const [lsp, setLsp] = useState<{ status: LspStatus; detail?: string }>({ status: 'connecting' });
  const theme = useResolvedTheme();

  // Callback'ler her render'da değişebilir; editör olayları son halini kullansın.
  const cbs = useRef({ onEdit, onSave, onSwitchTab, onRunTest });
  useEffect(() => {
    cbs.current = { onEdit, onSave, onSwitchTab, onRunTest };
  });
  const hasHiddenTests = (data.hidden_test_files ?? 0) > 0 || data.files.some((f) => f.has_hidden_tests);

  const docUri = useCallback(
    (path: string) =>
      path.startsWith(WORKSPACE_PREFIX)
        ? `${data.workspace_root_uri}/${path.slice(WORKSPACE_PREFIX.length)}`
        : `${data.exercise_root_uri}/${path}`,
    [data.exercise_root_uri, data.workspace_root_uri],
  );
  const changeSubs = useRef(new Map<string, Monaco.IDisposable>());
  const fileObjs = useRef(new Map<string, ExerciseFile>());
  const initialized = useRef(false);

  const activate = useCallback((path: string, notify: boolean) => {
    const editor = editorRef.current;
    const model = modelsRef.current.get(path);
    if (!editor || !model) return;
    if (notify && activeRef.current !== path) cbs.current.onSwitchTab();
    const prev = modelsRef.current.get(activeRef.current);
    if (prev && editor.getModel() === prev) viewStates.current.set(activeRef.current, editor.saveViewState());
    editor.setModel(model);
    const vs = viewStates.current.get(path);
    if (vs) editor.restoreViewState(vs);
    activeRef.current = path;
    setActive(path);
  }, []);

  // Editörü oluştur.
  useEffect(() => {
    defineThemes();
    const editor = monaco.editor.create(containerRef.current!, {
      model: null,
      automaticLayout: true,
      fontSize: 14,
      lineHeight: 21,
      fontFamily: 'ui-monospace, SFMono-Regular, "JetBrains Mono", "Fira Code", Menlo, Consolas, monospace',
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      wrappingIndent: 'indent',
      scrollbar: { alwaysConsumeMouseWheel: false },
      tabSize: 4,
      insertSpaces: true,
      padding: { top: 10, bottom: 10 },
      renderLineHighlight: 'line',
      smoothScrolling: true,
      fixedOverflowWidgets: true,
      theme: document.documentElement.dataset.theme === 'dark' ? 'ro-dark' : 'ro-light',
      ariaLabel: 'Rust kod editörü',
      'semanticHighlighting.enabled': false,
    });
    editorRef.current = editor;
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => cbs.current.onSave());
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => cbs.current.onRunTest());
    return () => {
      editor.dispose();
      editorRef.current = null;
    };
  }, []);

  // LSP oturumu (egzersiz başına bir kez; bileşen `key={id}` ile yeniden oluşturulur).
  useEffect(() => {
    const models = modelsRef.current;
    const subs = changeSubs.current;
    const objs = fileObjs.current;
    const session = new LspSession(monaco, {
      url: lspUrl(data.id),
      rootUri: data.workspace_root_uri,
      onStatus: (status, detail) => setLsp({ status, detail }),
    });
    sessionRef.current = session;
    // Modeller aşağıdaki uzlaştırma efektinde (bu efektten sonra çalışır) oluşturulur.
    session.start();
    return () => {
      subs.forEach((s) => s.dispose());
      subs.clear();
      session.dispose();
      sessionRef.current = null;
      models.forEach((m) => m.dispose());
      models.clear();
      objs.clear();
      initialized.current = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data.id]);

  // Dosya listesi <-> modeller. Yeni dosya eklenir (sonradan eklenen açılır), silinen kaldırılır,
  // sunucudan gelen yeni nesnenin içeriği (ör. sıfırlama) modele yazılır; değişmeyen nesnelere dokunulmaz.
  useEffect(() => {
    const models = modelsRef.current;
    const session = sessionRef.current;
    const wasInitialized = initialized.current;
    let openNew: string | null = null;
    programmatic.current = true;
    try {
      for (const f of data.files) {
        let model = models.get(f.path);
        if (!model) {
          const uri = monaco.Uri.parse(docUri(f.path));
          model = monaco.editor.getModel(uri) ?? monaco.editor.createModel(f.content, languageOf(f), uri);
          if (model.getValue() !== f.content) model.setValue(f.content);
          models.set(f.path, model);
          // rust-analyzer yalnızca .rs belgelerini görür; Cargo.toml gönderilmez.
          if (languageOf(f) === 'rust') session?.addModel(docUri(f.path), model);
          const m = model;
          changeSubs.current.set(
            f.path,
            m.onDidChangeContent(() => {
              if (!programmatic.current) cbs.current.onEdit(f.path, m.getValue());
            }),
          );
          if (wasInitialized) openNew = f.path;
        } else if (fileObjs.current.get(f.path) !== f && model.getValue() !== f.content) {
          model.setValue(f.content);
        }
        fileObjs.current.set(f.path, f);
      }
      const paths = new Set(data.files.map((f) => f.path));
      for (const [path, model] of [...models]) {
        if (paths.has(path)) continue;
        changeSubs.current.get(path)?.dispose();
        changeSubs.current.delete(path);
        if (!path.endsWith('.toml')) {
          session?.removeModel(docUri(path));
        }
        if (editorRef.current?.getModel() === model) editorRef.current.setModel(null);
        model.dispose();
        models.delete(path);
        fileObjs.current.delete(path);
        viewStates.current.delete(path);
      }
    } finally {
      programmatic.current = false;
    }
    if (!wasInitialized) {
      initialized.current = true;
      const first = data.files[0]?.path;
      if (first) {
        activeRef.current = '';
        activate(first, false);
      }
    } else if (openNew) {
      activate(openNew, true);
    } else if (!models.has(activeRef.current)) {
      const first = data.files[0]?.path;
      if (first) {
        activeRef.current = '';
        activate(first, false);
      }
    }
  }, [data.files, docUri, activate]);

  // Salt okunur dosyalar.
  useEffect(() => {
    const f = data.files.find((x) => x.path === active);
    editorRef.current?.updateOptions({ readOnly: f ? !f.editable : false });
  }, [active, data.files]);

  // Derleyici teşhisleri -> marker (LSP marker'larından ayrı owner).
  useEffect(() => {
    for (const [path, model] of modelsRef.current) {
      const markers: Monaco.editor.IMarkerData[] = diagnostics
        .filter((d): d is CompilerDiagnostic & { line: number; column: number } => d.file === path && !d.hidden && d.line !== null && d.column !== null)
        .map((d) => {
          const endLine = d.end_line !== null && d.end_line >= d.line ? d.end_line : d.line;
          const endColRaw = d.end_column ?? d.column + 1;
          const endCol = endLine === d.line && endColRaw <= d.column ? d.column + 1 : endColRaw;
          return {
            startLineNumber: d.line,
            startColumn: d.column,
            endLineNumber: endLine,
            endColumn: endCol,
            message: d.message,
            source: 'rustc',
            severity:
              d.severity === 'error'
                ? monaco.MarkerSeverity.Error
                : d.severity === 'warning'
                  ? monaco.MarkerSeverity.Warning
                  : monaco.MarkerSeverity.Info,
          };
        });
      monaco.editor.setModelMarkers(model, COMPILER_MARKER_OWNER, markers);
    }
  }, [diagnostics, data]);

  // Kaydetme hataları (ör. Cargo.toml ayrıştırma) -> ilgili dosyada satır marker'ı.
  useEffect(() => {
    for (const [path, model] of modelsRef.current) {
      const msg = saveErrors.get(path);
      const loc = msg ? parseErrorLocation(msg) : null;
      if (!msg || !loc) {
        monaco.editor.setModelMarkers(model, SAVE_MARKER_OWNER, []);
        continue;
      }
      const line = Math.min(Math.max(loc.line, 1), model.getLineCount());
      const col = loc.column !== null && loc.line === line ? Math.min(loc.column, model.getLineMaxColumn(line)) : 1;
      monaco.editor.setModelMarkers(model, SAVE_MARKER_OWNER, [
        {
          startLineNumber: line,
          startColumn: col,
          endLineNumber: line,
          endColumn: Math.max(model.getLineMaxColumn(line), col + 1),
          message: msg,
          source: 'sunucu',
          severity: monaco.MarkerSeverity.Error,
        },
      ]);
    }
  }, [saveErrors, data]);

  // Sekme şeridi: taşma göstergesi ve aktif sekmeyi görünür tutma.
  const tabsRef = useRef<HTMLDivElement>(null);
  const [overflow, setOverflow] = useState({ left: false, right: false });
  const updateOverflow = useCallback(() => {
    const el = tabsRef.current;
    if (!el) return;
    const left = el.scrollLeft > 1;
    const right = el.scrollLeft + el.clientWidth < el.scrollWidth - 1;
    setOverflow((o) => (o.left === left && o.right === right ? o : { left, right }));
  }, []);
  useEffect(() => {
    const el = tabsRef.current;
    if (!el) return;
    updateOverflow();
    const ro = new ResizeObserver(updateOverflow);
    ro.observe(el);
    el.addEventListener('scroll', updateOverflow, { passive: true });
    return () => {
      ro.disconnect();
      el.removeEventListener('scroll', updateOverflow);
    };
  }, [updateOverflow, data.files]);
  useEffect(() => {
    const el = tabsRef.current;
    const tab = el?.querySelector<HTMLElement>('.tab--active');
    if (!el || !tab) return;
    const pad = 28;
    if (tab.offsetLeft - pad < el.scrollLeft) el.scrollTo({ left: Math.max(0, tab.offsetLeft - pad) });
    else if (tab.offsetLeft + tab.offsetWidth + pad > el.scrollLeft + el.clientWidth) el.scrollTo({ left: tab.offsetLeft + tab.offsetWidth + pad - el.clientWidth });
  }, [active, data.files]);
  const scrollTabs = (dir: -1 | 1) => tabsRef.current?.scrollBy({ left: dir * Math.max(120, tabsRef.current.clientWidth * 0.6), behavior: 'smooth' });

  useEffect(() => {
    monaco.editor.setTheme(theme === 'dark' ? 'ro-dark' : 'ro-light');
  }, [theme]);

  useImperativeHandle(
    handleRef,
    () => ({
      reveal: (path, line, column) => {
        activate(path, true);
        const editor = editorRef.current;
        if (!editor) return;
        const pos = { lineNumber: Math.max(1, line), column: Math.max(1, column) };
        editor.setPosition(pos);
        editor.revealPositionInCenter(pos);
        editor.focus();
      },
    }),
    [activate],
  );

  const lspClass = `lsp lsp--${lsp.status}`;
  return (
    <div className="editor">
      <div className="editor__bar">
        <div className={`tabs-wrap${overflow.left ? ' tabs-wrap--left' : ''}${overflow.right ? ' tabs-wrap--right' : ''}`}>
        {overflow.left && (
          <button type="button" className="tabs-scroll tabs-scroll--left" onClick={() => scrollTabs(-1)} aria-label="Sekmeleri sola kaydır" tabIndex={-1}>
            ‹
          </button>
        )}
        <div className="tabs" role="tablist" aria-label="Dosyalar" ref={tabsRef}>
          {data.files.map((f) => (
            <div key={f.path} className={`tab${f.path === active ? ' tab--active' : ''}`}>
              <button
                type="button"
                role="tab"
                aria-selected={f.path === active}
                className="tab__btn"
                // Fare/dokunma: basma anında etkinleştir. Odak kayması sekmeyi kaydırırsa «click» hiç oluşmayabilir.
                onPointerDown={(e) => {
                  if (e.button === 0) activate(f.path, true);
                }}
                // Klavye (Enter/Boşluk) için: detail === 0.
                onClick={(e) => {
                  if (e.detail === 0) activate(f.path, true);
                }}
                title={saveErrors.get(f.path) ? `${fileTitle(f.path)}: ${saveErrors.get(f.path)}` : fileTitle(f.path)}
              >
                {fileLabel(f.path)}
                {saveErrors.has(f.path) && <span className="tab__error" role="img" aria-label="kaydedilemedi" />}
                {dirty.has(f.path) && <span className="tab__dirty" title="Kaydedilmemiş değişiklik" aria-label="kaydedilmemiş" />}
              </button>
              {canDelete(f.path) && (
                <button type="button" className="tab__close" onClick={() => onDeleteFile(f.path)} title={`${f.path} dosyasını sil`} aria-label={`${f.path} dosyasını sil`}>
                  ×
                </button>
              )}
            </div>
          ))}
          <button type="button" className="tab tab--new" onClick={onNewFile} title="src/ altında yeni bir .rs dosyası oluştur" aria-label="Yeni dosya">
            <span aria-hidden="true">+</span>
            <span className="tab__new-label">Yeni dosya</span>
          </button>
        </div>
        {overflow.right && (
          <button type="button" className="tabs-scroll tabs-scroll--right" onClick={() => scrollTabs(1)} aria-label="Sekmeleri sağa kaydır" tabIndex={-1}>
            ›
          </button>
        )}
        </div>
        {hasHiddenTests && (
          <span className="editor__hidden" title="Bu egzersizin testleri gizli; sonuçlar alt panelde görünür.">
            <LockIcon />
            <span className="editor__hidden-full">Bu egzersizin testleri gizli; sonuçlar alt panelde görünür.</span>
            <span className="editor__hidden-short">Testler gizli</span>
          </span>
        )}
        <div className="editor__bar-spacer" />
        <span
          className={lspClass}
          role="status"
          title={`rust-analyzer: ${STATUS_TEXT[lsp.status]}${lsp.detail ? ` (${lsp.detail})` : ''}`}
          aria-label={`rust-analyzer: ${STATUS_TEXT[lsp.status]}`}
        >
          <span className="lsp__dot" aria-hidden="true" />
          <span className="lsp__full" aria-hidden="true">rust-analyzer: {STATUS_TEXT[lsp.status]}</span>
          <span className="lsp__short" aria-hidden="true">LSP {STATUS_SHORT[lsp.status]}</span>
        </span>
        {lsp.status === 'disconnected' && (
          <button
            type="button"
            className="btn btn--small"
            onClick={() => {
              setLsp({ status: 'connecting' });
              sessionRef.current?.reconnect();
            }}
          >
            Yeniden bağlan
          </button>
        )}
      </div>
      <div className="editor__host" ref={containerRef} />
    </div>
  );
}
