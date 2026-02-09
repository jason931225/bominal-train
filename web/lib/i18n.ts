import type { BominalUser } from "@/lib/types";

import en from "@/messages/en.json";
import ko from "@/messages/ko.json";

export type Locale = "en" | "ko";

type Messages = typeof en;
type MessagesRoot = Record<string, unknown>;

const MESSAGES: Record<Locale, Messages> = { en, ko };

export function isLocale(value: string | null | undefined): value is Locale {
  return value === "en" || value === "ko";
}

export function localeFromAcceptLanguage(value: string | null | undefined): Locale {
  const raw = (value || "").toLowerCase();
  if (raw.startsWith("ko")) return "ko";
  return "en";
}

export function localeFromUser(user: BominalUser | null | undefined): Locale | null {
  const candidate = user?.ui_locale;
  return isLocale(candidate) ? candidate : null;
}

function readPath(obj: MessagesRoot, path: string): unknown {
  const parts = path.split(".").filter(Boolean);
  let cur: unknown = obj;
  for (const part of parts) {
    if (typeof cur !== "object" || cur === null) return null;
    const record = cur as Record<string, unknown>;
    cur = record[part];
  }
  return cur;
}

function interpolate(template: string, vars?: Record<string, string | number>): string {
  if (!vars) return template;
  return template.replace(/\{(\w+)\}/g, (_match, key: string) => {
    const value = vars[key];
    return value === undefined || value === null ? "" : String(value);
  });
}

export function t(locale: Locale, key: string, vars?: Record<string, string | number>): string {
  const msg = readPath(MESSAGES[locale] as unknown as MessagesRoot, key);
  if (typeof msg === "string") {
    return interpolate(msg, vars);
  }
  // Fallback to English, then key.
  const fallback = readPath(MESSAGES.en as unknown as MessagesRoot, key);
  if (typeof fallback === "string") {
    return interpolate(fallback, vars);
  }
  return key;
}

