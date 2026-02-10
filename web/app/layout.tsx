import type { Metadata } from "next";
import { DynaPuff, Noto_Sans_KR, Noto_Serif_KR } from "next/font/google";
import { headers } from "next/headers";

import "./globals.css";

import { LocaleProvider } from "@/components/locale-provider";
import { ThemeInitScript } from "@/components/theme-init-script";
import { ThemeProvider } from "@/components/theme-provider";
import { TopNav } from "@/components/top-nav";
import { localeFromAcceptLanguage, localeFromUser, type Locale } from "@/lib/i18n";
import { getOptionalUser } from "@/lib/server-auth";
import { seasonFromMonth } from "@/lib/theme";

const fontSans = Noto_Sans_KR({
  // `next/font/google` subset typing for KR fonts currently only accepts "latin".
  // The font itself includes Korean glyphs; this controls which subset files Next requests.
  subsets: ["latin"],
  variable: "--font-sans",
  display: "swap",
});

const fontDisplay = Noto_Serif_KR({
  subsets: ["latin"],
  weight: ["400", "600", "700"],
  variable: "--font-display",
  display: "swap",
});

const fontBrand = DynaPuff({
  subsets: ["latin"],
  // Use a dedicated variable so we can keep other headings on the display font.
  variable: "--font-brand",
  display: "swap",
  weight: ["400", "600", "700"],
});

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
  const headersList = headers();
  const pathname = headersList.get("x-pathname") || "/";
  const acceptLanguage = headersList.get("accept-language");
  const locale = resolveRequestLocale(localeFromUser(user), acceptLanguage);
  const initialTheme = seasonFromMonth(new Date().getMonth() + 1);
  const isLanding = !user && pathname === "/";
  const mainClassName = isLanding
    ? "min-h-screen w-full"
    : "mx-auto w-full max-w-5xl px-4 py-8 sm:px-6 sm:py-12";
  return (
    <html
      lang={locale}
      data-theme-mode="auto"
      data-theme={initialTheme}
      className={`${fontSans.variable} ${fontDisplay.variable} ${fontBrand.variable}`}
      suppressHydrationWarning
    >
      <body className="font-sans antialiased">
        <ThemeInitScript />
        <ThemeProvider>
          <LocaleProvider initialLocale={locale}>
            {isLanding ? null : <TopNav user={user} locale={locale} />}
            <main className={mainClassName}>{children}</main>
          </LocaleProvider>
        </ThemeProvider>
      </body>
    </html>
  );
}
