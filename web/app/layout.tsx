import type { Metadata } from "next";

import "./globals.css";

import { ThemeInitScript } from "@/components/theme-init-script";
import { ThemeProvider } from "@/components/theme-provider";
import { TopNav } from "@/components/top-nav";

export const metadata: Metadata = {
  title: "bominal",
  description: "bominal modular dashboard",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
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
