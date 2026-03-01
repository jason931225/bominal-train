import { describe, expect, it } from "vitest";

import { resolveInitialThemeFromCookies } from "@/lib/theme";

describe("resolveInitialThemeFromCookies", () => {
  it("uses resolved theme cookie when mode is auto", () => {
    const result = resolveInitialThemeFromCookies({
      modeCookie: "auto",
      resolvedCookie: "winter",
      now: new Date(2026, 2, 1),
    });

    expect(result).toEqual({ mode: "auto", theme: "winter" });
  });

  it("falls back to seasonal auto theme when resolved cookie is missing", () => {
    const result = resolveInitialThemeFromCookies({
      modeCookie: "auto",
      resolvedCookie: null,
      now: new Date(2026, 2, 1),
    });

    expect(result).toEqual({ mode: "auto", theme: "spring" });
  });

  it("ignores resolved cookie when mode is explicit", () => {
    const result = resolveInitialThemeFromCookies({
      modeCookie: "summer",
      resolvedCookie: "winter",
      now: new Date(2026, 2, 1),
    });

    expect(result).toEqual({ mode: "summer", theme: "summer" });
  });
});
