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
    vi.mocked(signInWithPasskey).mockResolvedValue({ ok: false, error: "No passkey registered for this account" });
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("starts in email-only mode with Continue button", () => {
    renderLoginForm();

    expect(screen.getByLabelText("Email")).toBeInTheDocument();
    expect(screen.queryByLabelText("Password")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Continue" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Sign in" })).not.toBeInTheDocument();
  });

  it("falls back to password mode when no passkey exists", async () => {
    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "USER@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(screen.getByLabelText("Password")).toBeInTheDocument();
    });
    expect(signInWithPasskey).toHaveBeenCalledWith(
      "",
      { email: "user@example.com", rememberMe: false },
      expect.objectContaining({ onPromptStart: expect.any(Function) }),
    );
    expect(screen.getByRole("button", { name: "Sign in" })).toBeInTheDocument();
    expect(screen.queryByText("No passkey registered for this account")).not.toBeInTheDocument();
  });

  it("attempts passkey sign-in and navigates on success", async () => {
    vi.mocked(signInWithPasskey).mockResolvedValueOnce({ ok: true });

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "passkey@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(signInWithPasskey).toHaveBeenCalledWith(
        "",
        { email: "passkey@example.com", rememberMe: false },
        expect.objectContaining({ onPromptStart: expect.any(Function) }),
      );
    });
    expect(screen.queryByLabelText("Password")).not.toBeInTheDocument();
  });

  it("keeps password hidden after passkey prompt starts", async () => {
    vi.mocked(signInWithPasskey).mockImplementationOnce(
      (_apiBaseUrl, _params, hooks) =>
        new Promise(() => {
          hooks?.onPromptStart?.();
        }),
    );

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "pending@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    expect(screen.getByRole("button", { name: "Continuing..." })).toBeInTheDocument();
    await new Promise((resolve) => setTimeout(resolve, 1300));
    expect(screen.queryByLabelText("Password")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Sign in with password" })).toBeInTheDocument();
  });

  it("keeps waiting when passkey prompt bootstrap is still in flight", async () => {
    vi.mocked(signInWithPasskey).mockImplementationOnce(
      (_apiBaseUrl, _params, hooks) =>
        new Promise(() => {
          window.setTimeout(() => {
            hooks?.onPromptStart?.();
          }, 500);
        }),
    );

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "slow-options@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    expect(screen.getByRole("button", { name: "Continuing..." })).toBeInTheDocument();
    await new Promise((resolve) => setTimeout(resolve, 400));
    expect(screen.queryByLabelText("Password")).not.toBeInTheDocument();
  });

  it("shows password quickly when passkey prompt never starts", async () => {
    vi.mocked(signInWithPasskey).mockImplementationOnce(
      () =>
        new Promise(() => {
          // Intentionally never resolve and never trigger onPromptStart.
        }),
    );

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "no-prompt@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(screen.getByLabelText("Password")).toBeInTheDocument();
    }, { timeout: 1400 });
    expect(screen.getByRole("button", { name: "Sign in" })).toBeInTheDocument();
  });

  it("allows explicit password fallback after passkey prompt starts", async () => {
    let resolvePasskey: ((value: { ok: boolean; error?: string }) => void) | undefined;
    vi.mocked(signInWithPasskey).mockImplementationOnce(
      (_apiBaseUrl, _params, hooks) =>
        new Promise((resolve) => {
          hooks?.onPromptStart?.();
          resolvePasskey = resolve;
        }),
    );

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "late-passkey@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    await waitFor(() => {
      expect(signInWithPasskey).toHaveBeenCalledTimes(1);
    });
    fireEvent.click(screen.getByRole("button", { name: "Sign in with password" }));
    expect(screen.getByLabelText("Password")).toBeInTheDocument();

    fetchMock.mockResolvedValueOnce(
      new Response(JSON.stringify({ user: { id: "u1" } }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    );
    resolvePasskey?.({ ok: true });
    await Promise.resolve();
    fireEvent.change(screen.getByLabelText("Password"), { target: { value: "SuperSecret123" } });
    fireEvent.click(screen.getByRole("button", { name: "Sign in" }));
    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(1);
    });
  });

  it("shows password mode immediately when passkeys are unsupported", async () => {
    vi.mocked(isPasskeySupported).mockReturnValueOnce(false);

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "fallback@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(screen.getByLabelText("Password")).toBeInTheDocument();
    });
    expect(signInWithPasskey).not.toHaveBeenCalled();
  });

  it("preserves entered email in forgot-password link query", async () => {
    vi.mocked(isPasskeySupported).mockReturnValueOnce(false);
    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "user+tag@example.com " } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(screen.getByLabelText("Password")).toBeInTheDocument();
    });

    const forgotPasswordLink = screen.getByRole("link", { name: "Forgot password?" });
    expect(forgotPasswordLink).toHaveAttribute("href", "/forgot-password?email=user%2Btag%40example.com");
  });

  it("validates password sign-in after passkey fallback", async () => {
    fetchMock.mockResolvedValueOnce(
      new Response(JSON.stringify({ user: { id: "u1" } }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    );

    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "login@example.com" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(screen.getByLabelText("Password")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByLabelText("Remember me"));
    fireEvent.change(screen.getByLabelText("Password"), { target: { value: "SuperSecret123" } });
    fireEvent.click(screen.getByRole("button", { name: "Sign in" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(1);
    });

    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(url).toBe("/api/auth/login");
    expect(init.method).toBe("POST");
    expect(init.credentials).toBe("include");
    expect(JSON.parse(String(init.body))).toEqual({
      email: "login@example.com",
      password: "SuperSecret123",
      remember_me: true,
    });
  });

  it("shows email validation errors before passkey attempt", async () => {
    renderLoginForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "invalid-email" } });
    fireEvent.submit(screen.getByRole("button", { name: "Continue" }).closest("form") as HTMLFormElement);

    await waitFor(() => {
      expect(screen.getByText("Please enter a valid email.")).toBeInTheDocument();
    });
    expect(signInWithPasskey).not.toHaveBeenCalled();
  });
});
