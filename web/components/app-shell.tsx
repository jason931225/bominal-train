"use client";

import { useEffect } from "react";
import { usePathname } from "next/navigation";

import { PageTransition } from "@/components/page-transition";
import { TopNav } from "@/components/top-nav";
import type { Locale } from "@/lib/i18n";
import { ROUTES } from "@/lib/routes";
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
  const isReviewRoute = pathname === ROUTES.applicationReview;
  const isApprovedUser = !user || String(user.access_status || "").toLowerCase() === "approved";
  const showTopNav = !isLanding && !isReviewRoute && isApprovedUser;

  useEffect(() => {
    const html = document.documentElement;
    const body = document.body;

    html.classList.toggle("landing", isLanding);
    body.classList.toggle("landing", isLanding);

    if (!isLanding) return;

    const prevHtmlOverflow = html.style.overflow;
    const prevHtmlOverscroll = (html.style as any).overscrollBehavior;
    const prevBodyOverflow = body.style.overflow;
    const prevBodyHeight = body.style.height;
    const prevBodyOverscroll = (body.style as any).overscrollBehavior;
    const prevBodyPosition = body.style.position;
    const prevBodyTop = body.style.top;
    const prevBodyWidth = body.style.width;

    const prevScrollY = window.scrollY;
    const existingViewport = document.querySelector('meta[name="viewport"]');
    const viewportMeta = existingViewport ?? document.createElement("meta");
    const createdViewportMeta = !existingViewport;
    const prevViewportContent = existingViewport?.getAttribute("content");

    if (createdViewportMeta) {
      viewportMeta.setAttribute("name", "viewport");
      document.head.appendChild(viewportMeta);
    }

    html.style.overflow = "hidden";
    // Prevent scroll-chain/rubber-band in browsers that support it.
    (html.style as any).overscrollBehavior = "none";
    viewportMeta.setAttribute(
      "content",
      "width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no, viewport-fit=cover",
    );

    // iOS Safari can still "rubber-band" even with overflow hidden. A fixed-body
    // lock is more reliable and avoids the default white backdrop bleeding in.
    body.style.position = "fixed";
    body.style.top = `-${prevScrollY}px`;
    body.style.width = "100%";
    body.style.overflow = "hidden";
    body.style.height = "100dvh";
    (body.style as any).overscrollBehavior = "none";

    return () => {
      html.style.overflow = prevHtmlOverflow;
      (html.style as any).overscrollBehavior = prevHtmlOverscroll;
      body.style.overflow = prevBodyOverflow;
      body.style.height = prevBodyHeight;
      body.style.position = prevBodyPosition;
      body.style.top = prevBodyTop;
      body.style.width = prevBodyWidth;
      (body.style as any).overscrollBehavior = prevBodyOverscroll;

      // Restore scroll position after unlocking.
      window.scrollTo(0, prevScrollY);

      html.classList.remove("landing");
      body.classList.remove("landing");

      if (createdViewportMeta) {
        viewportMeta.remove();
      } else if (prevViewportContent != null) {
        viewportMeta.setAttribute("content", prevViewportContent);
      } else {
        viewportMeta.removeAttribute("content");
      }
    };
  }, [isLanding]);

  const mainClassName = isLanding || isReviewRoute
    ? "h-[100dvh] w-full overflow-hidden"
    : "mx-auto w-full max-w-5xl px-4 py-8 sm:px-6 sm:py-12";

  return (
    <>
      {showTopNav ? <TopNav user={user} locale={locale} /> : null}
      <main className={mainClassName}>
        {isLanding || isReviewRoute ? children : <PageTransition>{children}</PageTransition>}
      </main>
    </>
  );
}
