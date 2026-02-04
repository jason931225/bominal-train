import type { Metadata } from "next";

import "./globals.css";

import { ThemeInitScript } from "@/components/theme-init-script";
import { ThemeProvider } from "@/components/theme-provider";
import { TopNav } from "@/components/top-nav";
import { seasonFromMonth } from "@/lib/theme";

export const metadata: Metadata = {
  title: "bominal",
  description: "bominal modular dashboard",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const initialTheme = seasonFromMonth(new Date().getMonth() + 1);
  return (
    <html lang="en" data-theme-mode="auto" data-theme={initialTheme} suppressHydrationWarning>
      <body>
        <ThemeInitScript />
        <ThemeProvider>
          <TopNav />
          <main className="mx-auto w-full max-w-5xl px-4 py-8 sm:px-6 sm:py-12">{children}</main>
        </ThemeProvider>
      </body>
    </html>
  );
}
