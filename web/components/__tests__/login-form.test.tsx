import React from "react";

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { LocaleProvider } from "@/components/locale-provider";
import { LoginForm } from "@/components/login-form";
import { isPasskeySupported, signInWithPasskey } from "@/lib/passkey";

const pushMock = vi.fn();

vi.mock("next/navigation", async () => {
  const actual = await vi.importActual<typeof import("next/navigation")>("next/navigation");
  return {
    ...actual,
    useRouter: () => ({ push: pushMock }),
  };
});

vi.mock("@/lib/passkey", () => ({
  isPasskeySupported: vi.fn(),
  signInWithPasskey: vi.fn(),
}));

function renderLoginForm() {
  return render(
    <LocaleProvider initialLocale="en">
      <LoginForm />
    </LocaleProvider>,
  );
}

describe("LoginForm", () => {
  const fetchMock = vi.fn<typeof fetch>();

  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("fetch", fetchMock);
    vi.mocked(isPasskeySupported).mockReturnValue(true);
    fetchMock.mockResolvedValue(
      new Response(JSON.stringify({ password: true, passkey: true, magic_link: true, otp: false }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    );
    vi.mocked(signInWithPasskey).mockResolvedValue({ ok: false, error: "No passkey registered for this account" });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("starts in email entry mode", () => {
    renderLoginForm();

    expect(screen.getByLabelText("Email")).toBeInTheDocument();
    expect(screen.queryByLabelText("Password")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Continue" })).toBeInTheDocument();
  });

  it("shows passkey waiting state after continue", async () => {
    vi.mocked(signInWithPasskey).mockImplementationOnce(
      () =>
        new Promise(() => {
          // keep pending to preserve waiting state
        }),
    );

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "passkey@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(screen.getByText("Waiting for passkey...")).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: "Show alternative methods" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Return to sign in" })).toBeInTheDocument();
    expect(screen.queryByLabelText("Email")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Continue" })).not.toBeInTheDocument();
    expect(signInWithPasskey).toHaveBeenCalledWith(
      "",
      { email: "passkey@example.com", rememberMe: false },
      {},
    );
  });

  it("shows alternative methods on demand", async () => {
    vi.mocked(signInWithPasskey).mockImplementationOnce(
      () =>
        new Promise(() => {
          // keep pending to preserve waiting state
        }),
    );

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "user@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Show alternative methods" })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole("button", { name: "Show alternative methods" }));

    expect(screen.getByRole("button", { name: "Sign in with password" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Send one time link to my email" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Return to sign in" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Return to passkey" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Sign in with OTP" })).not.toBeInTheDocument();
  });

  it("returns to sign-in form from passkey and alternatives screens", async () => {
    vi.mocked(signInWithPasskey).mockImplementation(
      () =>
        new Promise(() => {
          // keep pending to preserve waiting state
        }),
    );

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "user@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Return to sign in" })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole("button", { name: "Return to sign in" }));

    expect(screen.getByLabelText("Email")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Continue" })).toBeInTheDocument();
    expect(screen.queryByText("Waiting for passkey...")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Show alternative methods" })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole("button", { name: "Show alternative methods" }));
    fireEvent.click(screen.getByRole("button", { name: "Return to sign in" }));

    expect(screen.getByLabelText("Email")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Continue" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Sign in with password" })).not.toBeInTheDocument();
  });

  it("supports password fallback sign-in", async () => {
    fetchMock
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ password: true, passkey: true, magic_link: true, otp: false }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ user: { id: "u1" } }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      );

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "user@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Sign in with password" })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole("button", { name: "Sign in with password" }));

    fireEvent.change(screen.getByLabelText("Password"), { target: { value: "SuperSecret123" } });
    fireEvent.click(screen.getByRole("button", { name: "Sign in" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(2);
    });

    const [loginUrl, loginInit] = fetchMock.mock.calls[1] as [string, RequestInit];
    expect(loginUrl).toBe("/api/auth/login");
    expect(loginInit.method).toBe("POST");
    expect(loginInit.credentials).toBe("include");
    expect(JSON.parse(String(loginInit.body))).toEqual({
      email: "user@example.com",
      password: "SuperSecret123",
      remember_me: false,
    });
  });

  it("surfaces no-passkey-registered errors instead of silently suppressing them", async () => {
    vi.mocked(signInWithPasskey).mockResolvedValueOnce({
      ok: false,
      error: "No passkey registered for this account",
    });

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "no-passkey@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(screen.getByText("No passkey registered for this account")).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: "Sign in with password" })).toBeInTheDocument();
  });

  it("surfaces passkey security failures instead of silently skipping", async () => {
    vi.mocked(signInWithPasskey).mockResolvedValueOnce({
      ok: false,
      error: "Passkey security check failed: Invalid domain",
    });

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "security@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(screen.getByText("Passkey security check failed: Invalid domain")).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: "Sign in with password" })).toBeInTheDocument();
  });

  it("requests OTP and verifies it when capability is enabled", async () => {
    fetchMock
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ password: true, passkey: true, magic_link: true, otp: true }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ message: "If eligible, a sign-in code has been sent" }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ user: { id: "u1" } }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      );

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "otp@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Sign in with OTP" })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole("button", { name: "Sign in with OTP" }));

    await waitFor(() => {
      expect(screen.getByLabelText("One-time code")).toBeInTheDocument();
    });

    fireEvent.change(screen.getByLabelText("One-time code"), { target: { value: "123456" } });
    fireEvent.click(screen.getByRole("button", { name: "Verify OTP" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(3);
    });

    const [requestUrl] = fetchMock.mock.calls[1] as [string, RequestInit];
    expect(requestUrl).toBe("/api/auth/request-signin-otp");

    const [verifyUrl, verifyInit] = fetchMock.mock.calls[2] as [string, RequestInit];
    expect(verifyUrl).toBe("/api/auth/verify-signin-otp");
    expect(JSON.parse(String(verifyInit.body))).toEqual({
      email: "otp@example.com",
      code: "123456",
      remember_me: false,
    });
  });

  it("shows magic-link specific fallback error when one-time link request fails", async () => {
    fetchMock
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ password: true, passkey: true, magic_link: true, otp: false }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      )
      .mockResolvedValueOnce(
        new Response(null, {
          status: 500,
        }),
      );

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "magiclink@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Send one time link to my email" })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole("button", { name: "Send one time link to my email" }));

    await waitFor(() => {
      expect(screen.getByText("Could not request sign-in link.")).toBeInTheDocument();
    });
  });
});
