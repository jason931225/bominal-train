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

    const scrollY = window.scrollY;

    const hadLandingClass = html.classList.contains("landing");

    const prevHtmlOverflow = html.style.overflow;
    const prevHtmlHeight = html.style.height;
    const prevHtmlOverscroll = html.style.getPropertyValue("overscroll-behavior");

    const prevBodyOverflow = body.style.overflow;
    const prevBodyHeight = body.style.height;
    const prevBodyMinHeight = body.style.minHeight;
    const prevBodyPosition = body.style.position;
    const prevBodyTop = body.style.top;
    const prevBodyWidth = body.style.width;
    const prevBodyOverscroll = body.style.getPropertyValue("overscroll-behavior");

    html.classList.add("landing");

    // Lock landing to a true fullscreen canvas. iOS Safari can still rubber-band
    // even with overflow hidden unless the body is fixed.
    html.style.overflow = "hidden";
    html.style.height = "100dvh";
    html.style.setProperty("overscroll-behavior", "none");

    body.style.overflow = "hidden";
    body.style.height = "100dvh";
    body.style.minHeight = "100dvh";
    body.style.position = "fixed";
    body.style.top = `-${scrollY}px`;
    body.style.width = "100%";
    body.style.setProperty("overscroll-behavior", "none");

    return () => {
      if (!hadLandingClass) {
        html.classList.remove("landing");
      }

      html.style.overflow = prevHtmlOverflow;
      html.style.height = prevHtmlHeight;
      html.style.setProperty("overscroll-behavior", prevHtmlOverscroll);

      body.style.overflow = prevBodyOverflow;
      body.style.height = prevBodyHeight;
      body.style.minHeight = prevBodyMinHeight;
      body.style.position = prevBodyPosition;
      body.style.top = prevBodyTop;
      body.style.width = prevBodyWidth;
      body.style.setProperty("overscroll-behavior", prevBodyOverscroll);

      window.scrollTo(0, scrollY);
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
