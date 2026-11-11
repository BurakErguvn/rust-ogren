'use client';
import { useState } from 'react';
import type { ReactNode } from 'react';

const LANG_LABEL: Record<string, string> = { rust: 'Rust', toml: 'TOML', bash: 'Kabuk', text: 'Çıktı' };

export function CodeBlock({ lang, text, children }: { lang: string | null; text: string; children: ReactNode }) {
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      /* pano erişimi yok */
    }
  };
  return (
    <div className={`codeblock${lang ? ` codeblock--${lang}` : ''}`}>
      <div className="codeblock__bar">
        <span className="codeblock__lang">{lang ? (LANG_LABEL[lang] ?? lang) : ''}</span>
        <button type="button" className="codeblock__copy" onClick={() => void copy()}>
          {copied ? 'Kopyalandı' : 'Kopyala'}
        </button>
      </div>
      <pre>{children}</pre>
    </div>
  );
}
