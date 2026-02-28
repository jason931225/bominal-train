import React from "react";

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { LocaleProvider } from "@/components/locale-provider";
import { RegisterForm } from "@/components/register-form";
const pushMock = vi.fn();

vi.mock("next/navigation", async () => {
  const actual = await vi.importActual<typeof import("next/navigation")>("next/navigation");
  return {
    ...actual,
    useRouter: () => ({ push: pushMock }),
  };
});

function renderRegisterForm() {
  return render(
    <LocaleProvider initialLocale="en">
      <RegisterForm />
    </LocaleProvider>,
  );
}

describe("RegisterForm", () => {
  const fetchMock = vi.fn<typeof fetch>();

  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("blocks submit when password confirmation does not match", async () => {
    renderRegisterForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "new-user@example.com" } });
    fireEvent.change(screen.getByLabelText("Display name"), { target: { value: "New User" } });
    fireEvent.change(screen.getByLabelText("Password"), { target: { value: "SuperSecret123" } });
    fireEvent.change(screen.getByLabelText("Confirm new password"), { target: { value: "DifferentSecret123" } });

    fireEvent.click(screen.getByRole("button", { name: "Create account" }));

    await waitFor(() => {
      expect(screen.getByTestId("register-confirm-password-error")).toBeInTheDocument();
    });
    expect(screen.getByTestId("register-confirm-password-error")).toHaveTextContent(
      "New password confirmation does not match.",
    );
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("submits when password confirmation matches", async () => {
    fetchMock.mockResolvedValueOnce(
      new Response(JSON.stringify({ user: { id: "u1" } }), {
        status: 201,
        headers: { "Content-Type": "application/json" },
      }),
    );
    fetchMock.mockResolvedValueOnce(
      new Response(JSON.stringify({ user: { id: "u1" } }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    );

    renderRegisterForm();

    fireEvent.change(screen.getByLabelText("Email"), { target: { value: "new-user@example.com" } });
    fireEvent.change(screen.getByLabelText("Display name"), { target: { value: "New User" } });
    fireEvent.change(screen.getByLabelText("Password"), { target: { value: "SuperSecret123" } });
    fireEvent.change(screen.getByLabelText("Confirm new password"), { target: { value: "SuperSecret123" } });

    fireEvent.click(screen.getByRole("button", { name: "Create account" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(2);
    });
    const [registerUrl, registerInit] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(registerUrl).toBe("/api/auth/register");
    expect(registerInit.method).toBe("POST");
    expect(registerInit.credentials).toBe("include");
    expect(JSON.parse(String(registerInit.body))).toEqual({
      email: "new-user@example.com",
      password: "SuperSecret123",
      display_name: "New User",
    });

    const [loginUrl, loginInit] = fetchMock.mock.calls[1] as [string, RequestInit];
    expect(loginUrl).toBe("/api/auth/login");
    expect(loginInit.method).toBe("POST");
    expect(loginInit.credentials).toBe("include");
    expect((loginInit.headers as Record<string, string>)["x-bominal-flow-source"]).toBe("signup");
    expect(JSON.parse(String(loginInit.body))).toEqual({
      email: "new-user@example.com",
      password: "SuperSecret123",
      remember_me: true,
    });
  });
});
