"use client";

import { useEffect } from "react";
import { usePathname } from "next/navigation";

import { PageTransition } from "@/components/page-transition";
import { TopNav } from "@/components/top-nav";
import type { Locale } from "@/lib/i18n";
import type { BominalUser } from "@/lib/types";

export function AppShell({
  user,
  locale,
  children,
}: {
  user: BominalUser | null;
  locale: Locale;
  children: React.ReactNode;
}) {
  const pathname = usePathname();

  // Root `app/layout.tsx` is persistent across navigations. Use client pathname
  // so landing-vs-app chrome stays correct even on back/forward restores.
  const isLanding = !user && pathname === "/";

  useEffect(() => {
    if (!isLanding) return;

    const html = document.documentElement;
    const body = document.body;

    const prevHtmlOverflow = html.style.overflow;
    const prevBodyOverflow = body.style.overflow;
    const prevBodyHeight = body.style.height;

    html.style.overflow = "hidden";
    body.style.overflow = "hidden";
    body.style.height = "100dvh";

    return () => {
      html.style.overflow = prevHtmlOverflow;
      body.style.overflow = prevBodyOverflow;
      body.style.height = prevBodyHeight;
    };
  }, [isLanding]);

  const mainClassName = isLanding
    ? "h-[100dvh] w-full overflow-hidden"
    : "mx-auto w-full max-w-5xl px-4 py-8 sm:px-6 sm:py-12";

  return (
    <>
      {isLanding ? null : <TopNav user={user} locale={locale} />}
      <main className={mainClassName}>
        {isLanding ? children : <PageTransition>{children}</PageTransition>}
      </main>
    </>
  );
}
