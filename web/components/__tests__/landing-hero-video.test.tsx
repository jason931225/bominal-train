import React from "react";

import { fireEvent, render, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { LandingHeroVideo } from "@/components/landing/landing-hero-video";

vi.mock("@/components/theme-provider", () => ({
  useTheme: () => ({ theme: "winter" }),
}));

vi.mock("@/components/landing/landing-intro-overlay", () => ({
  LandingIntroOverlay: () => null,
}));

describe("LandingHeroVideo", () => {
  beforeEach(() => {
    vi.spyOn(HTMLMediaElement.prototype, "play").mockResolvedValue(undefined);
    vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => undefined);
  });

  it("does not rewind the fading-out clip when switching direction", async () => {
    const { container } = render(<LandingHeroVideo />);

    const videos = Array.from(container.querySelectorAll("video")) as HTMLVideoElement[];
    expect(videos).toHaveLength(2);

    const forwardVideo = videos[0];

    forwardVideo.currentTime = 17;
    fireEvent.ended(forwardVideo);

    await waitFor(() => {
      expect(forwardVideo.className).toContain("opacity-0");
    });

    expect(forwardVideo.currentTime).toBe(17);
  });
});
