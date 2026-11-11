'use client';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';
import type { ReactNode } from 'react';
import { Sidebar, currentRoute } from './Sidebar';
import { ScoreInfo } from './ScoreInfo';
import { ThemeToggle } from './ThemeToggle';
import { TocProvider, useToc } from './TocContext';
import { useLocalBool } from '@/lib/useLocalBool';

function HeaderProgress() {
  const { toc } = useToc();
  if (!toc) return null;
  return (
    <span className="header__progress">
      <span title="Tamamlanan egzersiz / toplam · toplam puan">
        {toc.progress.completed}/{toc.progress.total}
        <span className="header__progress-score"> · {toc.progress.score_sum} puan</span>
      </span>
      <ScoreInfo />
    </span>
  );
}

function Shell({ children }: { children: ReactNode }) {
  const pathname = usePathname() ?? '/';
  const [collapsed, setCollapsed] = useLocalBool('rust-ogren:sidebar-collapsed', false);
  const [mobileOpen, setMobileOpen] = useState(false);
  const isHome = pathname === '/';
  const isExercise = currentRoute(pathname).kind === 'egzersiz';

  const toggle = () => {
    if (window.matchMedia('(max-width: 900px)').matches) setMobileOpen((o) => !o);
    else setCollapsed(!collapsed);
  };

  const cls = ['shell', collapsed ? 'shell--collapsed' : '', mobileOpen ? 'shell--mobile-open' : '', isHome ? 'shell--home' : '']
    .filter(Boolean)
    .join(' ');

  return (
    <div className={cls}>
      <header className="header">
        {!isHome && (
          <button type="button" className="icon-btn" onClick={toggle} aria-label="İçindekiler panelini aç/kapat" title="İçindekiler">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" aria-hidden="true">
              <path d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>
        )}
        <Link href="/" className="brand">
          <span className="brand__logo" aria-hidden="true">
            🦀
          </span>
          <span className="brand__name">Rust Öğren</span>
        </Link>
        <div className="header__spacer" />
        <HeaderProgress />
        <ThemeToggle />
      </header>
      <div className="shell__body">
        {!isHome && (
          <>
            <aside className="sidebar" id="sidebar">
              <Sidebar pathname={pathname} onNavigate={() => setMobileOpen(false)} />
            </aside>
            <div className="backdrop" onClick={() => setMobileOpen(false)} aria-hidden="true" />
          </>
        )}
        <main className={`main${isExercise ? ' main--fill' : ''}`} id="main">
          {children}
        </main>
      </div>
    </div>
  );
}

export function AppShell({ children }: { children: ReactNode }) {
  return (
    <TocProvider>
      <Shell>{children}</Shell>
    </TocProvider>
  );
}
