// LSP <-> Monaco köprüsü: hover, completion, signature help, definition, formatting ve
// publishDiagnostics -> marker (owner: "rust-analyzer").
import type * as Monaco from 'monaco-editor/editor/editor.api.js';
import { LspClient } from './lsp';
import type { LspDiagnostic, LspPosition, LspRange, LspStatus } from './lsp';

type M = typeof Monaco;

export const LSP_MARKER_OWNER = 'rust-analyzer';

interface MarkupContent {
  kind: 'markdown' | 'plaintext';
  value: string;
}
type MarkedString = string | { language: string; value: string };
type LspDocumentation = string | MarkupContent | undefined;

const toLspPos = (p: Monaco.IPosition): LspPosition => ({ line: p.lineNumber - 1, character: p.column - 1 });
const toRange = (r: LspRange): Monaco.IRange => ({
  startLineNumber: r.start.line + 1,
  startColumn: r.start.character + 1,
  endLineNumber: r.end.line + 1,
  endColumn: r.end.character + 1,
});

function toMarkdown(doc: LspDocumentation): Monaco.IMarkdownString | string | undefined {
  if (!doc) return undefined;
  if (typeof doc === 'string') return doc;
  return doc.kind === 'markdown' ? { value: doc.value } : doc.value;
}

function hoverContents(c: MarkupContent | MarkedString | MarkedString[]): Monaco.IMarkdownString[] {
  const one = (x: MarkupContent | MarkedString): Monaco.IMarkdownString => {
    if (typeof x === 'string') return { value: x };
    if ('kind' in x) return { value: x.value };
    return { value: '```' + x.language + '\n' + x.value + '\n```' };
  };
  return Array.isArray(c) ? c.map(one) : [one(c)];
}

const CANCEL = (token: Monaco.CancellationToken) => ({
  onCancel: (cb: () => void) => {
    token.onCancellationRequested(cb);
  },
});

interface LspCompletionItem {
  label: string;
  labelDetails?: { detail?: string; description?: string };
  kind?: number;
  tags?: number[];
  detail?: string;
  documentation?: LspDocumentation;
  deprecated?: boolean;
  preselect?: boolean;
  sortText?: string;
  filterText?: string;
  insertText?: string;
  insertTextFormat?: 1 | 2;
  textEdit?: { newText: string; range?: LspRange; insert?: LspRange; replace?: LspRange };
  additionalTextEdits?: { range: LspRange; newText: string }[];
  commitCharacters?: string[];
}

export class LspSession {
  private monaco: M;
  private client: LspClient;
  private models = new Map<string, Monaco.editor.ITextModel>(); // lspUri -> model
  private lspUriByModel = new Map<string, string>(); // model.uri.toString() -> lspUri
  private listeners = new Map<string, Monaco.IDisposable>();
  private providers: Monaco.IDisposable[] = [];
  private diagnostics = new Map<string, LspDiagnostic[]>();

  constructor(
    monaco: M,
    opts: { url: string; rootUri: string; onStatus: (s: LspStatus, detail?: string) => void },
  ) {
    this.monaco = monaco;
    this.client = new LspClient({
      url: opts.url,
      rootUri: opts.rootUri,
      onStatus: opts.onStatus,
      onDiagnostics: (uri, diags) => this.applyDiagnostics(uri, diags),
    });
    this.registerProviders();
  }

  start(): void {
    this.client.start();
  }

  /** Bağlantı koptuktan sonra yeniden bağlanır; açık belgeler yeniden gönderilir. */
  reconnect(): void {
    this.client.start();
  }

  addModel(lspUri: string, model: Monaco.editor.ITextModel): void {
    this.models.set(lspUri, model);
    this.lspUriByModel.set(model.uri.toString(), lspUri);
    this.client.setDocument(lspUri, model.getValue());
    this.listeners.set(
      lspUri,
      model.onDidChangeContent(() => this.client.setDocument(lspUri, model.getValue())),
    );
    const pending = this.diagnostics.get(lspUri);
    if (pending) this.applyDiagnostics(lspUri, pending);
  }

  removeModel(lspUri: string): void {
    const model = this.models.get(lspUri);
    if (model) {
      this.lspUriByModel.delete(model.uri.toString());
      this.monaco.editor.setModelMarkers(model, LSP_MARKER_OWNER, []);
    }
    this.listeners.get(lspUri)?.dispose();
    this.listeners.delete(lspUri);
    this.models.delete(lspUri);
    this.client.closeDocument(lspUri);
  }

  dispose(): void {
    for (const uri of [...this.models.keys()]) this.removeModel(uri);
    this.providers.forEach((p) => p.dispose());
    this.providers = [];
    this.client.dispose();
  }

  // ---- teşhisler ------------------------------------------------------------------------------

  private applyDiagnostics(uri: string, diags: LspDiagnostic[]): void {
    this.diagnostics.set(uri, diags);
    const model = this.models.get(uri);
    if (!model) return;
    const sev = this.monaco.MarkerSeverity;
    // Gizli test kodu diskte görünen içeriğin sonuna eklenir; flycheck satır numaraları disk
    // üzerindedir. Modelin son satırından sonra başlayan teşhisler atılır (son satıra sıkıştırılmaz).
    const lineCount = model.getLineCount();
    const visible = diags.filter((d) => d.range.start.line + 1 <= lineCount);
    const markers: Monaco.editor.IMarkerData[] = visible.map((d) => ({
      ...toRange(d.range),
      message: d.message,
      source: d.source ?? 'rust-analyzer',
      code: d.code !== undefined ? String(d.code) : undefined,
      severity: d.severity === 1 ? sev.Error : d.severity === 2 ? sev.Warning : d.severity === 3 ? sev.Info : sev.Hint,
      tags: d.tags?.map((t) => (t === 1 ? this.monaco.MarkerTag.Unnecessary : this.monaco.MarkerTag.Deprecated)),
    }));
    this.monaco.editor.setModelMarkers(model, LSP_MARKER_OWNER, markers);
  }

  // ---- sağlayıcılar ---------------------------------------------------------------------------

  private uriOf(model: Monaco.editor.ITextModel): string | null {
    const uri = this.lspUriByModel.get(model.uri.toString());
    return uri && this.client.isReady ? uri : null;
  }

  private async req<T>(method: string, params: unknown, token: Monaco.CancellationToken): Promise<T | null> {
    try {
      return await this.client.request<T>(method, params, CANCEL(token));
    } catch {
      return null;
    }
  }

  private registerProviders(): void {
    const languages = this.monaco.languages;
    const K = languages.CompletionItemKind;
    const kindMap: Record<number, Monaco.languages.CompletionItemKind> = {
      1: K.Text, 2: K.Method, 3: K.Function, 4: K.Constructor, 5: K.Field, 6: K.Variable, 7: K.Class,
      8: K.Interface, 9: K.Module, 10: K.Property, 11: K.Unit, 12: K.Value, 13: K.Enum, 14: K.Keyword,
      15: K.Snippet, 16: K.Color, 17: K.File, 18: K.Reference, 19: K.Folder, 20: K.EnumMember,
      21: K.Constant, 22: K.Struct, 23: K.Event, 24: K.Operator, 25: K.TypeParameter,
    };

    this.providers.push(
      languages.registerHoverProvider('rust', {
        provideHover: async (model, position, token) => {
          const uri = this.uriOf(model);
          if (!uri) return null;
          const res = await this.req<{ contents: MarkupContent | MarkedString | MarkedString[]; range?: LspRange }>(
            'textDocument/hover',
            { textDocument: { uri }, position: toLspPos(position) },
            token,
          );
          if (!res?.contents) return null;
          return { contents: hoverContents(res.contents), range: res.range ? toRange(res.range) : undefined };
        },
      }),

      languages.registerCompletionItemProvider('rust', {
        triggerCharacters: ['.', ':', '<', '"', '/', '('],
        provideCompletionItems: async (model, position, context, token) => {
          const uri = this.uriOf(model);
          if (!uri) return null;
          const triggerChar = context.triggerKind === languages.CompletionTriggerKind.TriggerCharacter;
          const res = await this.req<LspCompletionItem[] | { items: LspCompletionItem[]; isIncomplete?: boolean }>(
            'textDocument/completion',
            {
              textDocument: { uri },
              position: toLspPos(position),
              context: triggerChar
                ? { triggerKind: 2, triggerCharacter: context.triggerCharacter }
                : { triggerKind: 1 },
            },
            token,
          );
          if (!res) return null;
          const items = Array.isArray(res) ? res : res.items;
          const incomplete = !Array.isArray(res) && !!res.isIncomplete;
          const word = model.getWordUntilPosition(position);
          const defaultRange: Monaco.IRange = {
            startLineNumber: position.lineNumber,
            endLineNumber: position.lineNumber,
            startColumn: word.startColumn,
            endColumn: word.endColumn,
          };
          const suggestions: Monaco.languages.CompletionItem[] = items.map((i) => {
            const te = i.textEdit;
            let range: Monaco.languages.CompletionItem['range'] = defaultRange;
            if (te?.range) range = toRange(te.range);
            else if (te?.insert && te.replace) range = { insert: toRange(te.insert), replace: toRange(te.replace) };
            return {
              label: i.labelDetails
                ? { label: i.label, detail: i.labelDetails.detail, description: i.labelDetails.description }
                : i.label,
              kind: kindMap[i.kind ?? 1] ?? K.Text,
              detail: i.detail,
              documentation: toMarkdown(i.documentation),
              sortText: i.sortText,
              filterText: i.filterText,
              preselect: i.preselect,
              commitCharacters: i.commitCharacters,
              tags: i.tags?.includes(1) || i.deprecated ? [languages.CompletionItemTag.Deprecated] : undefined,
              insertText: te?.newText ?? i.insertText ?? i.label,
              insertTextRules:
                i.insertTextFormat === 2 ? languages.CompletionItemInsertTextRule.InsertAsSnippet : undefined,
              range,
              additionalTextEdits: i.additionalTextEdits?.map((e) => ({ range: toRange(e.range), text: e.newText })),
            };
          });
          return { suggestions, incomplete };
        },
      }),

      languages.registerSignatureHelpProvider('rust', {
        signatureHelpTriggerCharacters: ['(', ',', '<'],
        provideSignatureHelp: async (model, position, token, context) => {
          const uri = this.uriOf(model);
          if (!uri) return null;
          const res = await this.req<Monaco.languages.SignatureHelp>(
            'textDocument/signatureHelp',
            {
              textDocument: { uri },
              position: toLspPos(position),
              context: {
                triggerKind: context.triggerKind === languages.SignatureHelpTriggerKind.TriggerCharacter ? 2 : 1,
                triggerCharacter: context.triggerCharacter,
                isRetrigger: context.isRetrigger,
              },
            },
            token,
          );
          if (!res?.signatures?.length) return null;
          return {
            value: {
              signatures: res.signatures.map((s) => ({
                ...s,
                documentation: toMarkdown(s.documentation as LspDocumentation),
                parameters: (s.parameters ?? []).map((p) => ({
                  ...p,
                  documentation: toMarkdown(p.documentation as LspDocumentation),
                })),
              })),
              activeSignature: res.activeSignature ?? 0,
              activeParameter: res.activeParameter ?? 0,
            },
            dispose: () => {},
          };
        },
      }),

      languages.registerDefinitionProvider('rust', {
        provideDefinition: async (model, position, token) => {
          const uri = this.uriOf(model);
          if (!uri) return null;
          type Loc = { uri: string; range: LspRange };
          type Link = { targetUri: string; targetSelectionRange: LspRange };
          const res = await this.req<Loc | (Loc | Link)[]>(
            'textDocument/definition',
            { textDocument: { uri }, position: toLspPos(position) },
            token,
          );
          if (!res) return null;
          const out: Monaco.languages.Location[] = [];
          for (const l of Array.isArray(res) ? res : [res]) {
            const target = 'targetUri' in l ? l.targetUri : l.uri;
            const range = 'targetUri' in l ? l.targetSelectionRange : l.range;
            // Yalnızca açık (egzersize ait, listelenen) dosyalara ve görünen satırlara gidilebilir;
            // tests/ gibi listelenmeyen dosyalar ve gizli test satırları atılır.
            const targetModel = this.models.get(target);
            if (!targetModel) continue;
            if (range.start.line + 1 > targetModel.getLineCount()) continue;
            out.push({ uri: targetModel.uri, range: toRange(range) });
          }
          return out;
        },
      }),

      languages.registerDocumentFormattingEditProvider('rust', {
        displayName: 'rustfmt',
        provideDocumentFormattingEdits: async (model, options, token) => {
          const uri = this.uriOf(model);
          if (!uri) return null;
          const res = await this.req<{ range: LspRange; newText: string }[]>(
            'textDocument/formatting',
            { textDocument: { uri }, options: { tabSize: options.tabSize, insertSpaces: options.insertSpaces } },
            token,
          );
          return res?.map((e) => ({ range: toRange(e.range), text: e.newText })) ?? null;
        },
      }),
    );
  }
}
