"use client";

import { useState } from "react";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";

import { useLocale } from "@/components/locale-provider";
import { NavBurgerMenu } from "@/components/nav-burger-menu";
import { TopNavTaskAttention } from "@/components/top-nav-task-attention";
import { TopNavBrand } from "@/components/top-nav-brand";
import { clientApiBaseUrl } from "@/lib/api-base";
import type { Locale } from "@/lib/i18n";
import { isPathPrefix, ROUTES } from "@/lib/routes";
import { UI_BUTTON_OUTLINE, UI_BUTTON_PRIMARY } from "@/lib/ui";
import type { BominalUser } from "@/lib/types";

type SectionRule = {
  prefix: string;
  brandKey?: string;
};

const SECTION_RULES: SectionRule[] = [
  { prefix: ROUTES.modules.train, brandKey: "nav.train" },
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

function getBrandSectionLabel(pathname: string, translate: (key: string) => string): string | null {
  const key = resolveSection(pathname)?.brandKey ?? null;
  return key ? translate(key) : null;
}

function TopNavLocaleSwitch({
  locale,
  disabled,
  onChange,
}: {
  locale: Locale;
  disabled: boolean;
  onChange: (next: Locale) => void;
}) {
  const options: Array<{ value: Locale; label: string }> = [
    { value: "en", label: "ENG" },
    { value: "ko", label: "KOR" },
  ];

  return (
    <div className="inline-flex items-center rounded-full border border-blossom-200 bg-white p-0.5 shadow-sm">
      {options.map((option) => {
        const active = locale === option.value;
        return (
          <button
            key={option.value}
            type="button"
            onClick={() => onChange(option.value)}
            disabled={disabled}
            aria-pressed={active}
            className={`rounded-full px-3 py-1 text-xs font-semibold tracking-tight transition-colors ${
              active ? "bg-blossom-500 text-white" : "text-slate-600 hover:bg-blossom-50 hover:text-blossom-700"
            } disabled:cursor-not-allowed disabled:opacity-60`}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}

export function TopNav({ user }: { user: BominalUser | null }) {
  const pathname = usePathname() ?? ROUTES.modules.train;
  const router = useRouter();
  const { locale, setLocale, t } = useLocale();
  const [localeSubmitting, setLocaleSubmitting] = useState(false);
  const brandSectionLabel = getBrandSectionLabel(pathname, t);

  const onLocaleChange = async (next: Locale) => {
    if (next === locale || localeSubmitting) return;
    const previous = locale;
    setLocale(next);

    if (!user) {
      return;
    }

    setLocaleSubmitting(true);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/auth/account`, {
        method: "PATCH",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ ui_locale: next }),
      });
      if (!response.ok) {
        setLocale(previous);
        return;
      }
      router.refresh();
    } catch {
      setLocale(previous);
    } finally {
      setLocaleSubmitting(false);
    }
  };

  return (
    <header className="sticky top-0 z-[140] border-b border-blossom-100 bg-white">
      <div className="mx-auto flex w-full max-w-5xl items-center justify-between px-4 py-3.5 sm:px-6">
        <TopNavBrand href={user ? ROUTES.modules.train : ROUTES.login} sectionLabel={brandSectionLabel} />

        {user ? (
          <div className="flex items-center gap-3">
            <TopNavLocaleSwitch locale={locale} disabled={localeSubmitting} onChange={onLocaleChange} />
            <TopNavTaskAttention userId={user.id} displayName={user.display_name?.trim() || user.email} />
            <NavBurgerMenu isAdmin={user.role === "admin"} />
          </div>
        ) : (
          <div className="flex items-center gap-2">
            <TopNavLocaleSwitch locale={locale} disabled={false} onChange={onLocaleChange} />
            <Link href={ROUTES.login} className={UI_BUTTON_OUTLINE}>
              {t("nav.login")}
            </Link>
            <Link href={ROUTES.register} className={UI_BUTTON_PRIMARY}>
              {t("nav.register")}
            </Link>
          </div>
        )}
      </div>
    </header>
  );
}
