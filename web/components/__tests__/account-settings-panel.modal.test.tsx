import React from "react";

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { AccountSettingsPanel } from "@/components/account/account-settings-panel";
import { LocaleProvider } from "@/components/locale-provider";
import type { BominalUser } from "@/lib/types";

const refreshMock = vi.fn();
const pushMock = vi.fn();

vi.mock("next/navigation", async () => {
  const actual = await vi.importActual<typeof import("next/navigation")>("next/navigation");
  return {
    ...actual,
    useRouter: () => ({ refresh: refreshMock, push: pushMock }),
  };
});

vi.mock("@/lib/passkey", () => ({
  listPasskeysFromSession: vi.fn().mockResolvedValue({ credentials: [] }),
  registerPasskeyFromSession: vi.fn().mockResolvedValue({ ok: false }),
  removePasskeyFromSession: vi.fn(),
  verifyPasskeyStepUpFromSession: vi.fn().mockResolvedValue({ ok: false, stepUpToken: null }),
}));

const INITIAL_USER: BominalUser = {
  id: "user-1",
  email: "user@example.com",
  display_name: "User",
  phone_number: null,
  ui_locale: "en",
  billing_address: null,
  billing_address_line1: null,
  billing_address_line2: null,
  billing_city: null,
  billing_state_province: null,
  billing_country: null,
  billing_postal_code: null,
  birthday: null,
  role: "user",
  access_status: "approved",
  access_reviewed_at: null,
  created_at: "2026-02-26T00:00:00Z",
};

function renderPanel() {
  return render(
    <LocaleProvider initialLocale="en">
      <AccountSettingsPanel initialUser={INITIAL_USER} />
    </LocaleProvider>,
  );
}

describe("AccountSettingsPanel modal overlay", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("renders the sensitive-action overlay outside the component container", async () => {
    const { container } = renderPanel();

    fireEvent.click(screen.getByRole("button", { name: "Add passkey" }));

    await waitFor(() => {
      expect(screen.getByText("Verify to continue")).toBeInTheDocument();
    });

    const overlay = document.querySelector("div[class*='bg-slate-900/45']");
    expect(overlay).not.toBeNull();
    expect(container.contains(overlay)).toBe(false);
  });
});
