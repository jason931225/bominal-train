import React from "react";

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { LocaleProvider } from "@/components/locale-provider";
import AuthConfirmPage from "@/app/auth/confirm/page";

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
      <AuthConfirmPage />
    </LocaleProvider>,
  );
}

describe("AuthConfirmPage", () => {
  const fetchMock = vi.fn<typeof fetch>();

  beforeEach(() => {
    vi.clearAllMocks();
    currentSearchParams = new URLSearchParams("token_hash=hash-abc123&type=email");
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("does not call supabase confirm on initial render", async () => {
    renderPage();

    await new Promise((resolve) => setTimeout(resolve, 30));
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("calls supabase confirm only after explicit continue click", async () => {
    fetchMock.mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          mode: "magiclink",
          redirect_to: "/modules/train",
          access_token: "access-token-123",
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" },
        },
      ),
    );

    renderPage();
    await new Promise((resolve) => setTimeout(resolve, 30));
    expect(fetchMock).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(1);
    });
    expect(replaceMock).toHaveBeenCalledWith("/modules/train");
  });

  it("falls back to passkey route when magiclink confirm has no redirect target", async () => {
    fetchMock.mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          mode: "magiclink",
          redirect_to: "",
          access_token: "access-token-123",
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
    expect(replaceMock).toHaveBeenCalledWith("/auth/passkey/add?source=magiclink&next=%2Fmodules%2Ftrain");
  });

  it("routes recovery callbacks to reset-password without mode query", async () => {
    currentSearchParams = new URLSearchParams("token_hash=hash-abc123&type=recovery");
    fetchMock.mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          mode: "recovery",
          redirect_to: "https://www.bominal.com/reset-password",
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
    expect(replaceMock).toHaveBeenCalledWith("/reset-password");
  });
});
