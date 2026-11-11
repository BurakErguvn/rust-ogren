#!/usr/bin/env node
// Yalnızca geliştirme için sahte (mock) API. app/API.md sözleşmesini izler.
// Gerçek axum backend'i çalışırken KULLANMA (aynı port: 3000).
//
//   node scripts/mock-api.mjs            # :3000
//   MOCK_PORT=3100 MOCK_WORKSPACE=/tmp/ws node scripts/mock-api.mjs
//
// - İçerik: tr/book (varsa) > scripts/fixtures > book/src
// - Egzersiz: gerçek bir çalışma kopyası oluşturur ve gerçek `cargo test` çalıştırır.
// - app/web/out varsa statik dosyaları da sunar (üretim derlemesini denemek için).
// - LSP: gerçek `rust-analyzer` süreci, elle yazılmış küçük bir WebSocket köprüsüyle.
import http from 'node:http';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { spawn, execFile } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(here, '../../..');
const PORT = Number(process.env.MOCK_PORT ?? 3000);
const WS_DIR = path.resolve(process.env.MOCK_WORKSPACE ?? path.join(os.tmpdir(), 'rust-ogren-mock-ws'));
const FIXTURES = path.join(here, 'fixtures');
const STATE_FILE = path.join(WS_DIR, '.mock-progress.json');

fs.mkdirSync(WS_DIR, { recursive: true });

// ---------------------------------------------------------------- içindekiler

function readIfExists(...candidates) {
  for (const c of candidates) if (fs.existsSync(c)) return fs.readFileSync(c, 'utf8');
  return null;
}

function pageSource(id) {
  const tr = readIfExists(path.join(ROOT, 'tr/book', `${id}.md`), path.join(FIXTURES, `${id.replaceAll('/', '__')}.md`));
  if (tr !== null) return { md: tr, translated: true };
  return { md: fs.readFileSync(path.join(ROOT, 'book/src', `${id}.md`), 'utf8'), translated: false };
}

function parseSummary() {
  const text = fs.readFileSync(path.join(ROOT, 'book/src/SUMMARY.md'), 'utf8');
  const chapters = [];
  for (const line of text.split('\n')) {
    const m = /^(\s*)[-*]\s+\[([^\]]*)\]\(([^)]+)\.md\)/.exec(line);
    if (!m) continue;
    const page = { id: m[3], summaryTitle: m[2] };
    if (m[1].length === 0) chapters.push({ id: m[3].split('/')[0], pages: [page] });
    else chapters.at(-1).pages.push(page);
  }
  return chapters;
}

function pageTitle(p) {
  const { md, translated } = pageSource(p.id);
  if (!translated) return p.summaryTitle;
  const m = /^#\s+(.+?)\s*$/m.exec(md);
  return m ? m[1].replace(/\s*\{#[^}]*\}\s*$/, '') : p.summaryTitle;
}

// tr/removed-exercises.json (+ MOCK_REMOVED=a/b,c/d) içindekiler uygulamada yoktur.
const removed = new Set(
  (() => {
    try {
      return JSON.parse(fs.readFileSync(path.join(ROOT, 'tr/removed-exercises.json'), 'utf8')).removed ?? [];
    } catch {
      return [];
    }
  })().concat((process.env.MOCK_REMOVED ?? '').split(',').filter(Boolean)),
);
const hasExercise = (id) => !removed.has(id) && fs.existsSync(path.join(ROOT, 'exercises', id));

let progress = {};
try {
  progress = JSON.parse(fs.readFileSync(STATE_FILE, 'utf8'));
} catch {
  /* yeni */
}
const saveProgress = () => fs.writeFileSync(STATE_FILE, JSON.stringify(progress));
const getProgress = (id) => {
  const p = progress[id] ?? { completed: false, score: null, failed_submits: 0 };
  return { ...p };
};

function buildToc() {
  const chapters = parseSummary().map((c) => ({
    id: c.id,
    title: pageTitle(c.pages[0]),
    pages: c.pages.map((p) => ({
      id: p.id,
      title: pageTitle(p),
      translated: pageSource(p.id).translated,
      exercise: hasExercise(p.id) ? getProgress(p.id) : null,
    })),
  }));
  const all = chapters.flatMap((c) => c.pages).filter((p) => p.exercise);
  return {
    chapters,
    progress: {
      completed: all.filter((p) => p.exercise.completed).length,
      total: all.length,
      score_sum: all.reduce((s, p) => s + (p.exercise.score ?? 0), 0),
    },
  };
}

// ---------------------------------------------------------------- çalışma kopyası

function copyDir(src, dst) {
  fs.mkdirSync(dst, { recursive: true });
  for (const e of fs.readdirSync(src, { withFileTypes: true })) {
    if (e.name === 'target') continue;
    const s = path.join(src, e.name);
    const d = path.join(dst, e.name);
    if (e.isDirectory()) copyDir(s, d);
    else fs.copyFileSync(s, d);
  }
}

const DEFAULT_WS_TOML = `[workspace]\nmembers = ["exercises/*/*", "helpers/*"]\nresolver = "2"\n\n[profile.dev.package.copy]\noverflow-checks = true\n`;
// Cargo.toml düzenlemesi isteyen egzersizler (id -> kök Cargo.toml da düzenlenir mi?)
const TOML_EXERCISES = { '02_basic_calculator/08_overflow': true, '05_ticket_v2/11_dependencies': false };

function ensureWorkspace() {
  const cargo = path.join(WS_DIR, 'Cargo.toml');
  if (fs.existsSync(cargo)) return;
  fs.writeFileSync(cargo, DEFAULT_WS_TOML);
  if (fs.existsSync(path.join(ROOT, 'Cargo.lock'))) fs.copyFileSync(path.join(ROOT, 'Cargo.lock'), path.join(WS_DIR, 'Cargo.lock'));
  copyDir(path.join(ROOT, 'helpers'), path.join(WS_DIR, 'helpers'));
}

const exDir = (id) => path.join(WS_DIR, 'exercises', id);

function createExercise(id, force = false) {
  ensureWorkspace();
  const dst = exDir(id);
  if (fs.existsSync(dst) && !force) return;
  fs.rmSync(dst, { recursive: true, force: true });
  copyDir(path.join(ROOT, 'exercises', id), dst);
  const tr = path.join(ROOT, 'tr/exercises', id);
  if (fs.existsSync(tr)) copyDir(tr, dst); // yalnızca .rs dosyaları üzerine yazar
}

// Gizli test kodu: dosyadaki ilk satır başı `#[cfg(test)]` ile sonrası. Yer tutucu (todo!/TODO/__) varsa görünür kalır.
const PLACEHOLDER = /todo!\(|\bTODO\b|\b__\b/;
function splitHidden(content) {
  const m = /^#\[cfg\(test\)\]/m.exec(content);
  if (!m) return { visible: content, hidden: '' };
  const hidden = content.slice(m.index);
  if (PLACEHOLDER.test(hidden)) return { visible: content, hidden: '' };
  return { visible: content.slice(0, m.index), hidden };
}

function diskFiles(id) {
  const base = exDir(id);
  const out = [];
  const walk = (dir) => {
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
      const p = path.join(dir, e.name);
      if (e.isDirectory()) walk(p);
      else if (e.name.endsWith('.rs')) out.push(path.relative(base, p).split(path.sep).join('/'));
    }
  };
  for (const top of ['src', 'tests']) if (fs.existsSync(path.join(base, top))) walk(path.join(base, top));
  const rank = (p) => (p === 'src/lib.rs' ? 0 : p === 'src/main.rs' ? 1 : 2);
  out.sort((a, b) => rank(a) - rank(b) || a.localeCompare(b));
  return out;
}

/** Görünür dosyalar + gizli dosya sayısı. */
function listFiles(id) {
  const base = exDir(id);
  const files = [];
  let hiddenFiles = 0;
  for (const p of diskFiles(id)) {
    const content = fs.readFileSync(path.join(base, p), 'utf8');
    if (p.startsWith('tests/')) {
      if (PLACEHOLDER.test(content)) files.push({ path: p, content, editable: true, language: 'rust', has_hidden_tests: false });
      else hiddenFiles += 1;
      continue;
    }
    const { visible, hidden } = splitHidden(content);
    files.push({ path: p, content: visible, editable: true, language: 'rust', has_hidden_tests: hidden !== '' });
  }
  if (id in TOML_EXERCISES) {
    files.push({ path: 'Cargo.toml', content: fs.readFileSync(path.join(base, 'Cargo.toml'), 'utf8'), editable: true, language: 'toml', has_hidden_tests: false });
    if (TOML_EXERCISES[id]) {
      files.push({ path: '@workspace/Cargo.toml', content: fs.readFileSync(path.join(WS_DIR, 'Cargo.toml'), 'utf8'), editable: true, language: 'toml', has_hidden_tests: false });
    }
  }
  return { files, hiddenFiles };
}

/** Çalışma kopyasında orijinalde bulunmayan (öğrencinin oluşturduğu) dosya mı? */
const isLearnerFile = (id, rel) => !fs.existsSync(path.join(ROOT, 'exercises', id, rel)) && !fs.existsSync(path.join(ROOT, 'tr/exercises', id, rel));

/** Diskteki dosyanın gizli kısmını koruyarak görünür içeriği yazar. */
function writeVisible(id, rel, content) {
  const f = path.join(exDir(id), rel);
  fs.mkdirSync(path.dirname(f), { recursive: true });
  const hidden = fs.existsSync(f) ? splitHidden(fs.readFileSync(f, 'utf8')).hidden : '';
  fs.writeFileSync(f, content + hidden);
}

function filesResponse(id) {
  createExercise(id);
  const { files, hiddenFiles } = listFiles(id);
  return {
    id,
    workspace_root_uri: pathToFileURL(WS_DIR).href,
    exercise_root_uri: pathToFileURL(exDir(id)).href,
    files,
    hidden_test_files: hiddenFiles,
    progress: getProgress(id),
  };
}

/** Teşhis gizli koddaysa konum ve alıntıları sakla. */
function maskDiagnostic(id, d) {
  const f = path.join(exDir(id), d.file);
  let hidden = d.file.startsWith('tests/') && !PLACEHOLDER.test(fs.readFileSync(f, 'utf8'));
  if (!hidden && fs.existsSync(f)) {
    const { visible } = splitHidden(fs.readFileSync(f, 'utf8'));
    const full = fs.readFileSync(f, 'utf8');
    if (visible !== full && d.line > visible.split('\n').length - 1) hidden = true;
  }
  if (!hidden) return d;
  const rendered = (d.rendered ?? '')
    .split('\n')
    .filter((l) => /^(error|warning)/.test(l) || /^\s*= (note|help)/.test(l))
    .join('\n');
  return { ...d, line: null, column: null, end_line: null, end_column: null, rendered, hidden: true };
}

// ---------------------------------------------------------------- test / submit

let busy = false;

function run(cmd, args, cwd, timeoutMs) {
  return new Promise((resolve) => {
    execFile(cmd, args, { cwd, timeout: timeoutMs, maxBuffer: 32 * 1024 * 1024, env: { ...process.env, CARGO_TERM_COLOR: 'never' } }, (err, stdout, stderr) => {
      resolve({ err, stdout, stderr, timedOut: !!err?.killed });
    });
  });
}

async function runTests(id, kind) {
  if (process.env.MOCK_DELAY_MS) await new Promise((r) => setTimeout(r, Number(process.env.MOCK_DELAY_MS)));
  createExercise(id);
  const dir = exDir(id);
  const started = Date.now();
  const build = await run('cargo', ['test', '--no-run', '--message-format=json', '--offline'], dir, 60_000);
  const diagnostics = [];
  for (const line of build.stdout.split('\n')) {
    if (!line.startsWith('{')) continue;
    let j;
    try {
      j = JSON.parse(line);
    } catch {
      continue;
    }
    if (j.reason !== 'compiler-message' || !j.message) continue;
    const span = j.message.spans?.find((s) => s.is_primary);
    if (!span) continue;
    const rel = path.isAbsolute(span.file_name) ? path.relative(dir, span.file_name) : path.relative(dir, path.join(WS_DIR, span.file_name));
    if (!/^(src|tests)\//.test(rel)) continue;
    const raw = {
      file: rel.split(path.sep).join('/'),
      line: span.line_start,
      column: span.column_start,
      end_line: span.line_end,
      end_column: span.column_end,
      severity: j.message.level === 'error' ? 'error' : j.message.level === 'warning' ? 'warning' : 'info',
      message: j.message.message,
      rendered: j.message.rendered ?? undefined,
    };
    diagnostics.push(maskDiagnostic(id, raw));
  }
  const result = { kind, tests: [], tests_passed: 0, tests_total: 0, diagnostics, log: '', duration_ms: 0 };
  if (build.timedOut) {
    result.status = 'timeout';
    result.log = (build.stderr ?? '').slice(-8000);
  } else if (build.err && diagnostics.some((d) => d.severity === 'error')) {
    result.status = 'compile_error';
    result.log = build.stderr.slice(-60000);
  } else if (build.err) {
    result.status = 'compile_error';
    result.log = build.stderr.slice(-60000);
  } else {
    const t = await run('cargo', ['test', '--offline'], dir, 30_000);
    const out = `${t.stdout}\n${t.stderr}`;
    result.log = out.slice(-60000);
    const failures = {};
    const failBlock = /---- (\S+) stdout ----\n([\s\S]*?)(?=\n---- |\nfailures:\n|\ntest result:)/g;
    for (const m of out.matchAll(failBlock)) failures[m[1]] = m[2].trim();
    for (const m of out.matchAll(/^test (\S+) \.\.\. (ok|FAILED)/gm)) {
      result.tests.push({ name: m[1], passed: m[2] === 'ok', output: m[2] === 'ok' ? '' : (failures[m[1]] ?? '') });
    }
    result.tests_total = result.tests.length;
    result.tests_passed = result.tests.filter((x) => x.passed).length;
    const origHasTests = ['src', 'tests'].some((d) => {
      const walk = (dir) => fs.existsSync(dir) && fs.readdirSync(dir, { withFileTypes: true }).some((e) => (e.isDirectory() ? walk(path.join(dir, e.name)) : e.name.endsWith('.rs') && /#\[(tokio::)?test\]/.test(fs.readFileSync(path.join(dir, e.name), 'utf8'))));
      return walk(path.join(ROOT, 'exercises', id, d));
    });
    result.status = t.timedOut ? 'timeout' : result.tests_total > 0 ? (result.tests_passed === result.tests_total ? 'passed' : 'failed') : origHasTests ? 'failed' : 'passed';
  }
  result.duration_ms = Date.now() - started;
  if (kind === 'submit') {
    const p = progress[id] ?? { completed: false, score: null, failed_submits: 0 };
    if (!p.completed) {
      if (result.status === 'passed') {
        p.completed = true;
        p.score = Math.max(100 - 10 * p.failed_submits, 50);
      } else p.failed_submits += 1;
    }
    progress[id] = p;
    saveProgress();
  }
  result.progress = getProgress(id);
  return result;
}

// ---------------------------------------------------------------- HTTP

const send = (res, code, body) => {
  if (code === 204) {
    res.writeHead(204).end();
    return;
  }
  res.writeHead(code, { 'Content-Type': 'application/json; charset=utf-8' });
  res.end(JSON.stringify(body));
};
const readBody = (req) =>
  new Promise((resolve) => {
    const chunks = [];
    req.on('data', (c) => chunks.push(c));
    req.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')));
  });

// Üretim derlemesini (app/web/out) aynı porttan sunar, axum'un yaptığı gibi.
const OUT = path.resolve(here, '../out');
const MIME = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml', '.json': 'application/json', '.txt': 'text/plain', '.woff2': 'font/woff2', '.ttf': 'font/ttf', '.ico': 'image/x-icon' };
function serveStatic(p, res) {
  if (!fs.existsSync(OUT)) return false;
  let f = path.join(OUT, p);
  if (!f.startsWith(OUT)) return false;
  if (fs.existsSync(f) && fs.statSync(f).isDirectory()) f = path.join(f, 'index.html');
  if (!fs.existsSync(f)) {
    const nf = path.join(OUT, '404.html');
    res.writeHead(404, { 'Content-Type': MIME['.html'] }).end(fs.existsSync(nf) ? fs.readFileSync(nf) : 'Bulunamadı');
    return true;
  }
  res.writeHead(200, { 'Content-Type': MIME[path.extname(f)] ?? 'application/octet-stream' }).end(fs.readFileSync(f));
  return true;
}

async function handle(req, res) {
  const url = new URL(req.url, 'http://x');
  const p = decodeURIComponent(url.pathname).replace(/\/+$/, '');
  try {
    if (req.method === 'GET' && p === '/api/toc') return send(res, 200, buildToc());

    let m;
    if (req.method === 'GET' && (m = /^\/api\/pages\/(.+)$/.exec(p))) {
      const id = m[1];
      const flat = buildToc().chapters.flatMap((c) => c.pages);
      const idx = flat.findIndex((x) => x.id === id);
      if (idx < 0) return send(res, 404, { error: 'Sayfa bulunamadı.' });
      const src = pageSource(id);
      return send(res, 200, {
        id,
        title: flat[idx].title,
        translated: src.translated,
        markdown: src.md,
        prev: flat[idx - 1]?.id ?? null,
        next: flat[idx + 1]?.id ?? null,
        exercise: flat[idx].exercise,
        notes: (() => {
          try {
            return JSON.parse(fs.readFileSync(path.join(ROOT, 'tr/notes', `${id}.json`), 'utf8')).notes ?? [];
          } catch {
            return [];
          }
        })(),
      });
    }
    if (req.method === 'GET' && (m = /^\/api\/book-assets\/(.+)$/.exec(p))) {
      const f = path.join(ROOT, 'book/src', m[1]);
      if (!f.startsWith(path.join(ROOT, 'book/src')) || !fs.existsSync(f)) return send(res, 404, { error: 'Bulunamadı.' });
      res.writeHead(200).end(fs.readFileSync(f));
      return;
    }
    if (req.method === 'GET' && p === '/api/progress') {
      const t = buildToc();
      return send(res, 200, { ...t.progress, chapters: [] });
    }
    if ((m = /^\/api\/exercises\/([^/]+\/[^/]+)\/(files|test|submit|reset)(?:\/(.+))?$/.exec(p))) {
      const [, id, action, rest] = m;
      if (!hasExercise(id)) return send(res, 404, { error: 'Egzersiz bulunamadı.' });
      if (req.method === 'GET' && action === 'files' && !rest) return send(res, 200, filesResponse(id));
      if (req.method === 'PUT' && action === 'files' && rest) {
        const tomlOk = rest === 'Cargo.toml' ? id in TOML_EXERCISES : rest === '@workspace/Cargo.toml' ? TOML_EXERCISES[id] === true : false;
        if ((!/^(src|tests)\/.+\.rs$/.test(rest) && !tomlOk) || rest.includes('..')) return send(res, 400, { error: 'Geçersiz dosya yolu.' });
        createExercise(id);
        const body = JSON.parse((await readBody(req)) || '{}');
        if (rest.endsWith('Cargo.toml')) {
          const lines = String(body.content ?? '').split('\n');
          const bad = lines.findIndex((l) => /^\s*\[[^\]]*$/.test(l));
          if (bad >= 0) return send(res, 422, { error: `Cargo.toml ayrıştırılamadı: TOML parse error at line ${bad + 1}, column 1\n  unclosed table, expected \`]\`` });
        }
        if (rest === '@workspace/Cargo.toml') fs.writeFileSync(path.join(WS_DIR, 'Cargo.toml'), body.content ?? '');
        else writeVisible(id, rest, body.content ?? '');
        return send(res, 204);
      }
      if (req.method === 'DELETE' && action === 'files' && rest) {
        if (!/^src\/.+\.rs$/.test(rest) || rest.includes('..')) return send(res, 400, { error: 'Geçersiz dosya yolu.' });
        if (!isLearnerFile(id, rest)) return send(res, 403, { error: 'Bu dosya egzersizin özgün dosyası; silinemez.' });
        fs.rmSync(path.join(exDir(id), rest), { force: true });
        return send(res, 204);
      }
      if (req.method === 'POST' && action === 'reset') {
        createExercise(id, true);
        if (TOML_EXERCISES[id]) fs.writeFileSync(path.join(WS_DIR, 'Cargo.toml'), DEFAULT_WS_TOML);
        return send(res, 200, filesResponse(id));
      }
      if (req.method === 'POST' && (action === 'test' || action === 'submit')) {
        if (busy) return send(res, 409, { error: 'Başka bir test çalışıyor, lütfen bekle.' });
        busy = true;
        try {
          return send(res, 200, await runTests(id, action));
        } finally {
          busy = false;
        }
      }
    }
    if (!p.startsWith('/api') && (req.method === 'GET' || req.method === 'HEAD') && serveStatic(p, res)) return;
    send(res, 404, { error: 'Bulunamadı.' });
  } catch (e) {
    console.error(e);
    send(res, 500, { error: `Sunucu hatası: ${e.message}` });
  }
}

const server = http.createServer((req, res) => void handle(req, res));

// ---------------------------------------------------------------- WS <-> rust-analyzer

server.on('upgrade', (req, socket) => {
  const url = new URL(req.url, 'http://x');
  const m = /^\/api\/lsp\/([^/]+\/[^/]+)\/?$/.exec(url.pathname);
  if (!m || !hasExercise(m[1]) || process.env.MOCK_NO_LSP) {
    socket.destroy();
    return;
  }
  createExercise(m[1]);
  const key = req.headers['sec-websocket-key'];
  const accept = crypto.createHash('sha1').update(`${key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11`).digest('base64');
  socket.write(`HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ${accept}\r\n\r\n`);

  const ra = spawn('rust-analyzer', [], { cwd: WS_DIR, stdio: ['pipe', 'pipe', 'inherit'], detached: true });
  ra.on('error', (e) => {
    console.error('rust-analyzer başlatılamadı:', e.message);
    socket.destroy();
  });
  const kill = () => {
    try {
      process.kill(-ra.pid, 'SIGKILL');
    } catch {
      /* zaten bitti */
    }
  };
  socket.on('close', kill);
  socket.on('error', kill);
  ra.on('exit', () => socket.destroy());

  const sendFrame = (opcode, payload) => {
    const len = payload.length;
    const header = len < 126 ? Buffer.from([0x80 | opcode, len]) : len < 65536 ? Buffer.from([0x80 | opcode, 126, len >> 8, len & 255]) : (() => {
      const b = Buffer.alloc(10);
      b[0] = 0x80 | opcode;
      b[1] = 127;
      b.writeBigUInt64BE(BigInt(len), 2);
      return b;
    })();
    if (!socket.destroyed) socket.write(Buffer.concat([header, payload]));
  };

  // stdout (Content-Length çerçeveli) -> WS text
  let outBuf = Buffer.alloc(0);
  ra.stdout.on('data', (chunk) => {
    outBuf = Buffer.concat([outBuf, chunk]);
    for (;;) {
      const sep = outBuf.indexOf('\r\n\r\n');
      if (sep < 0) return;
      const cl = /content-length:\s*(\d+)/i.exec(outBuf.subarray(0, sep).toString());
      if (!cl) {
        outBuf = outBuf.subarray(sep + 4);
        continue;
      }
      const len = Number(cl[1]);
      if (outBuf.length < sep + 4 + len) return;
      sendFrame(1, outBuf.subarray(sep + 4, sep + 4 + len));
      outBuf = outBuf.subarray(sep + 4 + len);
    }
  });

  // WS frame -> stdin
  let inBuf = Buffer.alloc(0);
  let fragments = [];
  socket.on('data', (chunk) => {
    inBuf = Buffer.concat([inBuf, chunk]);
    for (;;) {
      if (inBuf.length < 2) return;
      const fin = (inBuf[0] & 0x80) !== 0;
      const opcode = inBuf[0] & 0x0f;
      const masked = (inBuf[1] & 0x80) !== 0;
      let len = inBuf[1] & 0x7f;
      let off = 2;
      if (len === 126) {
        if (inBuf.length < 4) return;
        len = inBuf.readUInt16BE(2);
        off = 4;
      } else if (len === 127) {
        if (inBuf.length < 10) return;
        len = Number(inBuf.readBigUInt64BE(2));
        off = 10;
      }
      const maskKey = masked ? inBuf.subarray(off, off + 4) : null;
      if (masked) off += 4;
      if (inBuf.length < off + len) return;
      const payload = Buffer.from(inBuf.subarray(off, off + len));
      inBuf = inBuf.subarray(off + len);
      if (maskKey) for (let i = 0; i < payload.length; i++) payload[i] ^= maskKey[i % 4];
      if (opcode === 8) {
        socket.end();
        kill();
        return;
      }
      if (opcode === 9) {
        sendFrame(10, payload);
        continue;
      }
      if (opcode === 1 || opcode === 0 || opcode === 2) {
        fragments.push(payload);
        if (fin) {
          const msg = Buffer.concat(fragments);
          fragments = [];
          ra.stdin.write(`Content-Length: ${msg.length}\r\n\r\n`);
          ra.stdin.write(msg);
        }
      }
    }
  });
});

server.listen(PORT, () => {
  console.log(`mock API http://localhost:${PORT}  (çalışma kopyası: ${WS_DIR})`);
});
process.on('SIGINT', () => process.exit(0));
