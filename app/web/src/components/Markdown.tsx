'use client';
import Link from 'next/link';
import ReactMarkdown from 'react-markdown';
import type { Components, Options } from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeRaw from 'rehype-raw';
import rehypeHighlight from 'rehype-highlight';
import type { Element, ElementContent } from 'hast';
import { isValidElement, useMemo } from 'react';
import type { ReactElement, ReactNode } from 'react';
import { rehypeCodeSuffix, remarkCodeBlocks, remarkHeadingIds, remarkNotes, resolveImage, resolveLink } from '@/lib/markdown';
import type { PageNote } from '@/lib/types';
import { CodeBlock } from './CodeBlock';

function hastText(node: ElementContent | Element): string {
  if (node.type === 'text') return node.value;
  if (node.type === 'element') return node.children.map(hastText).join('');
  return '';
}

function heading(Tag: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6') {
  const withAnchor = Tag !== 'h1';
  const H = ({ id, className, children }: { id?: string; className?: string; children?: ReactNode }) => (
    <Tag id={id} className={className}>
      {children}
      {id && withAnchor && !className?.includes('sr-only') && (
        <a className="anchor" href={`#${id}`} aria-label="Bu başlığa bağlantı">
          #
        </a>
      )}
    </Tag>
  );
  H.displayName = Tag;
  return H;
}

const rehypePlugins: Options['rehypePlugins'] = [
  rehypeRaw,
  rehypeCodeSuffix,
  [rehypeHighlight, { detect: false, plainText: ['text'], ignoreMissing: true }],
];

function NoteIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 20 20" aria-hidden="true" className="app-note__icon">
      <circle cx="10" cy="10" r="8.25" fill="none" stroke="currentColor" strokeWidth="1.6" />
      <rect x="9.1" y="8.6" width="1.8" height="5.6" rx="0.9" fill="currentColor" />
      <circle cx="10" cy="6" r="1.1" fill="currentColor" />
    </svg>
  );
}

export function Markdown({ source, pageId, notes }: { source: string; pageId: string; notes?: PageNote[] }) {
  const remarkPlugins = useMemo<Options['remarkPlugins']>(
    () => [
      remarkGfm,
      remarkHeadingIds,
      [remarkNotes, notes?.length ?? 0, (notes ?? []).map((n) => n.before)],
      remarkCodeBlocks,
    ],
    [notes],
  );
  const components = useMemo<Components>(
    () => ({
      h1: heading('h1'),
      h2: heading('h2'),
      h3: heading('h3'),
      h4: heading('h4'),
      h5: heading('h5'),
      h6: heading('h6'),
      a({ href, children, node: _node, ...rest }) {
        void _node;
        if (!href) return <a {...rest}>{children}</a>;
        const r = resolveLink(href, pageId);
        if (r.kind === 'external')
          return (
            <a {...rest} href={r.href} target="_blank" rel="noopener noreferrer">
              {children}
            </a>
          );
        if (r.kind === 'page')
          return (
            <Link href={r.href} className={rest.className}>
              {children}
            </Link>
          );
        return (
          <a {...rest} href={r.href}>
            {children}
          </a>
        );
      },
      img({ src, alt }) {
        const s = typeof src === 'string' ? resolveImage(src, pageId) : undefined;
        // eslint-disable-next-line @next/next/no-img-element
        return <img src={s} alt={alt ?? ''} loading="lazy" />;
      },
      pre({ node, children }) {
        const code = node?.children.find((c): c is Element => c.type === 'element' && c.tagName === 'code');
        const cls = code?.properties?.className;
        const classes = Array.isArray(cls) ? cls.map(String) : [];
        const lang = classes.find((c) => c.startsWith('language-'))?.slice('language-'.length) ?? null;
        const text = code ? hastText(code) : '';
        const child = isValidElement(children) ? (children as ReactElement) : children;
        return (
          <CodeBlock lang={lang} text={text.replace(/\n$/, '')}>
            {child}
          </CodeBlock>
        );
      },
      div({ node, children, ...rest }) {
        const idx = node?.properties?.dataNote;
        if (idx !== undefined && notes) {
          const note = notes[Number(idx)];
          if (!note) return null;
          return (
            <aside className="app-note" aria-label="Uygulama notu">
              <div className="app-note__title">
                <NoteIcon />
                Uygulama notu
              </div>
              <Markdown source={note.markdown} pageId={pageId} />
            </aside>
          );
        }
        return <div {...rest}>{children}</div>;
      },
      table({ children }) {
        return (
          <div className="table-wrap">
            <table>{children}</table>
          </div>
        );
      },
    }),
    [pageId, notes],
  );

  return (
    <div className="prose">
      <ReactMarkdown
        remarkPlugins={remarkPlugins}
        rehypePlugins={rehypePlugins}
        remarkRehypeOptions={{ footnoteLabel: 'Dipnotlar', footnoteBackLabel: 'Metne dön' }}
        components={components}
      >
        {source}
      </ReactMarkdown>
    </div>
  );
}
