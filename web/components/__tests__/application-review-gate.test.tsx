import React from "react";

import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ApplicationReviewGate } from "@/components/application-review-gate";
import { LocaleProvider } from "@/components/locale-provider";

vi.mock("next/navigation", async () => {
  const actual = await vi.importActual<typeof import("next/navigation")>("next/navigation");
  return {
    ...actual,
    useRouter: () => ({ refresh: vi.fn(), push: vi.fn() }),
  };
});

vi.mock("@/components/theme-provider", () => ({
  useTheme: () => ({ theme: "spring" }),
}));

describe("ApplicationReviewGate", () => {
  it("uses full viewport height and keeps sign-out/delete buttons the same size", () => {
    const { container } = render(
      <LocaleProvider initialLocale="en">
        <ApplicationReviewGate email="review-user@example.com" />
      </LocaleProvider>,
    );

    const rootSection = container.querySelector("section");
    expect(rootSection).not.toBeNull();
    expect(rootSection?.className).toContain("min-h-[100dvh]");
    expect(rootSection?.className).not.toContain("calc(100dvh-3.75rem)");

    const signOutButton = screen.getByRole("button", { name: "Sign out" });
    const deleteButton = screen.getByRole("button", { name: "Delete account" });
    expect(signOutButton.className).toContain("h-10");
    expect(deleteButton.className).toContain("h-10");
  });
});
