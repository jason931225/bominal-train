"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

export function TopNavBrand({ href }: { href: string }) {
  const pathname = usePathname();
  const sectionLabel = pathname.startsWith("/modules/train")
    ? "train"
    : pathname.startsWith("/modules/restaurant")
      ? "restaurant"
    : pathname.startsWith("/modules/calendar")
        ? "calendar"
        : pathname.startsWith("/settings/payment") ||
            pathname.startsWith("/payment") ||
            pathname.startsWith("/payment-settings")
          ? "payment settings"
          : pathname.startsWith("/settings/account") ||
              pathname.startsWith("/account") ||
              pathname.startsWith("/account-settings")
            ? "account settings"
            : pathname.startsWith("/admin")
              ? "admin"
              : null;

  return (
    <Link href={href} className="group inline-flex items-center gap-3">
      <span className="font-display text-2xl lowercase tracking-tight text-slate-900 transition group-hover:text-blossom-700">
        bominal
      </span>
      {sectionLabel ? (
        <span className="hidden text-sm font-medium text-slate-600 sm:inline">/ {sectionLabel}</span>
      ) : null}
    </Link>
  );
}
