import { applyThemeFavicon, THEME_FAVICON_LINK_ID, THEME_FAVICON_SHORTCUT_LINK_ID, themeFaviconHref } from "@/lib/theme-favicon";
import { describe, expect, it } from "vitest";

describe("theme-favicon", () => {
  it("builds seasonal favicon href", () => {
    expect(themeFaviconHref("spring")).toBe("/favicons/seasonal/bominal_spring.ico");
    expect(themeFaviconHref("summer")).toBe("/favicons/seasonal/bominal_summer.ico");
    expect(themeFaviconHref("autumn")).toBe("/favicons/seasonal/bominal_autumn.ico");
    expect(themeFaviconHref("winter")).toBe("/favicons/seasonal/bominal_winter.ico");
  });

  it("upserts icon links and updates href on theme change", () => {
    expect(document.getElementById(THEME_FAVICON_LINK_ID)).toBeNull();
    expect(document.getElementById(THEME_FAVICON_SHORTCUT_LINK_ID)).toBeNull();

    applyThemeFavicon("spring");

    const icon = document.getElementById(THEME_FAVICON_LINK_ID);
    const shortcut = document.getElementById(THEME_FAVICON_SHORTCUT_LINK_ID);

    expect(icon).toBeInstanceOf(HTMLLinkElement);
    expect(shortcut).toBeInstanceOf(HTMLLinkElement);
    expect(icon).toHaveAttribute("href", "/favicons/seasonal/bominal_spring.ico");
    expect(shortcut).toHaveAttribute("href", "/favicons/seasonal/bominal_spring.ico");

    applyThemeFavicon("winter");

    expect(document.querySelectorAll(`#${THEME_FAVICON_LINK_ID}`)).toHaveLength(1);
    expect(document.querySelectorAll(`#${THEME_FAVICON_SHORTCUT_LINK_ID}`)).toHaveLength(1);
    expect(icon).toHaveAttribute("href", "/favicons/seasonal/bominal_winter.ico");
    expect(shortcut).toHaveAttribute("href", "/favicons/seasonal/bominal_winter.ico");
  });
});
