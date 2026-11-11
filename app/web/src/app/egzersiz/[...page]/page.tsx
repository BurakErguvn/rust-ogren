import { ExerciseLoader } from '@/components/ExerciseLoader';
import { exercisePageIds } from '@/lib/book';

export const dynamicParams = false;

export function generateStaticParams() {
  return exercisePageIds().map((id) => ({ page: id.split('/') }));
}

export default async function ExercisePage({ params }: { params: Promise<{ page: string[] }> }) {
  const { page } = await params;
  return <ExerciseLoader pageId={page.join('/')} />;
}
