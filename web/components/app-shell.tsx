"use client";

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

  const mainClassName = isLanding
    ? "min-h-screen w-full"
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
