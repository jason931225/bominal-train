import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { isLocale, localeFromAcceptLanguage, localeFromUser, t } from "@/lib/i18n";
import { formatDateTimeKst, kstDateInputValue } from "@/lib/kst";
import { isPathPrefix, ROUTES } from "@/lib/routes";
import type { BominalUser } from "@/lib/types";

const ORIGINAL_ENV = { ...process.env };

function resetApiBaseEnv() {
  delete process.env.API_SERVER_URL;
  delete process.env.NEXT_PUBLIC_API_BASE_URL;
}

describe("i18n helpers", () => {
  it("validates locale and derives locale from accept-language/user", () => {
    expect(isLocale("en")).toBe(true);
    expect(isLocale("ko")).toBe(true);
    expect(isLocale("ja")).toBe(false);

    expect(localeFromAcceptLanguage("ko-KR,ko;q=0.9,en;q=0.8")).toBe("ko");
    expect(localeFromAcceptLanguage("en-US,en;q=0.9")).toBe("en");
    expect(localeFromAcceptLanguage(null)).toBe("en");

    const user = { ui_locale: "ko" } as BominalUser;
    const invalid = { ui_locale: "jp" } as unknown as BominalUser;
    expect(localeFromUser(user)).toBe("ko");
    expect(localeFromUser(invalid)).toBeNull();
    expect(localeFromUser(null)).toBeNull();
  });

  it("translates nested keys with interpolation and fallback behavior", () => {
    expect(t("en", "dashboard.welcome", { name: "Jason" })).toContain("Jason");
    expect(t("ko", "dashboard.welcome", { name: "Jason" })).toContain("Jason");
    expect(t("ko", "modules.available")).toBe("사용 가능");
    expect(t("ko", "this.key.does.not.exist")).toBe("this.key.does.not.exist");
    // Missing interpolation variables are replaced with an empty string.
    expect(t("en", "train.providerLoginRequired", {})).toContain("login required");
  });
});

describe("kst/date helpers", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("formats KST date input values and datetime strings", () => {
    const date = new Date("2026-02-22T00:30:00+09:00");
    expect(kstDateInputValue(date)).toBe("2026-02-22");

    const formattedKo = formatDateTimeKst("2026-02-22T13:29:06+09:00", "ko");
    const formattedEn = formatDateTimeKst("2026-02-22T13:29:06+09:00", "en");
    const formattedFromDate = formatDateTimeKst(new Date("2026-02-22T13:29:06+09:00"), "en");
    const formattedDefault = formatDateTimeKst("2026-02-22T13:29:06+09:00");
    expect(formattedKo.endsWith("KST")).toBe(true);
    expect(formattedEn.endsWith("KST")).toBe(true);
    expect(formattedFromDate.endsWith("KST")).toBe(true);
    expect(formattedDefault.endsWith("KST")).toBe(true);
    expect(formatDateTimeKst(null, "en")).toBe("-");
    expect(formatDateTimeKst("invalid-date", "en")).toBe("-");
  });

  it("throws when KST date parts cannot be resolved", () => {
    function MockDateTimeFormat() {
      return {
        formatToParts: () => [],
        format: () => "",
      } as unknown as Intl.DateTimeFormat;
    }

    const spy = vi
      .spyOn(Intl, "DateTimeFormat")
      .mockImplementation(MockDateTimeFormat as unknown as typeof Intl.DateTimeFormat);

    expect(() => kstDateInputValue(new Date("2026-02-22T00:00:00+09:00"))).toThrow(
      "Could not format KST date parts.",
    );
    expect(spy).toHaveBeenCalled();
  });
});

describe("routes helper", () => {
  it("keeps route constants stable and evaluates path prefixes", () => {
    expect(ROUTES.modules.train).toBe("/modules/train");
    expect(ROUTES.settings.payment).toBe("/settings/payment");
    expect(ROUTES.admin.maintenance).toBe("/admin/maintenance");

    expect(isPathPrefix("/modules/train", "/modules/train")).toBe(true);
    expect(isPathPrefix("/modules/train/tasks/123", "/modules/train")).toBe(true);
    expect(isPathPrefix("/modules/restaurant", "/modules/train")).toBe(false);
  });
});

describe("api base resolution", () => {
  beforeEach(() => {
    vi.resetModules();
    process.env = { ...ORIGINAL_ENV };
    resetApiBaseEnv();
  });

  afterEach(() => {
    process.env = { ...ORIGINAL_ENV };
    vi.resetModules();
  });

  it("prefers API_SERVER_URL, then NEXT_PUBLIC_API_BASE_URL, then localhost", async () => {
    process.env.API_SERVER_URL = "http://api-internal:8000";
    process.env.NEXT_PUBLIC_API_BASE_URL = "https://example.com";

    const first = await import("@/lib/api-base");
    expect(first.clientApiBaseUrl).toBe("");
    expect(first.serverApiBaseUrl).toBe("http://api-internal:8000");

    vi.resetModules();
    resetApiBaseEnv();
    process.env.NEXT_PUBLIC_API_BASE_URL = "https://public.example.com";
    const second = await import("@/lib/api-base");
    expect(second.serverApiBaseUrl).toBe("https://public.example.com");

    vi.resetModules();
    resetApiBaseEnv();
    const third = await import("@/lib/api-base");
    expect(third.serverApiBaseUrl).toBe("http://localhost:8000");
  });
});

describe("i18n fallback branches", () => {
  afterEach(() => {
    vi.doUnmock("@/messages/ko.json");
    vi.doUnmock("@/messages/en.json");
    vi.resetModules();
  });

  it("falls back to english message when locale message key is missing", async () => {
    vi.resetModules();
    vi.doMock("@/messages/ko.json", () => ({ default: {} }));
    vi.doMock("@/messages/en.json", () => ({ default: { x: { y: "Fallback EN" } } }));
    const i18nModule = await import("@/lib/i18n");
    expect(i18nModule.t("ko", "x.y")).toBe("Fallback EN");
  });
});
