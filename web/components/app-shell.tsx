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
    const prevHtmlOverscroll = (html.style as any).overscrollBehavior;
    const prevHtmlBackground = html.style.background;
    const prevBodyOverflow = body.style.overflow;
    const prevBodyHeight = body.style.height;
    const prevBodyOverscroll = (body.style as any).overscrollBehavior;
    const prevBodyBackground = body.style.background;

    html.style.overflow = "hidden";
    // Prevent scroll-chain/rubber-band in browsers that support it.
    (html.style as any).overscrollBehavior = "none";
    // Ensure any overscroll backdrop isn't the default white.
    html.style.background = "rgb(2 6 23)"; // slate-950
    body.style.overflow = "hidden";
    body.style.height = "100dvh";
    (body.style as any).overscrollBehavior = "none";
    body.style.background = "rgb(2 6 23)"; // slate-950

    return () => {
      html.style.overflow = prevHtmlOverflow;
      (html.style as any).overscrollBehavior = prevHtmlOverscroll;
      html.style.background = prevHtmlBackground;
      body.style.overflow = prevBodyOverflow;
      body.style.height = prevBodyHeight;
      (body.style as any).overscrollBehavior = prevBodyOverscroll;
      body.style.background = prevBodyBackground;
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
