import { splitInlineCode } from '@/lib/markdown';

/** `kod` içeren başlıkları satır içi kod olarak gösterir (tek bir span, flex kapsayıcılarda bölünmez). */
export function Title({ text }: { text: string }) {
  return (
    <span className="title">
      {splitInlineCode(text).map((p, i) => (p.code ? <code key={i}>{p.text}</code> : p.text))}
    </span>
  );
}
