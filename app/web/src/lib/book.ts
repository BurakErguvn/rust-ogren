// Derleme zamanında (Node) kullanılır: generateStaticParams için SUMMARY.md ve egzersiz dizinleri.
import fs from 'node:fs';
import path from 'node:path';

const root = process.env.REPO_ROOT ?? path.resolve(process.cwd(), '../..');

export function allPageIds(): string[] {
  const summary = fs.readFileSync(path.join(root, 'book/src/SUMMARY.md'), 'utf8');
  const ids: string[] = [];
  const re = /^\s*[-*]\s+\[[^\]]*\]\(([^)]+)\.md\)/gm;
  for (const m of summary.matchAll(re)) ids.push(m[1]);
  return ids;
}

/** `tr/removed-exercises.json` (`{ "removed": [...] }`); dosya yoksa/bozuksa hiçbiri kaldırılmamıştır. */
export function removedExercises(): Set<string> {
  try {
    const raw = fs.readFileSync(path.join(root, 'tr/removed-exercises.json'), 'utf8');
    const j = JSON.parse(raw) as { removed?: unknown };
    return new Set(Array.isArray(j.removed) ? j.removed.filter((x): x is string => typeof x === 'string') : []);
  } catch {
    return new Set();
  }
}

export function hasExercise(pageId: string): boolean {
  return !removedExercises().has(pageId) && fs.existsSync(path.join(root, 'exercises', pageId));
}

export function exercisePageIds(): string[] {
  return allPageIds().filter(hasExercise);
}
