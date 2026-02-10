"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

import { NavBurgerMenu } from "@/components/nav-burger-menu";
import { TopNavBrand } from "@/components/top-nav-brand";
import type { Locale } from "@/lib/i18n";
import { t } from "@/lib/i18n";
import { isPathPrefix, ROUTES } from "@/lib/routes";
import { UI_BUTTON_OUTLINE, UI_BUTTON_PRIMARY } from "@/lib/ui";
import type { BominalUser } from "@/lib/types";

type SectionRule = {
  prefix: string;
  brandKey?: string;
};

const SECTION_RULES: SectionRule[] = [
  { prefix: ROUTES.modules.train, brandKey: "nav.train" },
  { prefix: ROUTES.modules.restaurant, brandKey: "nav.restaurant" },
  { prefix: ROUTES.modules.calendar, brandKey: "nav.calendar" },
  { prefix: ROUTES.settings.account, brandKey: "nav.accountSettings" },
  { prefix: ROUTES.settings.payment, brandKey: "nav.paymentSettings" },
  { prefix: ROUTES.admin.maintenance, brandKey: "nav.admin" },
  { prefix: ROUTES.admin.root, brandKey: "nav.admin" },
];

function resolveSection(pathname: string): SectionRule | null {
  for (const rule of SECTION_RULES) {
    if (isPathPrefix(pathname, rule.prefix)) return rule;
  }
  return null;
}

function getBrandSectionLabel(locale: Locale, pathname: string): string | null {
  const key = resolveSection(pathname)?.brandKey ?? null;
  return key ? t(locale, key) : null;
}

export function TopNav({ user, locale }: { user: BominalUser | null; locale: Locale }) {
  const pathname = usePathname() ?? ROUTES.dashboard;
  const brandSectionLabel = getBrandSectionLabel(locale, pathname);

  return (
    <header className="sticky top-0 z-20 border-b border-blossom-100/80 bg-white/90 backdrop-blur">
      <div className="mx-auto flex w-full max-w-5xl items-center justify-between px-4 py-3.5 sm:px-6">
        <TopNavBrand href={user ? ROUTES.dashboard : ROUTES.login} sectionLabel={brandSectionLabel} />

        {user ? (
          <div className="flex items-center gap-3">
            <span
              title={user.display_name?.trim() || user.email}
              className="inline-flex h-9 max-w-[11rem] items-center truncate rounded-full border border-blossom-200 bg-white px-3 text-sm font-medium text-slate-700 shadow-sm"
            >
              {user.display_name?.trim() || user.email}
            </span>
            <NavBurgerMenu isAdmin={user.role === "admin"} />
          </div>
        ) : (
          <div className="flex items-center gap-2">
            <Link href={ROUTES.login} className={UI_BUTTON_OUTLINE}>
              {t(locale, "nav.login")}
            </Link>
            <Link href={ROUTES.register} className={UI_BUTTON_PRIMARY}>
              {t(locale, "nav.register")}
            </Link>
          </div>
        )}
      </div>
    </header>
  );
}
