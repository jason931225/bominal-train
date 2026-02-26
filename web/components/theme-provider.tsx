"use client";

import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";

import { applyThemeFavicon } from "@/lib/theme-favicon";
import { THEME_STORAGE_KEY, isThemeMode, resolveTheme, type ThemeMode, type ThemeName } from "@/lib/theme";

type ThemeContextValue = {
  mode: ThemeMode;
  theme: ThemeName;
  setMode: (mode: ThemeMode) => void;
};

const ThemeContext = createContext<ThemeContextValue | null>(null);

function getInitialMode(): ThemeMode {
  if (typeof document !== "undefined") {
    const attrMode = document.documentElement.getAttribute("data-theme-mode");
    if (isThemeMode(attrMode)) {
      return attrMode;
    }
  }

  if (typeof window !== "undefined") {
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
    if (isThemeMode(stored)) {
      return stored;
    }
  }

  return "auto";
}

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [mode, setModeState] = useState<ThemeMode>(() => getInitialMode());
  const [theme, setTheme] = useState<ThemeName>(() => resolveTheme(getInitialMode()));

  const applyMode = useCallback((nextMode: ThemeMode) => {
    const resolved = resolveTheme(nextMode);
    setTheme(resolved);

    if (typeof document !== "undefined") {
      const root = document.documentElement;
      root.dataset.themeMode = nextMode;
      root.dataset.theme = resolved;
      root.classList.add("theme-ready");
      applyThemeFavicon(resolved);
    }

    if (typeof window !== "undefined") {
      window.localStorage.setItem(THEME_STORAGE_KEY, nextMode);
    }
  }, []);

  useEffect(() => {
    applyMode(mode);
  }, [mode, applyMode]);

  useEffect(() => {
    if (mode !== "auto") {
      return;
    }

    const timer = window.setInterval(() => {
      applyMode("auto");
    }, 60 * 60 * 1000);

    return () => {
      window.clearInterval(timer);
    };
  }, [mode, applyMode]);

  const setMode = useCallback((nextMode: ThemeMode) => {
    setModeState(nextMode);
  }, []);

  const value = useMemo(
    () => ({
      mode,
      theme,
      setMode,
    }),
    [mode, theme, setMode],
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme(): ThemeContextValue {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error("useTheme must be used within ThemeProvider");
  }
  return context;
}
