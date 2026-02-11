"use client";

import { useEffect, useMemo, useRef, useState } from "react";

import { LandingIntroOverlay } from "@/components/landing/landing-intro-overlay";
import { useTheme } from "@/components/theme-provider";

type Direction = "forward" | "reverse";

const LANDING_VIDEO_BASE_URL =
  process.env.NEXT_PUBLIC_LANDING_VIDEO_BASE_URL ??
  "https://github.com/jason931225/bominal.github.io/raw/refs/heads/main/public/video";

export function LandingHeroVideo() {
  const { theme } = useTheme();
  const forwardSrc = useMemo(() => `${LANDING_VIDEO_BASE_URL}/${theme}.mp4`, [theme]);
  const reverseSrc = useMemo(() => `${LANDING_VIDEO_BASE_URL}/${theme}-rev.mp4`, [theme]);

  const forwardRef = useRef<HTMLVideoElement | null>(null);
  const reverseRef = useRef<HTMLVideoElement | null>(null);
  const [active, setActive] = useState<Direction>("forward");

  useEffect(() => {
    // Reset loop when theme changes.
    setActive("forward");
  }, [forwardSrc, reverseSrc]);

  useEffect(() => {
    const forwardVideo = forwardRef.current;
    const reverseVideo = reverseRef.current;
    if (!forwardVideo || !reverseVideo) return;

    const activeVideo = active === "forward" ? forwardVideo : reverseVideo;
    const inactiveVideo = active === "forward" ? reverseVideo : forwardVideo;

    // Keep the inactive video stopped at the start so the crossfade looks clean.
    inactiveVideo.pause();
    try {
      inactiveVideo.currentTime = 0;
    } catch {
      // Ignore if the browser refuses to seek before metadata is loaded.
    }

    try {
      activeVideo.currentTime = 0;
    } catch {
      // Ignore if the browser refuses to seek before metadata is loaded.
    }

    const playPromise = activeVideo.play();
    if (playPromise) {
      playPromise.catch(() => null);
    }
  }, [active, forwardSrc, reverseSrc]);

  const onEnded = () => {
    setActive((current) => (current === "forward" ? "reverse" : "forward"));
  };

  return (
    <section className="relative h-[100dvh] w-full overflow-hidden bg-[rgb(var(--bg-base-start))]">
      <div className="absolute inset-0">
        <video
          ref={forwardRef}
          src={forwardSrc}
          muted
          playsInline
          preload="auto"
          disablePictureInPicture
          onEnded={active === "forward" ? onEnded : undefined}
          className={`absolute inset-0 h-full w-full object-cover object-center transition-opacity duration-200 ease-out ${active === "forward" ? "opacity-100" : "opacity-0"}`}
        />
        <video
          ref={reverseRef}
          src={reverseSrc}
          muted
          playsInline
          preload="auto"
          disablePictureInPicture
          onEnded={active === "reverse" ? onEnded : undefined}
          className={`absolute inset-0 h-full w-full object-cover object-center transition-opacity duration-200 ease-out ${active === "reverse" ? "opacity-100" : "opacity-0"}`}
        />

        <div className="pointer-events-none absolute inset-0 bg-gradient-to-b from-black/15 via-black/10 to-black/25" />
      </div>

      <LandingIntroOverlay />
    </section>
  );
}
