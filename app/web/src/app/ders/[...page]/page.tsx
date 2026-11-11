import { LessonView } from '@/components/LessonView';
import { allPageIds } from '@/lib/book';

export const dynamicParams = false;

export function generateStaticParams() {
  return allPageIds().map((id) => ({ page: id.split('/') }));
}

export default async function LessonPage({ params }: { params: Promise<{ page: string[] }> }) {
  const { page } = await params;
  return <LessonView pageId={page.join('/')} />;
}
