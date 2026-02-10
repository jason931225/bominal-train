export const THEME_STORAGE_KEY = "bominal_theme_mode";

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
