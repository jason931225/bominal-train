import React from "react";

import { render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { LandingIntroOverlay } from "@/components/landing/landing-intro-overlay";

const pushMock = vi.fn();

vi.mock("next/navigation", async () => {
  const actual = await vi.importActual<typeof import("next/navigation")>("next/navigation");
  return {
    ...actual,
    useRouter: () => ({ push: pushMock }),
  };
});

vi.mock("framer-motion", async () => {
  const actual = await vi.importActual<typeof import("framer-motion")>("framer-motion");
  return {
    ...actual,
    useReducedMotion: () => true,
  };
});

describe("LandingIntroOverlay", () => {
  it("applies gradient text classes on each animated letter span", async () => {
    const { container } = render(<LandingIntroOverlay />);

    await waitFor(() => {
      expect(container.querySelector("div.font-brand")).toBeInTheDocument();
    });

    const letterSpans = Array.from(container.querySelectorAll("div.font-brand span.inline-block"));
    expect(letterSpans).toHaveLength(7);

    for (const letterSpan of letterSpans) {
      expect(letterSpan.className).toContain("sm:text-transparent");
      expect(letterSpan.className).toContain("sm:bg-clip-text");
      expect(letterSpan.className).toContain("sm:bg-gradient-to-b");
    }
  });
});
