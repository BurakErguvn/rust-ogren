import Link from 'next/link';

export default function NotFound() {
  return (
    <div className="lesson">
      <h1>Sayfa bulunamadı</h1>
      <p>Aradığın sayfa yok ya da taşınmış olabilir.</p>
      <p>
        <Link href="/" className="btn btn--primary">
          Ana sayfaya dön
        </Link>
      </p>
    </div>
  );
}
