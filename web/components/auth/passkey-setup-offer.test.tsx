import React from "react";

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { PasskeySetupOffer } from "@/components/auth/passkey-setup-offer";
import { LocaleProvider } from "@/components/locale-provider";
import { listPasskeysFromSession, registerPasskeyFromSession } from "@/lib/passkey";

const replaceMock = vi.fn();

vi.mock("next/navigation", async () => {
  const actual = await vi.importActual<typeof import("next/navigation")>("next/navigation");
  return {
    ...actual,
    useRouter: () => ({ replace: replaceMock }),
  };
});

vi.mock("@/lib/passkey", () => ({
  listPasskeysFromSession: vi.fn(),
  registerPasskeyFromSession: vi.fn(),
}));

function renderOffer(source: "signup" | "reset" | "magiclink" | "unknown" = "signup") {
  return render(
    <LocaleProvider initialLocale="en">
      <PasskeySetupOffer source={source} nextPath="/modules/train" />
    </LocaleProvider>,
  );
}

describe("PasskeySetupOffer", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(listPasskeysFromSession).mockResolvedValue({ credentials: [] });
    vi.mocked(registerPasskeyFromSession).mockResolvedValue({ ok: true });
  });

  it("redirects immediately when passkeys already exist", async () => {
    vi.mocked(listPasskeysFromSession).mockResolvedValueOnce({
      credentials: [{ id: "pk1", created_at: "2026-02-28T00:00:00Z", last_used_at: null }],
    });

    renderOffer();

    await waitFor(() => {
      expect(replaceMock).toHaveBeenCalledWith("/modules/train");
    });
  });

  it("renders add/skip actions when no passkey exists", async () => {
    renderOffer();

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Add passkey now" })).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: "Skip for now" })).toBeInTheDocument();
  });

  it("adds passkey and redirects on success", async () => {
    renderOffer("magiclink");

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Add passkey now" })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole("button", { name: "Add passkey now" }));

    await waitFor(() => {
      expect(registerPasskeyFromSession).toHaveBeenCalledWith("");
      expect(replaceMock).toHaveBeenCalledWith("/modules/train");
    });
  });
});
