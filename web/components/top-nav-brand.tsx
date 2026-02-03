"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

import { useTheme } from "@/components/theme-provider";
import { THEME_BUILD_LABEL } from "@/lib/theme";

export function TopNavBrand({ href }: { href: string }) {
  const pathname = usePathname();
  const { theme } = useTheme();
  const sectionLabel = pathname.startsWith("/modules/train")
    ? "train"
    : pathname.startsWith("/modules/restaurant")
      ? "restaurant"
      : pathname.startsWith("/modules/calendar")
        ? "calendar"
        : pathname.startsWith("/payment-settings")
          ? "payment settings"
          : pathname.startsWith("/account-settings")
            ? "account settings"
            : pathname.startsWith("/admin")
              ? "admin"
              : null;

  return (
    <Link href={href} className="group inline-flex items-center gap-3">
      <span className="font-serif text-2xl lowercase tracking-tight text-blossom-700 transition group-hover:text-blossom-600">
        bominal
      </span>
      <span className="hidden rounded-full border border-blossom-100 bg-blossom-50 px-2 py-0.5 text-xs text-blossom-600 sm:inline">
        {THEME_BUILD_LABEL[theme]}
      </span>
      {sectionLabel ? (
        <span className="hidden text-sm font-medium text-slate-600 sm:inline">/ {sectionLabel}</span>
      ) : null}
    </Link>
  );
}
