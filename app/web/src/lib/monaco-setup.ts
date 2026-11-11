'use client';
// Monaco yerel olarak paketlenir (CDN yok, çevrimdışı çalışır).
// `monaco-editor` ana girişi tüm dilleri ve TS/CSS/HTML/JSON servislerini çeker; burada yalnızca
// çekirdek editör + ihtiyacımız olan özellikler + Rust dili alınır.
import * as monaco from 'monaco-editor/editor/editor.api.js';

import 'monaco-editor/editor/contrib/bracketMatching/browser/bracketMatching.js';
import 'monaco-editor/editor/contrib/clipboard/browser/clipboard.js';
import 'monaco-editor/editor/contrib/comment/browser/comment.js';
import 'monaco-editor/editor/contrib/contextmenu/browser/contextmenu.js';
import 'monaco-editor/editor/contrib/cursorUndo/browser/cursorUndo.js';
import 'monaco-editor/editor/contrib/folding/browser/folding.js';
import 'monaco-editor/editor/contrib/format/browser/formatActions.js';
import 'monaco-editor/editor/contrib/gotoError/browser/gotoError.js';
import 'monaco-editor/editor/contrib/gotoSymbol/browser/goToCommands.js';
import 'monaco-editor/editor/contrib/gotoSymbol/browser/link/goToDefinitionAtPosition.js';
import 'monaco-editor/editor/contrib/hover/browser/hoverContribution.js';
import 'monaco-editor/editor/contrib/indentation/browser/indentation.js';
import 'monaco-editor/editor/contrib/linesOperations/browser/linesOperations.js';
import 'monaco-editor/editor/contrib/multicursor/browser/multicursor.js';
import 'monaco-editor/editor/contrib/parameterHints/browser/parameterHints.js';
import 'monaco-editor/editor/contrib/snippet/browser/snippetController2.js';
import 'monaco-editor/editor/contrib/suggest/browser/suggestController.js';
import 'monaco-editor/editor/contrib/wordHighlighter/browser/wordHighlighter.js';
import 'monaco-editor/editor/contrib/wordOperations/browser/wordOperations.js';
import 'monaco-editor/editor/contrib/smartSelect/browser/smartSelect.js';
import 'monaco-editor/editor/contrib/find/browser/findController.js';
import 'monaco-editor/features/find/register.js';
import 'monaco-editor/editor/browser/coreCommands.js';
import 'monaco-editor/editor/browser/widget/codeEditor/codeEditorWidget.js';
import 'monaco-editor/editor/standalone/browser/quickAccess/standaloneGotoLineQuickAccess.js';

// Rust sözdizimi renklendirmesi (Monarch).
import 'monaco-editor/languages/definitions/rust/register.js';

// TOML (Cargo.toml) için küçük bir Monarch tanımı.
monaco.languages.register({ id: 'toml', extensions: ['.toml'], aliases: ['TOML'] });
monaco.languages.setLanguageConfiguration('toml', {
  comments: { lineComment: '#' },
  brackets: [
    ['[', ']'],
    ['{', '}'],
  ],
  autoClosingPairs: [
    { open: '[', close: ']' },
    { open: '{', close: '}' },
    { open: '"', close: '"', notIn: ['string'] },
    { open: "'", close: "'", notIn: ['string'] },
  ],
});
monaco.languages.setMonarchTokensProvider('toml', {
  tokenizer: {
    root: [
      [/#.*$/, 'comment'],
      [/^\s*\[\[?[^\]]*\]\]?/, 'type'],
      [/^\s*[A-Za-z0-9_.-]+(?=\s*=)/, 'variable'],
      [/"""/, 'string', '@mlstring'],
      [/"([^"\\]|\\.)*"/, 'string'],
      [/'[^']*'/, 'string'],
      [/\b(true|false)\b/, 'keyword'],
      [/\d{4}-\d{2}-\d{2}[T\d:.Z+-]*/, 'number'],
      [/[+-]?\d[\d_]*(\.\d+)?([eE][+-]?\d+)?/, 'number'],
      [/[{}[\],=]/, 'delimiter'],
    ],
    mlstring: [
      [/"""/, 'string', '@pop'],
      [/./, 'string'],
    ],
  },
});

const globalScope = self as unknown as { MonacoEnvironment?: monaco.Environment };
if (!globalScope.MonacoEnvironment) {
  globalScope.MonacoEnvironment = {
    getWorker() {
      return new Worker(new URL('monaco-editor/editor/editor.worker.js', import.meta.url), { type: 'module' });
    },
  };
}

if (process.env.NODE_ENV === 'development') {
  // Geliştirmede tarayıcı konsolundan/otomasyondan erişim için.
  (window as unknown as { monaco: typeof monaco }).monaco = monaco;
}

export { monaco };
