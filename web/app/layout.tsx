import type { Metadata } from "next";
import { Fraunces, Manrope } from "next/font/google";

import "./globals.css";

import { ThemeInitScript } from "@/components/theme-init-script";
import { ThemeProvider } from "@/components/theme-provider";
import { TopNav } from "@/components/top-nav";
import { seasonFromMonth } from "@/lib/theme";

const fontSans = Manrope({
  subsets: ["latin"],
  variable: "--font-sans",
  display: "swap",
});

const fontDisplay = Fraunces({
  subsets: ["latin"],
  variable: "--font-display",
  display: "swap",
});

export const metadata: Metadata = {
  title: "bominal",
  description: "bominal modular dashboard and train automation.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const initialTheme = seasonFromMonth(new Date().getMonth() + 1);
  return (
    <html
      lang="en"
      data-theme-mode="auto"
      data-theme={initialTheme}
      className={`${fontSans.variable} ${fontDisplay.variable}`}
      suppressHydrationWarning
    >
      <body className="font-sans antialiased">
        <ThemeInitScript />
        <ThemeProvider>
          <TopNav />
          <main className="mx-auto w-full max-w-5xl px-4 py-8 sm:px-6 sm:py-12">{children}</main>
        </ThemeProvider>
      </body>
    </html>
  );
}
