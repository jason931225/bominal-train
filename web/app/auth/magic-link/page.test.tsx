import React from "react";

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { LocaleProvider } from "@/components/locale-provider";
import AuthMagicLinkPage from "@/app/auth/magic-link/page";

const replaceMock = vi.fn();
let currentSearchParams = new URLSearchParams();

vi.mock("next/navigation", async () => {
  const actual = await vi.importActual<typeof import("next/navigation")>("next/navigation");
  return {
    ...actual,
    useRouter: () => ({ replace: replaceMock }),
    useSearchParams: () => currentSearchParams,
  };
});

function renderPage() {
  return render(
    <LocaleProvider initialLocale="en">
      <AuthMagicLinkPage />
    </LocaleProvider>,
  );
}

describe("AuthMagicLinkPage", () => {
  const fetchMock = vi.fn<typeof fetch>();

  beforeEach(() => {
    vi.clearAllMocks();
    currentSearchParams = new URLSearchParams("email=magic@example.com&code=654321");
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("submits legacy magic-link confirm on continue", async () => {
    fetchMock.mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          user: { id: "u1" },
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" },
        },
      ),
    );

    renderPage();
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(1);
    });
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/auth/magic-link/confirm",
      expect.objectContaining({
        method: "POST",
        credentials: "include",
      }),
    );
    expect(replaceMock).toHaveBeenCalledWith("/auth/passkey/add?source=magiclink&next=%2Fmodules%2Ftrain");
  });

  it("shows invalid-link error when required params are missing", async () => {
    currentSearchParams = new URLSearchParams("email=magic@example.com");
    renderPage();

    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(screen.getByText("Missing link parameters. Request a new sign-in or recovery email.")).toBeInTheDocument();
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });
});
