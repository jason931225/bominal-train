import { applyThemeFavicon, THEME_FAVICON_LINK_ID, THEME_FAVICON_SHORTCUT_LINK_ID, themeFaviconHref } from "@/lib/theme-favicon";
import { describe, expect, it } from "vitest";

describe("theme-favicon", () => {
  it("always resolves to catdog favicon", () => {
    expect(themeFaviconHref("spring")).toBe("/favicons/catdog.ico");
    expect(themeFaviconHref("summer")).toBe("/favicons/catdog.ico");
    expect(themeFaviconHref("autumn")).toBe("/favicons/catdog.ico");
    expect(themeFaviconHref("winter")).toBe("/favicons/catdog.ico");
  });

  it("upserts icon links and updates href on theme change", () => {
    expect(document.getElementById(THEME_FAVICON_LINK_ID)).toBeNull();
    expect(document.getElementById(THEME_FAVICON_SHORTCUT_LINK_ID)).toBeNull();

    applyThemeFavicon("spring");

    const icon = document.getElementById(THEME_FAVICON_LINK_ID);
    const shortcut = document.getElementById(THEME_FAVICON_SHORTCUT_LINK_ID);

    expect(icon).toBeInstanceOf(HTMLLinkElement);
    expect(shortcut).toBeInstanceOf(HTMLLinkElement);
    expect(icon).toHaveAttribute("href", "/favicons/catdog.ico");
    expect(shortcut).toHaveAttribute("href", "/favicons/catdog.ico");

    applyThemeFavicon("winter");

    expect(document.querySelectorAll(`#${THEME_FAVICON_LINK_ID}`)).toHaveLength(1);
    expect(document.querySelectorAll(`#${THEME_FAVICON_SHORTCUT_LINK_ID}`)).toHaveLength(1);
    expect(icon).toHaveAttribute("href", "/favicons/catdog.ico");
    expect(shortcut).toHaveAttribute("href", "/favicons/catdog.ico");
  });
});
