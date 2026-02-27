import React from "react";

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { PasswordResetConfirmForm } from "@/components/auth/password-reset-confirm-form";
import { LocaleProvider } from "@/components/locale-provider";
import { clearSupabaseAccessToken, getSupabaseAccessToken } from "@/lib/supabase-auth";

const pushMock = vi.fn();

vi.mock("next/navigation", async () => {
  const actual = await vi.importActual<typeof import("next/navigation")>("next/navigation");
  return {
    ...actual,
    useRouter: () => ({ push: pushMock }),
  };
});

vi.mock("@/lib/supabase-auth", () => ({
  getSupabaseAccessToken: vi.fn(),
  clearSupabaseAccessToken: vi.fn(),
}));

describe("PasswordResetConfirmForm", () => {
  const fetchMock = vi.fn<typeof fetch>();

  function renderSupabaseMode() {
    return render(
      <LocaleProvider initialLocale="en">
        <PasswordResetConfirmForm mode="supabase" />
      </LocaleProvider>,
    );
  }

  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("submits supabase reset endpoint with bearer token in supabase mode", async () => {
    vi.mocked(getSupabaseAccessToken).mockResolvedValueOnce("recovery-access-token");
    fetchMock.mockResolvedValueOnce(
      new Response(JSON.stringify({ message: "Password reset complete" }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    );

    renderSupabaseMode();

    expect(screen.queryByLabelText("Email")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Reset code (OTP)")).not.toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("New password"), { target: { value: "NewPassword123" } });
    fireEvent.change(screen.getByLabelText("Confirm new password"), { target: { value: "NewPassword123" } });
    fireEvent.click(screen.getByRole("button", { name: "Set new password" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(1);
    });
    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(url).toBe("/api/auth/reset-password/supabase");
    expect(init.method).toBe("POST");
    expect(init.credentials).toBe("include");
    expect((init.headers as Record<string, string>).Authorization).toBe("Bearer recovery-access-token");
    expect(JSON.parse(String(init.body))).toEqual({ new_password: "NewPassword123" });
    expect(clearSupabaseAccessToken).toHaveBeenCalledTimes(1);
  });

  it("shows missing-recovery error when token is unavailable", async () => {
    vi.mocked(getSupabaseAccessToken).mockResolvedValueOnce(null);

    renderSupabaseMode();

    fireEvent.change(screen.getByLabelText("New password"), { target: { value: "NewPassword123" } });
    fireEvent.change(screen.getByLabelText("Confirm new password"), { target: { value: "NewPassword123" } });
    fireEvent.click(screen.getByRole("button", { name: "Set new password" }));

    await waitFor(() => {
      expect(screen.getByText("Your recovery session is missing or expired. Request a new reset email.")).toBeInTheDocument();
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });
});
