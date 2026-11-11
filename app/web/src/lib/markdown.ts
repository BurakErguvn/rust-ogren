// Markdown yardımcıları: başlık kimlikleri, mdBook gizli satırları, bağlantı çözümleme.
import type { Root, RootContent, Heading, Text, Code, Parent } from 'mdast';
import type { Element, ElementContent, Root as HastRoot } from 'hast';

/** mdBook/GitHub biçiminde slug: küçük harf, harf/rakam/boşluk/-/_ dışındakiler silinir, boşluk -> '-'. */
export function slugify(text: string): string {
  return text
    .trim()
    .toLocaleLowerCase('en-US')
    .replace(/[^\p{L}\p{N}\s_-]/gu, '')
    .replace(/\s/g, '-');
}

function textOf(node: RootContent | Root): string {
  if ('value' in node && typeof node.value === 'string') return node.value;
  if ('children' in node) return (node.children as RootContent[]).map(textOf).join('');
  return '';
}

const SLUG_SUFFIX = /\s*\{#([^\s}]+)\}\s*$/;

/**
 * - `## Başlık {#slug}` -> görünür metinden `{#slug}` silinir, başlık kimliği `slug` olur.
 * - Sonek yoksa slug başlık metninden üretilir; tekrarlar `-1`, `-2` ... alır.
 */
export function remarkHeadingIds() {
  return (tree: Root) => {
    const used = new Map<string, number>();
    const walk = (node: Parent) => {
      for (const child of node.children) {
        if (child.type === 'heading') {
          const h = child as Heading;
          let explicit: string | null = null;
          const last = h.children[h.children.length - 1];
          if (last && last.type === 'text') {
            const m = SLUG_SUFFIX.exec((last as Text).value);
            if (m) {
              explicit = m[1];
              const stripped = (last as Text).value.replace(SLUG_SUFFIX, '');
              if (stripped === '') h.children.pop();
              else (last as Text).value = stripped;
            }
          }
          let id = explicit ?? slugify(textOf(h));
          if (!id) id = 'bolum';
          const n = used.get(id) ?? 0;
          used.set(id, n + 1);
          if (n > 0) id = `${id}-${n}`;
          h.data = { ...(h.data ?? {}), hProperties: { id } };
        } else if ('children' in child) {
          walk(child as Parent);
        }
      }
    };
    walk(tree);
  };
}

/**
 * Uygulama notlarını yer tutucu düğümler olarak ekler: `before` id'li başlığın hemen önüne,
 * `null` (veya bulunamayan id) ise ilk H1'den sonraya (H1 yoksa en başa). Düğüm, `data-note="<i>"` özellikli
 * boş bir `div` olur; Markdown bileşeni bunu not kutusuyla değiştirir. `remarkHeadingIds`'den sonra çalışmalıdır.
 */
export function remarkNotes(count: number, befores: (string | null)[]) {
  return (tree: Root) => {
    if (count === 0) return;
    const headingId = (n: RootContent): string | null => {
      const props = n.type === 'heading' ? (n.data as { hProperties?: { id?: string } } | undefined)?.hProperties : undefined;
      return props?.id ?? null;
    };
    const h1 = tree.children.findIndex((n) => n.type === 'heading' && (n as Heading).depth === 1);
    const topIndex = h1 + 1;
    const inserts: { at: number; i: number }[] = [];
    befores.forEach((before, i) => {
      const found = before === null ? -1 : tree.children.findIndex((n) => headingId(n) === before);
      inserts.push({ at: found >= 0 ? found : topIndex, i });
    });
    // Sondan başa ekle ki indeksler kaymasın; aynı konumdakiler özgün sırayı korusun.
    inserts.sort((a, b) => b.at - a.at || b.i - a.i);
    for (const { at, i } of inserts) {
      const node = { type: 'paragraph', children: [], data: { hName: 'div', hProperties: { dataNote: String(i) } } } as unknown as RootContent;
      tree.children.splice(at, 0, node);
    }
  };
}

/**
 * Kod bloğu işlemleri:
 * - Bilgi satırındaki `rust,ignore` gibi özellikleri atar (dil = `rust`).
 * - mdBook gizli satırları: (girintiden sonra) `# ` ile başlayan veya yalnızca `#` olan satırlar
 *   gösterilmez; `##` ile başlayan satırlar tek `#` olarak gösterilir.
 */
export function remarkCodeBlocks() {
  return (tree: Root) => {
    const walk = (node: Parent) => {
      for (const child of node.children) {
        if (child.type === 'code') {
          const c = child as Code;
          const lang = (c.lang ?? '').split(/[,\s{]/)[0].toLowerCase();
          c.lang = lang || null;
          if (lang === 'rust') {
            c.value = c.value
              .split('\n')
              .filter((l) => {
                const t = l.trimStart();
                return !(t.startsWith('# ') || t === '#');
              })
              .map((l) => l.replace(/^(\s*)##/, '$1#'))
              .join('\n');
          }
        } else if ('children' in child) {
          walk(child as Parent);
        }
      }
    };
    walk(tree);
  };
}

/** `a/b/../c` gibi yolları sadeleştirir. Kökün üstüne çıkarsa null döner. */
function normalizePath(p: string): string | null {
  const out: string[] = [];
  for (const seg of p.split('/')) {
    if (seg === '' || seg === '.') continue;
    if (seg === '..') {
      if (out.length === 0) return null;
      out.pop();
    } else out.push(seg);
  }
  return out.join('/');
}

export type ResolvedLink =
  | { kind: 'external'; href: string }
  | { kind: 'hash'; href: string }
  | { kind: 'page'; href: string }
  | { kind: 'other'; href: string };

/** Sayfa içi göreli `.md` bağlantılarını `/ders/<sayfa_id>/#slug` biçimine çevirir. */
export function resolveLink(href: string, currentPageId: string): ResolvedLink {
  if (/^[a-z][a-z0-9+.-]*:/i.test(href) || href.startsWith('//')) return { kind: 'external', href };
  if (href.startsWith('#')) return { kind: 'hash', href };
  const [pathPart, hash] = splitHash(href);
  if (pathPart.endsWith('.md')) {
    const dir = currentPageId.includes('/') ? currentPageId.slice(0, currentPageId.lastIndexOf('/')) : '';
    const resolved = normalizePath(`${dir}/${pathPart.slice(0, -3)}`);
    if (resolved) return { kind: 'page', href: `/ders/${resolved}/${hash ? `#${hash}` : ''}` };
  }
  return { kind: 'other', href };
}

function splitHash(href: string): [string, string] {
  const i = href.indexOf('#');
  return i < 0 ? [href, ''] : [href.slice(0, i), href.slice(i + 1)];
}

/** Göreli görsel yolunu `/api/book-assets/...` adresine çevirir. */
export function resolveImage(src: string, currentPageId: string): string {
  if (/^([a-z][a-z0-9+.-]*:|\/\/|\/)/i.test(src)) return src;
  const dir = currentPageId.includes('/') ? currentPageId.slice(0, currentPageId.lastIndexOf('/')) : '';
  const resolved = normalizePath(`${dir}/${src}`);
  return resolved ? `/api/book-assets/${resolved}` : src;
}

/** Başlıklardaki `kod` parçalarını ayırır (içindekiler/başlık gösterimi için). */
export function splitInlineCode(title: string): { code: boolean; text: string }[] {
  const parts: { code: boolean; text: string }[] = [];
  title.split('`').forEach((text, i) => {
    if (text) parts.push({ code: i % 2 === 1, text });
  });
  return parts;
}

const APOSTROPHE_SUFFIX = /^['\u2019][\p{L}\p{N}]*/u;

/**
 * Satır içi koddan hemen sonra gelen Türkçe ek (`u32`'deki) kodla aynı satırda kalsın ve
 * kod kutusunun sağ boşluğu ekten önce boşluk bırakmasın: kod + ek, `nowrap` bir `span.code-suffix` içine alınır.
 */
export function rehypeCodeSuffix() {
  return (tree: HastRoot) => {
    const walk = (node: Element | HastRoot) => {
      const children: ElementContent[] = node.children as ElementContent[];
      if (node.type === 'element' && Array.isArray(node.properties?.className) && node.properties.className.includes('code-suffix')) return;
      if (!(node.type === 'element' && node.tagName === 'pre')) {
        for (let i = 0; i < children.length - 1; i++) {
          const c = children[i];
          const next = children[i + 1];
          if (c.type !== 'element' || c.tagName !== 'code' || next.type !== 'text') continue;
          const m = APOSTROPHE_SUFFIX.exec(next.value);
          if (!m) continue;
          const wrapper: Element = {
            type: 'element',
            tagName: 'span',
            properties: { className: ['code-suffix'] },
            children: [c, { type: 'text', value: m[0] }],
          };
          const rest = next.value.slice(m[0].length);
          children.splice(i, 2, wrapper, ...(rest ? [{ type: 'text' as const, value: rest }] : []));
        }
      }
      for (const c of children) if (c.type === 'element') walk(c);
    };
    walk(tree);
  };
}
