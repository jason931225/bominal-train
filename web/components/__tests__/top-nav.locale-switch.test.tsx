import React from "react";

import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { LocaleProvider } from "@/components/locale-provider";
import { TopNav } from "@/components/top-nav";
import type { BominalUser } from "@/lib/types";

const refreshMock = vi.fn();

vi.mock("next/navigation", async () => {
  const actual = await vi.importActual<typeof import("next/navigation")>("next/navigation");
  return {
    ...actual,
    usePathname: () => "/modules/train",
    useRouter: () => ({ refresh: refreshMock }),
  };
});

vi.mock("@/components/top-nav-brand", () => ({
  TopNavBrand: ({ sectionLabel }: { sectionLabel: string | null }) => <div>{sectionLabel ?? ""}</div>,
}));

vi.mock("@/components/top-nav-task-attention", () => ({
  TopNavTaskAttention: () => <div data-testid="top-nav-attention" />,
}));

vi.mock("@/components/nav-burger-menu", () => ({
  NavBurgerMenu: () => <div data-testid="top-nav-menu" />,
}));

const USER: BominalUser = {
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
  created_at: "2026-02-27T00:00:00Z",
};

describe("TopNav locale switch", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("switches nav copy between ENG and KOR for guests", async () => {
    render(
      <LocaleProvider initialLocale="en">
        <TopNav user={null} />
      </LocaleProvider>,
    );

    expect(screen.queryByRole("link", { name: "Login" })).not.toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "Register" })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "ENG" })).toHaveAttribute("aria-pressed", "true");
    fireEvent.click(screen.getByRole("button", { name: "KOR" }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "KOR" })).toHaveAttribute("aria-pressed", "true");
    });
  });

  it("persists locale for signed-in users and refreshes", async () => {
    const fetchMock = vi.fn<typeof fetch>().mockResolvedValue(
      new Response(JSON.stringify({ user: { id: USER.id } }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    );
    vi.stubGlobal("fetch", fetchMock);

    render(
      <LocaleProvider initialLocale="en">
        <TopNav user={USER} />
      </LocaleProvider>,
    );

    fireEvent.click(screen.getByRole("button", { name: "KOR" }));

    await waitFor(() => {
      expect(fetchMock).toHaveBeenCalledTimes(1);
    });

    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(url).toBe("/api/auth/account");
    expect(init.method).toBe("PATCH");
    expect(init.credentials).toBe("include");
    expect(JSON.parse(String(init.body))).toEqual({ ui_locale: "ko" });
    expect(refreshMock).toHaveBeenCalledTimes(1);
  });
});
