// Küçük, bağımlılıksız bir LSP (JSON-RPC) istemcisi.
// WebSocket text mesajı = tek JSON-RPC mesajı (Content-Length yok), bkz. app/API.md.

export type LspStatus = 'connecting' | 'indexing' | 'ready' | 'disconnected';

export interface LspPosition {
  line: number;
  character: number;
}
export interface LspRange {
  start: LspPosition;
  end: LspPosition;
}
export interface LspDiagnostic {
  range: LspRange;
  severity?: 1 | 2 | 3 | 4;
  code?: string | number;
  source?: string;
  message: string;
  tags?: number[];
}

interface JsonRpcMessage {
  jsonrpc: '2.0';
  id?: number | string;
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: { code: number; message: string };
}

interface Pending {
  resolve: (v: unknown) => void;
  reject: (e: Error) => void;
  timer: ReturnType<typeof setTimeout>;
}

export interface LspClientOptions {
  url: string;
  rootUri: string;
  onStatus: (status: LspStatus, detail?: string) => void;
  onDiagnostics: (uri: string, diagnostics: LspDiagnostic[]) => void;
}

const REQUEST_TIMEOUT_MS = 20_000;

export class LspClient {
  private ws: WebSocket | null = null;
  private nextId = 1;
  private pending = new Map<number, Pending>();
  private docs = new Map<string, { text: string; version: number }>();
  private initialized = false;
  private disposed = false;
  private progress = new Set<string>();
  private sawServerStatus = false;
  private readyTimer: ReturnType<typeof setTimeout> | null = null;
  private opts: LspClientOptions;
  status: LspStatus = 'connecting';
  serverCapabilities: Record<string, unknown> = {};

  constructor(opts: LspClientOptions) {
    this.opts = opts;
  }

  start(): void {
    if (this.disposed) return;
    this.initialized = false;
    this.progress.clear();
    this.sawServerStatus = false;
    this.setStatus('connecting');
    let ws: WebSocket;
    try {
      ws = new WebSocket(this.opts.url);
    } catch {
      this.setStatus('disconnected');
      return;
    }
    this.ws = ws;
    ws.onopen = () => void this.handshake();
    ws.onmessage = (ev) => {
      if (typeof ev.data === 'string') this.onMessage(ev.data);
    };
    ws.onclose = () => this.onClosed(ws);
    ws.onerror = () => {
      /* onclose da tetiklenir */
    };
  }

  dispose(): void {
    this.disposed = true;
    if (this.readyTimer) clearTimeout(this.readyTimer);
    const ws = this.ws;
    this.ws = null;
    if (ws) {
      try {
        if (this.initialized && ws.readyState === WebSocket.OPEN) {
          this.sendRaw({ jsonrpc: '2.0', id: this.nextId++, method: 'shutdown' });
          this.sendRaw({ jsonrpc: '2.0', method: 'exit' });
        }
        ws.close();
      } catch {
        /* yok say */
      }
    }
    this.rejectAll('Bağlantı kapatıldı');
  }

  // ---- belgeler -------------------------------------------------------------------------------

  /** Belgeyi kaydeder veya içeriğini günceller (tam senkronizasyon). */
  setDocument(uri: string, text: string): void {
    const existing = this.docs.get(uri);
    if (!existing) {
      this.docs.set(uri, { text, version: 1 });
      if (this.initialized) this.sendOpen(uri);
      return;
    }
    if (existing.text === text) return;
    existing.text = text;
    existing.version += 1;
    if (this.initialized) {
      this.notify('textDocument/didChange', {
        textDocument: { uri, version: existing.version },
        contentChanges: [{ text }],
      });
    }
  }

  closeDocument(uri: string): void {
    if (!this.docs.delete(uri)) return;
    if (this.initialized) this.notify('textDocument/didClose', { textDocument: { uri } });
  }

  hasDocument(uri: string): boolean {
    return this.docs.has(uri);
  }

  get isReady(): boolean {
    return this.initialized && this.ws?.readyState === WebSocket.OPEN;
  }

  // ---- JSON-RPC ----------------------------------------------------------------------------------

  request<T>(method: string, params: unknown, signal?: { onCancel: (cb: () => void) => void }): Promise<T | null> {
    if (!this.isReady) return Promise.resolve(null);
    const id = this.nextId++;
    return new Promise<T | null>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        resolve(null);
      }, REQUEST_TIMEOUT_MS);
      this.pending.set(id, { resolve: resolve as (v: unknown) => void, reject, timer });
      signal?.onCancel(() => {
        if (this.pending.delete(id)) {
          clearTimeout(timer);
          this.notify('$/cancelRequest', { id });
          resolve(null);
        }
      });
      this.sendRaw({ jsonrpc: '2.0', id, method, params });
    });
  }

  notify(method: string, params: unknown): void {
    this.sendRaw({ jsonrpc: '2.0', method, params });
  }

  private sendRaw(msg: JsonRpcMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) this.ws.send(JSON.stringify(msg));
  }

  private rawRequest(method: string, params: unknown): Promise<unknown> {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`${method} zaman aşımına uğradı`));
      }, 60_000);
      this.pending.set(id, { resolve, reject, timer });
      this.sendRaw({ jsonrpc: '2.0', id, method, params });
    });
  }

  private async handshake(): Promise<void> {
    try {
      const res = (await this.rawRequest('initialize', {
        processId: null,
        clientInfo: { name: 'rust-ogren-web' },
        locale: 'tr',
        rootUri: this.opts.rootUri,
        workspaceFolders: [{ uri: this.opts.rootUri, name: 'workspace' }],
        capabilities: {
          general: { positionEncodings: ['utf-16'] },
          window: { workDoneProgress: true },
          workspace: { configuration: true, workspaceFolders: true },
          experimental: { serverStatusNotification: true },
          textDocument: {
            synchronization: { dynamicRegistration: false, didSave: false },
            publishDiagnostics: { relatedInformation: false, tagSupport: { valueSet: [1, 2] } },
            hover: { contentFormat: ['markdown', 'plaintext'] },
            completion: {
              contextSupport: true,
              completionItem: {
                snippetSupport: true,
                insertReplaceSupport: true,
                labelDetailsSupport: true,
                deprecatedSupport: true,
                tagSupport: { valueSet: [1] },
                documentationFormat: ['markdown', 'plaintext'],
              },
            },
            signatureHelp: {
              contextSupport: true,
              signatureInformation: {
                documentationFormat: ['markdown', 'plaintext'],
                parameterInformation: { labelOffsetSupport: true },
                activeParameterSupport: true,
              },
            },
            definition: { linkSupport: true },
            formatting: {},
          },
        },
      })) as { capabilities?: Record<string, unknown> } | null;
      this.serverCapabilities = res?.capabilities ?? {};
      this.notify('initialized', {});
      this.initialized = true;
      for (const uri of this.docs.keys()) this.sendOpen(uri);
      this.setStatus('indexing');
      // İlerleme/serverStatus bildirimi hiç gelmezse 4 sn sonra hazır say.
      this.readyTimer = setTimeout(() => {
        if (!this.sawServerStatus && this.progress.size === 0 && this.status === 'indexing') this.setStatus('ready');
      }, 4000);
    } catch {
      this.setStatus('disconnected', 'rust-analyzer başlatılamadı');
      this.ws?.close();
    }
  }

  private sendOpen(uri: string): void {
    const d = this.docs.get(uri);
    if (!d) return;
    this.notify('textDocument/didOpen', {
      textDocument: { uri, languageId: 'rust', version: d.version, text: d.text },
    });
  }

  private onMessage(data: string): void {
    let msg: JsonRpcMessage;
    try {
      msg = JSON.parse(data) as JsonRpcMessage;
    } catch {
      return;
    }
    if (msg.method !== undefined && msg.id !== undefined) {
      this.onServerRequest(msg);
    } else if (msg.method !== undefined) {
      this.onNotification(msg.method, msg.params);
    } else if (typeof msg.id === 'number') {
      const p = this.pending.get(msg.id);
      if (!p) return;
      this.pending.delete(msg.id);
      clearTimeout(p.timer);
      if (msg.error) p.reject(new Error(msg.error.message));
      else p.resolve(msg.result ?? null);
    }
  }

  private onServerRequest(msg: JsonRpcMessage): void {
    switch (msg.method) {
      case 'workspace/configuration': {
        const items = (msg.params as { items?: unknown[] } | undefined)?.items ?? [];
        this.sendRaw({ jsonrpc: '2.0', id: msg.id, result: items.map(() => null) });
        break;
      }
      case 'window/workDoneProgress/create':
      case 'client/registerCapability':
      case 'client/unregisterCapability':
      case 'workspace/diagnostic/refresh':
      case 'workspace/inlayHint/refresh':
      case 'workspace/semanticTokens/refresh':
      case 'workspace/codeLens/refresh':
        this.sendRaw({ jsonrpc: '2.0', id: msg.id, result: null });
        break;
      case 'workspace/workspaceFolders':
        this.sendRaw({
          jsonrpc: '2.0',
          id: msg.id,
          result: [{ uri: this.opts.rootUri, name: 'workspace' }],
        });
        break;
      default:
        this.sendRaw({ jsonrpc: '2.0', id: msg.id, error: { code: -32601, message: 'Method not found' } });
    }
  }

  private onNotification(method: string, params: unknown): void {
    if (method === 'textDocument/publishDiagnostics') {
      const p = params as { uri: string; diagnostics: LspDiagnostic[] };
      this.opts.onDiagnostics(p.uri, p.diagnostics ?? []);
    } else if (method === 'experimental/serverStatus') {
      const p = params as { health?: string; quiescent?: boolean; message?: string };
      this.sawServerStatus = true;
      if (p.quiescent) this.setStatus('ready', p.health === 'error' ? p.message : undefined);
      else this.setStatus('indexing', p.message);
    } else if (method === '$/progress') {
      const p = params as { token: string | number; value?: { kind?: string } };
      const token = String(p.token);
      if (p.value?.kind === 'begin') this.progress.add(token);
      else if (p.value?.kind === 'end') this.progress.delete(token);
      if (!this.sawServerStatus && this.initialized) {
        this.setStatus(this.progress.size > 0 ? 'indexing' : 'ready');
      }
    }
  }

  private onClosed(ws: WebSocket): void {
    if (this.ws !== ws && this.ws !== null) return;
    this.initialized = false;
    this.rejectAll('Bağlantı koptu');
    if (!this.disposed) this.setStatus('disconnected');
  }

  private rejectAll(reason: string): void {
    for (const p of this.pending.values()) {
      clearTimeout(p.timer);
      p.reject(new Error(reason));
    }
    this.pending.clear();
  }

  private setStatus(status: LspStatus, detail?: string): void {
    this.status = status;
    if (!this.disposed) this.opts.onStatus(status, detail);
  }
}
