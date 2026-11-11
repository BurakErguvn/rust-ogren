export function ScoreBadge({ score, large = false }: { score: number | null; large?: boolean }) {
  if (score === null) return null;
  const tier = score >= 100 ? 'top' : score >= 80 ? 'good' : 'ok';
  return (
    <span
      className={`score score--${tier}${large ? ' score--large' : ''}`}
      title={`Puan: ${score}`}
      aria-label={`Puan ${score}`}
    >
      {score}
    </span>
  );
}

export function CheckIcon({ className }: { className?: string }) {
  return (
    <svg className={className} width="14" height="14" viewBox="0 0 16 16" aria-hidden="true">
      <path d="M3 8.5l3.2 3.2L13 4.8" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}
