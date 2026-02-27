import { applyThemeFavicon, THEME_FAVICON_LINK_ID, themeFaviconHref } from "@/lib/theme-favicon";
import { describe, expect, it } from "vitest";

describe("theme-favicon", () => {
  it("always resolves to catdog favicon", () => {
    expect(themeFaviconHref("spring")).toBe("/favicons/catdog.png");
    expect(themeFaviconHref("summer")).toBe("/favicons/catdog.png");
    expect(themeFaviconHref("autumn")).toBe("/favicons/catdog.png");
    expect(themeFaviconHref("winter")).toBe("/favicons/catdog.png");
  });

  it("upserts icon link and updates href on theme change", () => {
    expect(document.getElementById(THEME_FAVICON_LINK_ID)).toBeNull();

    applyThemeFavicon("spring");

    const icon = document.getElementById(THEME_FAVICON_LINK_ID);

    expect(icon).toBeInstanceOf(HTMLLinkElement);
    expect(icon).toHaveAttribute("href", "/favicons/catdog.png");

    applyThemeFavicon("winter");

    expect(document.querySelectorAll(`#${THEME_FAVICON_LINK_ID}`)).toHaveLength(1);
    expect(icon).toHaveAttribute("href", "/favicons/catdog.png");
  });
});
