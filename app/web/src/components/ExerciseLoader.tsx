'use client';
import dynamic from 'next/dynamic';

// Monaco yalnızca tarayıcıda yüklenir (SSR/statik üretimde çalışmaz).
const ExerciseView = dynamic(() => import('./ExerciseView'), {
  ssr: false,
  loading: () => (
    <div className="exercise-loading" aria-busy="true">
      Editör yükleniyor…
    </div>
  ),
});

export function ExerciseLoader({ pageId }: { pageId: string }) {
  return <ExerciseView key={pageId} pageId={pageId} />;
}
