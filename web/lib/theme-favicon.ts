import type { ThemeName } from "@/lib/theme";

const THEME_FAVICON_PATH = "/favicons/catdog.png";

export const THEME_FAVICON_LINK_ID = "bominal-theme-icon";

export function themeFaviconHref(_theme: ThemeName): string {
  return THEME_FAVICON_PATH;
}

function upsertLink(id: string, href: string): void {
  if (typeof document === "undefined") {
    return;
  }

  const existing = document.getElementById(id);
  let link: HTMLLinkElement;
  if (existing instanceof HTMLLinkElement) {
    link = existing;
  } else {
    link = document.createElement("link");
    link.id = id;
    link.rel = "icon";
    link.type = "image/png";
    document.head.appendChild(link);
  }
  link.setAttribute("href", href);
}

export function applyThemeFavicon(theme: ThemeName): void {
  const href = themeFaviconHref(theme);
  upsertLink(THEME_FAVICON_LINK_ID, href);
}
