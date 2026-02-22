"use client";

import { type CSSProperties, useState } from "react";
import Link from "next/link";

import { useLocale } from "@/components/locale-provider";
import { LogoutButton } from "@/components/logout-button";
import { useTheme } from "@/components/theme-provider";
import { ROUTES } from "@/lib/routes";
import { THEME_MODE_OPTIONS, type ThemeMode } from "@/lib/theme";
import { UI_MENU_ITEM } from "@/lib/ui";

const THEME_DOT_COLORS = {
  spring: "#f6b6cf",
  summer: "#9ddfce",
  autumn: "#f2bf97",
  winter: "#aec7ed",
} as const;

function themeDotStyle(mode: ThemeMode): CSSProperties {
  if (mode === "auto") {
    return {
      backgroundImage: "linear-gradient(135deg, #f6b6cf 0%, #9ddfce 33%, #f2bf97 66%, #aec7ed 100%)",
    };
  }

  return {
    backgroundColor: THEME_DOT_COLORS[mode],
  };
}

export function NavBurgerMenu({ isAdmin = false }: { isAdmin?: boolean }) {
  const { mode, theme, setMode } = useTheme();
  const { t } = useLocale();
  const [themeMenuOpen, setThemeMenuOpen] = useState(false);
  const appVersion = (process.env.APP_VERSION ?? "0.0.0").trim() || "0.0.0";
  const themeLabel = (value: ThemeMode) => t(`theme.${value}`);
  const selectedThemeLabel = themeLabel(mode);

  return (
    <details className="group relative [&_summary::-webkit-details-marker]:hidden">
      <summary
        aria-label={t("nav.openMenu")}
        className="inline-flex h-9 w-9 cursor-pointer list-none items-center justify-center rounded-full border border-blossom-200 bg-white text-slate-700 transition hover:bg-blossom-50 focus:outline-none focus:ring-2 focus:ring-blossom-100"
      >
        <svg viewBox="0 0 20 20" className="h-4 w-4" fill="none" stroke="currentColor" strokeWidth="1.8">
          <path d="M4 6h12M4 10h12M4 14h12" strokeLinecap="round" />
        </svg>
      </summary>

      <div className="absolute right-0 z-30 mt-2 w-56 rounded-2xl border border-blossom-100 bg-white p-1.5 shadow-lg">
        <Link href={ROUTES.dashboard} className={UI_MENU_ITEM}>
          {t("nav.dashboard")}
        </Link>
        <Link href={ROUTES.modules.train} className={UI_MENU_ITEM}>
          {t("nav.train")}
        </Link>
        <Link href={ROUTES.modules.restaurant} className={UI_MENU_ITEM}>
          {t("nav.restaurant")}
        </Link>
        <Link href={ROUTES.modules.calendar} className={UI_MENU_ITEM}>
          {t("nav.calendar")}
        </Link>

        <div className="my-1 border-t border-blossom-100" />

        <div className="px-2 py-1.5">
          <button
            type="button"
            onClick={() => setThemeMenuOpen((current) => !current)}
            className="flex w-full items-center justify-between rounded-xl px-2.5 py-1.5 text-left transition hover:bg-blossom-50"
          >
            <span>
              <span className="block text-[11px] font-semibold uppercase tracking-[0.14em] text-blossom-500">
                {t("theme.label")}
              </span>
              <span className="block text-sm text-slate-700">{selectedThemeLabel}</span>
            </span>
            <svg
              viewBox="0 0 20 20"
              className={`h-4 w-4 text-slate-500 transition ${themeMenuOpen ? "rotate-180" : ""}`}
              fill="none"
              stroke="currentColor"
              strokeWidth="1.8"
            >
              <path d="M5 8l5 5 5-5" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </button>

          {themeMenuOpen ? (
            <div className="mt-1 grid gap-1">
              {THEME_MODE_OPTIONS.filter((optionMode) => optionMode !== mode).map((optionMode) => (
                <button
                  key={optionMode}
                  type="button"
                  onClick={() => {
                    setMode(optionMode);
                    setThemeMenuOpen(false);
                  }}
                  className="flex w-full items-center justify-between rounded-xl px-2.5 py-1.5 text-left text-sm text-slate-700 transition hover:bg-blossom-50"
                >
                  <span>{themeLabel(optionMode)}</span>
                  {optionMode !== "auto" ? (
                    <span
                      className="inline-flex h-2.5 w-2.5 rounded-full border border-white/80 shadow-[0_0_0_1px_rgba(15,23,42,0.08)]"
                      style={themeDotStyle(optionMode)}
                    />
                  ) : null}
                </button>
              ))}
            </div>
          ) : null}

          <p className="px-1 pt-1 text-[11px] text-slate-500">
            {t("theme.currentPrefix")} {themeLabel(theme)}
          </p>
        </div>

        <div className="my-1 border-t border-blossom-100" />

        {isAdmin && (
          <>
            <Link href={ROUTES.admin.maintenance} className={UI_MENU_ITEM}>
              {t("nav.maintenance")}
            </Link>
            <div className="my-1 border-t border-blossom-100" />
          </>
        )}

        <Link href={ROUTES.settings.account} className={UI_MENU_ITEM}>
          {t("nav.accountSettings")}
        </Link>
        <Link href={ROUTES.settings.payment} className={UI_MENU_ITEM}>
          {t("nav.paymentSettings")}
        </Link>
        <div className="mt-1 border-t border-blossom-100 pt-1">
          <LogoutButton variant="menu" />
        </div>
        <p className="px-3 pt-2 text-center text-[10px] text-slate-400">{appVersion}</p>
      </div>
    </details>
  );
}
