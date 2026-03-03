"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { createPortal } from "react-dom";
import { type CSSProperties, useEffect, useRef, useState } from "react";

import { useLocale } from "@/components/locale-provider";
import { LogoutButton } from "@/components/logout-button";
import { useTheme } from "@/components/theme-provider";
import { NEXT_PUBLIC_RESTAURANT_MODULE_ENABLED } from "@/lib/feature-flags";
import { ROUTES } from "@/lib/routes";
import { THEME_MODE_OPTIONS, type ThemeMode } from "@/lib/theme";
import { UI_MENU_ITEM } from "@/lib/ui";

const THEME_DOT_COLORS = {
  spring: "#f6b6cf",
  summer: "#9ddfce",
  autumn: "#f2bf97",
  winter: "#aec7ed",
} as const;

const RESTAURANT_MODULE_ENABLED = NEXT_PUBLIC_RESTAURANT_MODULE_ENABLED;
// Keep links defined but hidden so they can be re-enabled without rebuilding nav structure.
const SHOW_DISABLED_MODULE_LINKS = false;
const MOBILE_DRAWER_HIDDEN_TRANSLATE = "calc(-100dvh - 24px)";

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
  const pathname = usePathname();
  const rootRef = useRef<HTMLDivElement | null>(null);
  const mobilePanelRef = useRef<HTMLDivElement | null>(null);
  const mobileDragStartYRef = useRef<number | null>(null);
  const mobileDragYRef = useRef(0);
  const [menuOpen, setMenuOpen] = useState(false);
  const [themeMenuOpen, setThemeMenuOpen] = useState(false);
  const [portalReady, setPortalReady] = useState(false);
  const [mobileDragY, setMobileDragY] = useState(0);
  const [mobileDragging, setMobileDragging] = useState(false);
  const [mobileDrawerTop, setMobileDrawerTop] = useState(0);
  const appVersion = (process.env.APP_VERSION ?? "0.0.0").trim() || "0.0.0";
  const themeLabel = (value: ThemeMode) => t(`theme.${value}`);
  const selectedThemeLabel = themeLabel(mode);
  const closeMenu = () => {
    setMenuOpen(false);
    setThemeMenuOpen(false);
    setMobileDragY(0);
    mobileDragYRef.current = 0;
    setMobileDragging(false);
    mobileDragStartYRef.current = null;
  };
  const openMenu = () => {
    setMenuOpen(true);
  };

  useEffect(() => {
    setPortalReady(true);
  }, []);

  useEffect(() => {
    closeMenu();
    // Close on route changes so menu state does not linger on next page.
  }, [pathname]);

  useEffect(() => {
    if (!menuOpen) return;

    const handleDocumentClick = (event: MouseEvent) => {
      const target = event.target as Node | null;
      if (!target) return;
      const clickedTrigger = rootRef.current?.contains(target) ?? false;
      const clickedMobilePanel = mobilePanelRef.current?.contains(target) ?? false;
      if (!clickedTrigger && !clickedMobilePanel) closeMenu();
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        closeMenu();
      }
    };

    document.addEventListener("click", handleDocumentClick);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("click", handleDocumentClick);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [menuOpen]);

  useEffect(() => {
    const updateMobileDrawerTop = () => {
      const header = rootRef.current?.closest("header");
      if (!header) {
        setMobileDrawerTop(0);
        return;
      }
      const nextTop = Math.max(0, Math.round(header.getBoundingClientRect().bottom));
      setMobileDrawerTop(nextTop);
    };

    updateMobileDrawerTop();
    if (!menuOpen) return;

    window.addEventListener("resize", updateMobileDrawerTop);
    window.addEventListener("orientationchange", updateMobileDrawerTop);
    return () => {
      window.removeEventListener("resize", updateMobileDrawerTop);
      window.removeEventListener("orientationchange", updateMobileDrawerTop);
    };
  }, [menuOpen]);

  const topMenuItems = (
    <>
      <Link href={ROUTES.modules.train} className={UI_MENU_ITEM}>
        {t("nav.train")}
      </Link>
      {SHOW_DISABLED_MODULE_LINKS ? (
        <>
          <Link href={ROUTES.dashboard} className={UI_MENU_ITEM}>
            {t("nav.dashboard")}
          </Link>
          {RESTAURANT_MODULE_ENABLED ? (
            <Link href={ROUTES.modules.restaurant} className={UI_MENU_ITEM}>
              {t("nav.restaurant")}
            </Link>
          ) : null}
          <Link href={ROUTES.modules.calendar} className={UI_MENU_ITEM}>
            {t("nav.calendar")}
          </Link>
        </>
      ) : null}
    </>
  );

  const bottomMenuItems = (
    <>
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
      <div className="mt-3 border-t border-blossom-100 pt-3">
        <LogoutButton variant="menu" />
      </div>
      <p className="px-3 pt-3 text-center text-[10px] text-slate-400">{appVersion}</p>
    </>
  );

  const onMobileHandleTouchStart = (event: React.TouchEvent<HTMLDivElement>) => {
    if (!menuOpen) return;
    const panel = mobilePanelRef.current;
    if (!panel) return;
    if (panel.scrollTop > 0) return;
    const touch = event.touches[0];
    if (!touch) return;
    const panelRect = panel.getBoundingClientRect();
    // Reserve swipe-close gesture for touches near the bottom handle area.
    if (touch.clientY < panelRect.bottom - 140) return;
    mobileDragStartYRef.current = touch.clientY;
    mobileDragYRef.current = 0;
    setMobileDragging(true);
  };

  const onMobileHandleTouchMove = (event: React.TouchEvent<HTMLDivElement>) => {
    if (!menuOpen) return;
    const startY = mobileDragStartYRef.current;
    if (startY == null) return;
    const touch = event.touches[0];
    if (!touch) return;

    const delta = touch.clientY - startY;
    if (delta >= 0) {
      mobileDragYRef.current = 0;
      setMobileDragY(0);
      return;
    }

    const clamped = Math.max(delta, -360);
    mobileDragYRef.current = clamped;
    setMobileDragY(clamped);
    event.preventDefault();
  };

  const onMobileHandleTouchEnd = () => {
    if (!menuOpen) return;
    if (mobileDragStartYRef.current == null) return;

    const shouldClose = mobileDragYRef.current <= -90;
    mobileDragStartYRef.current = null;
    setMobileDragging(false);
    if (shouldClose) {
      closeMenu();
      return;
    }
    mobileDragYRef.current = 0;
    setMobileDragY(0);
  };

  const mobileDrawer = portalReady
    ? createPortal(
        <>
          <button
            type="button"
            aria-label={t("common.close")}
            onClick={closeMenu}
            className={`fixed inset-x-0 bottom-0 z-[90] bg-slate-900/45 transition-opacity duration-350 ease-out md:hidden ${
              menuOpen ? "pointer-events-auto opacity-100" : "pointer-events-none opacity-0"
            }`}
            style={{ top: mobileDrawerTop, touchAction: "none" }}
            onTouchMove={(event) => {
              event.preventDefault();
            }}
          />

          <div
            className={`fixed inset-x-0 bottom-0 z-[100] will-change-transform md:hidden ${
              menuOpen ? "pointer-events-auto" : "pointer-events-none"
            }`}
            style={{
              top: mobileDrawerTop,
              transform: menuOpen
                ? `translate3d(0, ${mobileDragY}px, 0)`
                : `translate3d(0, ${MOBILE_DRAWER_HIDDEN_TRANSLATE}, 0)`,
              transition: mobileDragging ? "none" : "transform 420ms cubic-bezier(0.22, 1, 0.36, 1)",
            }}
          >
            <div
              ref={mobilePanelRef}
              className="absolute inset-0 overflow-hidden rounded-b-2xl border-b border-blossom-100 bg-white"
              onTouchStart={onMobileHandleTouchStart}
              onTouchMove={onMobileHandleTouchMove}
              onTouchEnd={onMobileHandleTouchEnd}
              onTouchCancel={onMobileHandleTouchEnd}
            >
              <div
                className="flex min-h-full flex-col overflow-y-auto p-2 pb-24 pt-4"
                style={{ overscrollBehavior: "contain" }}
              >
                <div>{topMenuItems}</div>
                <div className="pb-8">{bottomMenuItems}</div>
                <div
                  className="absolute inset-x-0 bottom-0 z-10 flex justify-center border-t border-blossom-100/80 bg-white/95 pb-[calc(env(safe-area-inset-bottom,0px)+10px)] pt-3 backdrop-blur"
                  style={{ touchAction: "pan-y" }}
                >
                  <span className="h-1.5 w-12 rounded-full bg-slate-300" />
                </div>
              </div>
            </div>
          </div>
        </>,
        document.body,
      )
    : null;

  return (
    <div ref={rootRef} className="relative">
      <button
        type="button"
        aria-label={t("nav.openMenu")}
        aria-expanded={menuOpen}
        onClick={() => {
          if (menuOpen) {
            closeMenu();
            return;
          }
          openMenu();
        }}
        className="inline-flex h-11 w-11 cursor-pointer list-none items-center justify-center rounded-full border border-blossom-200 bg-white text-slate-700 transition hover:bg-blossom-50 focus:outline-none focus:ring-2 focus:ring-blossom-100"
      >
        <svg viewBox="0 0 20 20" className="h-4 w-4" fill="none" stroke="currentColor" strokeWidth="1.8">
          <path d="M4 6h12M4 10h12M4 14h12" strokeLinecap="round" />
        </svg>
      </button>

      {mobileDrawer}

      <div
        className={`absolute right-0 z-30 mt-2 hidden w-56 origin-top overflow-hidden rounded-2xl border border-blossom-100 bg-white transition-all duration-300 ease-[cubic-bezier(0.22,1,0.36,1)] md:block ${
          menuOpen
            ? "max-h-[32rem] translate-y-0 opacity-100 shadow-[0_14px_28px_-18px_rgba(15,23,42,0.45)]"
            : "pointer-events-none max-h-0 -translate-y-2 opacity-0 shadow-none"
        }`}
      >
        <div className="p-1.5">
          {topMenuItems}
          {bottomMenuItems}
        </div>
      </div>
    </div>
  );
}
