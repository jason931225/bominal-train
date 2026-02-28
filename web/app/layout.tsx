import type { Metadata } from "next";
import { headers } from "next/headers";

import "./globals.css";

import { AppShell } from "@/components/app-shell";
import { LocaleProvider } from "@/components/locale-provider";
import { ThemeInitScript } from "@/components/theme-init-script";
import { ThemeProvider } from "@/components/theme-provider";
import { localeFromAcceptLanguage, localeFromUser, type Locale } from "@/lib/i18n";
import { getOptionalUser } from "@/lib/server-auth";
import { seasonFromMonth } from "@/lib/theme";

const DEFAULT_FONT_BASE_URL =
  "https://github.com/jason931225/bominal.github.io/raw/refs/heads/main/public/font";
const GOOGLE_FONT_STYLESHEET_URL =
  "https://fonts.googleapis.com/css2?family=DynaPuff:wght@400;600;700&family=Noto+Sans+KR:wght@100..900&family=Noto+Serif+KR:wght@400;600;700&display=swap";

function resolveFontBaseUrl(value: string | undefined): string {
  if (!value) return DEFAULT_FONT_BASE_URL;
  try {
    const parsed = new URL(value);
    if (parsed.protocol !== "https:") return DEFAULT_FONT_BASE_URL;
    return parsed.toString().replace(/\/+$/, "");
  } catch {
    return DEFAULT_FONT_BASE_URL;
  }
}

function remoteFontFaceCss(fontBaseUrl: string): string {
  return `
@font-face {
  font-family: "Bominal Sans Fallback";
  src: url("${fontBaseUrl}/NotoSansKR-Regular.woff2") format("woff2");
  font-style: normal;
  font-weight: 100 900;
  font-display: swap;
}
@font-face {
  font-family: "Bominal Display Fallback";
  src: url("${fontBaseUrl}/NotoSerifKR-Regular.woff2") format("woff2");
  font-style: normal;
  font-weight: 400;
  font-display: swap;
}
@font-face {
  font-family: "Bominal Display Fallback";
  src: url("${fontBaseUrl}/NotoSerifKR-SemiBold.woff2") format("woff2");
  font-style: normal;
  font-weight: 600;
  font-display: swap;
}
@font-face {
  font-family: "Bominal Display Fallback";
  src: url("${fontBaseUrl}/NotoSerifKR-Bold.woff2") format("woff2");
  font-style: normal;
  font-weight: 700;
  font-display: swap;
}
@font-face {
  font-family: "Bominal Brand Fallback";
  src: url("${fontBaseUrl}/DynaPuff-SemiBold.woff2") format("woff2");
  font-style: normal;
  font-weight: 400 700;
  font-display: swap;
}
:root {
  --font-sans: "Noto Sans KR", "Bominal Sans Fallback";
  --font-display: "Noto Serif KR", "Bominal Display Fallback";
  --font-brand: "DynaPuff", "Bominal Brand Fallback";
}
`;
}

export const metadata: Metadata = {
  title: "bominal",
  description: "bominal modular dashboard and train automation.",
};

function resolveRequestLocale(userLocale: Locale | null, acceptLanguage: string | null): Locale {
  return userLocale ?? localeFromAcceptLanguage(acceptLanguage);
}

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const user = await getOptionalUser();
  const headerStore = await headers();
  const acceptLanguage = headerStore.get("accept-language");
  const locale = resolveRequestLocale(localeFromUser(user), acceptLanguage);
  const initialTheme = seasonFromMonth(new Date().getMonth() + 1);
  const fontBaseUrl = resolveFontBaseUrl(process.env.NEXT_PUBLIC_FONT_BASE_URL);

  return (
    <html
      lang={locale}
      data-theme-mode="auto"
      data-theme={initialTheme}
      suppressHydrationWarning
    >
      <head>
        <link rel="stylesheet" href={GOOGLE_FONT_STYLESHEET_URL} />
        <style id="remote-font-faces">{remoteFontFaceCss(fontBaseUrl)}</style>
      </head>
      <body className="font-sans antialiased">
        <ThemeInitScript />
        <ThemeProvider>
          <LocaleProvider initialLocale={locale}>
            <AppShell user={user}>
              {children}
            </AppShell>
          </LocaleProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
