import React from "react";

import { act, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { LandingIntroOverlay } from "@/components/landing/landing-intro-overlay";

const pushMock = vi.fn();
let reduceMotion = true;

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
    useReducedMotion: () => reduceMotion,
  };
});

describe("LandingIntroOverlay", () => {
  beforeEach(() => {
    pushMock.mockReset();
    reduceMotion = true;
    vi.useRealTimers();
    window.localStorage.clear();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

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

  it("plays intro once per day and skips animation on repeat visits that day", async () => {
    reduceMotion = false;
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-02-28T10:00:00.000Z"));

    const { unmount } = render(<LandingIntroOverlay />);

    // First visit of the day follows cinematic timing.
    expect(screen.queryByRole("button", { name: "Let's begin!" })).not.toBeInTheDocument();
    await act(async () => {
      vi.advanceTimersByTime(4_500);
    });
    expect(screen.getByRole("button", { name: "Let's begin!" })).toBeInTheDocument();

    unmount();
    render(<LandingIntroOverlay />);

    // Second visit same day should skip to immediate CTA.
    expect(screen.getByRole("button", { name: "Let's begin!" })).toBeInTheDocument();
  });
});
