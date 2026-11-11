import type { Metadata, Viewport } from 'next';
import type { ReactNode } from 'react';
import './globals.css';
import { AppShell } from '@/components/AppShell';
import { themeInitScript } from '@/lib/theme';

export const metadata: Metadata = {
  title: { default: 'Rust Öğren', template: '%s · Rust Öğren' },
  description: 'Etkileşimli Türkçe Rust kursu: anlatım, egzersizler ve tarayıcıda Rust IDE.',
};

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
  colorScheme: 'light dark',
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="tr" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeInitScript }} />
      </head>
      <body>
        <AppShell>{children}</AppShell>
      </body>
    </html>
  );
}
