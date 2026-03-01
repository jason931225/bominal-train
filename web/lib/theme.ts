export const THEME_STORAGE_KEY = "bominal_theme_mode";
export const THEME_RESOLVED_COOKIE_KEY = "bominal_theme_resolved";
export const THEME_COOKIE_MAX_AGE_SECONDS = 60 * 60 * 24 * 365;

export const SEASON_THEMES = ["spring", "summer", "autumn", "winter"] as const;

export type ThemeName = (typeof SEASON_THEMES)[number];
export type ThemeMode = ThemeName | "auto";

export const THEME_MODE_OPTIONS: ThemeMode[] = ["auto", "spring", "summer", "autumn", "winter"];

export function seasonFromMonth(month: number): ThemeName {
  if (month >= 3 && month <= 5) return "spring";
  if (month >= 6 && month <= 8) return "summer";
  if (month >= 9 && month <= 11) return "autumn";
  return "winter";
}

export function resolveTheme(mode: ThemeMode, date = new Date()): ThemeName {
  if (mode === "auto") {
    return seasonFromMonth(date.getMonth() + 1);
  }
  return mode;
}

export function isThemeMode(value: string | null | undefined): value is ThemeMode {
  return value === "auto" || value === "spring" || value === "summer" || value === "autumn" || value === "winter";
}

export function isThemeName(value: string | null | undefined): value is ThemeName {
  return value === "spring" || value === "summer" || value === "autumn" || value === "winter";
}

export function resolveInitialThemeFromCookies({
  modeCookie,
  resolvedCookie,
  now = new Date(),
}: {
  modeCookie: string | null | undefined;
  resolvedCookie: string | null | undefined;
  now?: Date;
}): { mode: ThemeMode; theme: ThemeName } {
  const mode: ThemeMode = isThemeMode(modeCookie) ? modeCookie : "auto";
  if (mode === "auto" && isThemeName(resolvedCookie)) {
    return { mode, theme: resolvedCookie };
  }
  return { mode, theme: resolveTheme(mode, now) };
}
