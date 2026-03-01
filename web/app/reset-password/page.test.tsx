import React from "react";

import { describe, expect, it, vi } from "vitest";

const redirectMock = vi.fn((target: string) => {
  throw new Error(`REDIRECT:${target}`);
});

const getOptionalUserMock = vi.fn();
const postLoginRouteForUserMock = vi.fn();
const cookiesMock = vi.fn();

vi.mock("next/navigation", () => ({
  redirect: redirectMock,
}));

vi.mock("next/headers", () => ({
  cookies: cookiesMock,
}));

vi.mock("@/lib/server-auth", () => ({
  getOptionalUser: getOptionalUserMock,
  postLoginRouteForUser: postLoginRouteForUserMock,
}));

vi.mock("@/lib/i18n-server", () => ({
  getServerT: async () => ({
    t: (key: string) => key,
  }),
}));

function cookieStoreWith(
  values: Partial<{
    bominal_supabase_recovery_ctx: string;
    bominal_passkey_setup_ctx: string;
  }>,
) {
  return {
    get: (name: string) => {
      const value = values[name as keyof typeof values];
      return value ? { value } : undefined;
    },
  };
}

describe("ResetPasswordPage", () => {
  it("redirects authenticated users without recovery context", async () => {
    vi.resetModules();
    redirectMock.mockClear();
    getOptionalUserMock.mockResolvedValueOnce({ id: "u1", email: "user@example.com" });
    postLoginRouteForUserMock.mockReturnValueOnce("/modules/train");
    cookiesMock.mockResolvedValueOnce(cookieStoreWith({}));

    const { default: ResetPasswordPage } = await import("@/app/reset-password/page");

    await expect(ResetPasswordPage({ searchParams: Promise.resolve({}) })).rejects.toThrow("REDIRECT:/modules/train");
  });

  it("does not redirect authenticated users while recovery context exists", async () => {
    vi.resetModules();
    redirectMock.mockClear();
    getOptionalUserMock.mockResolvedValueOnce({ id: "u1", email: "user@example.com" });
    postLoginRouteForUserMock.mockReturnValueOnce("/modules/train");
    cookiesMock.mockResolvedValueOnce(
      cookieStoreWith({
        bominal_supabase_recovery_ctx: "ctx-token",
      }),
    );

    const { default: ResetPasswordPage } = await import("@/app/reset-password/page");
    const result = await ResetPasswordPage({ searchParams: Promise.resolve({}) });

    expect(result).toBeTruthy();
    expect(redirectMock).not.toHaveBeenCalled();
  });

  it("routes authenticated users with post-reset passkey context to passkey", async () => {
    vi.resetModules();
    redirectMock.mockClear();
    getOptionalUserMock.mockResolvedValueOnce({ id: "u1", email: "user@example.com" });
    postLoginRouteForUserMock.mockReturnValueOnce("/modules/train");
    cookiesMock.mockResolvedValueOnce(
      cookieStoreWith({
        bominal_passkey_setup_ctx: "ctx-passkey",
      }),
    );

    const { default: ResetPasswordPage } = await import("@/app/reset-password/page");

    await expect(ResetPasswordPage({ searchParams: Promise.resolve({}) })).rejects.toThrow(
      "REDIRECT:/auth/passkey/add?source=reset&next=%2Fmodules%2Ftrain",
    );
  });
});
