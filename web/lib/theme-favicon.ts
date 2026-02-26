import type { ThemeName } from "@/lib/theme";

const THEME_FAVICON_PATH = "/favicons/catdog.ico";

export const THEME_FAVICON_LINK_ID = "bominal-theme-icon";
export const THEME_FAVICON_SHORTCUT_LINK_ID = "bominal-theme-shortcut-icon";

export function themeFaviconHref(_theme: ThemeName): string {
  return THEME_FAVICON_PATH;
}

function upsertLink(id: string, rel: "icon" | "shortcut icon", href: string): void {
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
    link.rel = rel;
    link.type = "image/x-icon";
    document.head.appendChild(link);
  }
  link.setAttribute("href", href);
}

export function applyThemeFavicon(theme: ThemeName): void {
  const href = themeFaviconHref(theme);
  upsertLink(THEME_FAVICON_LINK_ID, "icon", href);
  upsertLink(THEME_FAVICON_SHORTCUT_LINK_ID, "shortcut icon", href);
}
