import { headers } from "next/headers";

import { getOptionalUser } from "@/lib/server-auth";
import { localeFromAcceptLanguage, localeFromUser, t, type Locale } from "@/lib/i18n";

export async function getServerLocale(): Promise<Locale> {
  const user = await getOptionalUser();
  return localeFromUser(user) ?? localeFromAcceptLanguage(headers().get("accept-language"));
}

export async function getServerT(): Promise<{ locale: Locale; t: (key: string, vars?: Record<string, string | number>) => string }> {
  const locale = await getServerLocale();
  return { locale, t: (key, vars) => t(locale, key, vars) };
}

