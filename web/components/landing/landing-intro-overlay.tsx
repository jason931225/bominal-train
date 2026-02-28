"use client";

import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { useRouter } from "next/navigation";
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";

import { UI_LIQUID_GLASS_TEXT_WHITE, UI_LIQUID_GLASS_WHITE } from "@/lib/ui";

const WORDMARK = "bominal";

const LOGO_DELAY_MS = 800;
// Cinematic pacing (roughly 3.5s from the first letter to the CTA fully appearing).
const LETTER_STAGGER_MS = 130;
const LETTER_DURATION_MS = 620;
const LOGO_HOLD_MS = 700;
const BUTTON_ENTER_DURATION_MS = 600;
const SKIP_ENTER_DURATION_MS = 180;
const CTA_GAP_PX = 28;
const LANDING_INTRO_PLAYED_AT_KEY = "bominal_landing_intro_played_at_v1";

type IntroMode = "boot" | "play" | "skip";

const LANDING_WORDMARK_CLASS =
  "select-none font-brand whitespace-nowrap text-[clamp(3.5rem,12vw,7rem)] font-semibold lowercase leading-none tracking-tight drop-shadow-[0_18px_50px_rgba(0,0,0,0.45)]";
const LANDING_WORDMARK_LETTER_TEXT_CLASS =
  "text-white/80 sm:text-transparent sm:bg-clip-text sm:bg-gradient-to-b sm:from-white/95 sm:via-white/75 sm:to-white/45";

function currentLocalDayKey(): string {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function hasPlayedIntroToday(): boolean {
  if (typeof window === "undefined") return false;
  try {
    return window.localStorage.getItem(LANDING_INTRO_PLAYED_AT_KEY) === currentLocalDayKey();
  } catch {
    return false;
  }
}

function markPlayedIntroToday(): void {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(LANDING_INTRO_PLAYED_AT_KEY, currentLocalDayKey());
  } catch {
    // Ignore storage write failures and fall back to replay behavior.
  }
}

export function LandingIntroOverlay() {
  const router = useRouter();
  const reduceMotion = useReducedMotion();

  const letters = useMemo(() => Array.from(WORDMARK), []);
  const wordmarkRef = useRef<HTMLDivElement | null>(null);
  const [mode, setMode] = useState<IntroMode>("boot");
  const [logoStarted, setLogoStarted] = useState(false);
  const [ctaVisible, setCtaVisible] = useState(false);
  const [ctaOffsetPx, setCtaOffsetPx] = useState(0);

  useEffect(() => {
    // Avoid drawing an overlay on the first paint until we know if we're skipping.
    if (reduceMotion) {
      setMode("skip");
      setLogoStarted(true);
      setCtaVisible(true);
      return;
    }

    // Replay intro at most once per local calendar day.
    if (hasPlayedIntroToday()) {
      setMode("skip");
      setLogoStarted(true);
      setCtaVisible(true);
      return;
    }

    markPlayedIntroToday();
    setMode("play");
    setLogoStarted(false);
    setCtaVisible(false);
  }, [reduceMotion]);

  useEffect(() => {
    if (mode !== "play") {
      return;
    }

    const startTimer = window.setTimeout(() => {
      setLogoStarted(true);
    }, LOGO_DELAY_MS);

    const lettersMs = LETTER_DURATION_MS + Math.max(0, letters.length - 1) * LETTER_STAGGER_MS;
    const ctaTriggerTimer = window.setTimeout(() => {
      setCtaVisible(true);
    }, LOGO_DELAY_MS + lettersMs + LOGO_HOLD_MS);

    return () => {
      window.clearTimeout(startTimer);
      window.clearTimeout(ctaTriggerTimer);
    };
  }, [letters.length, mode]);

  useLayoutEffect(() => {
    if (mode === "boot") {
      return;
    }

    const wordmarkEl = wordmarkRef.current;
    if (!wordmarkEl) {
      return;
    }

    const update = () => {
      const rect = wordmarkEl.getBoundingClientRect();
      const next = Math.max(0, Math.round(rect.height / 2 + CTA_GAP_PX));
      setCtaOffsetPx(next);
    };

    update();

    // Keep the button offset correct on responsive resizes.
    window.addEventListener("resize", update);
    return () => {
      window.removeEventListener("resize", update);
    };
  }, [mode]);

  const onBegin = useCallback(() => {
    router.push("/login");
  }, [router]);

  if (mode === "boot") {
    return null;
  }

  const playIntro = mode === "play" && !reduceMotion;
  const skipFadeIn = mode === "skip" && !reduceMotion;

  return (
    <div className="pointer-events-none absolute inset-0 z-10 flex items-center justify-center">
      <motion.div
        initial={skipFadeIn ? { opacity: 0, filter: "blur(6px)" } : false}
        animate={{ opacity: 1, filter: "blur(0px)" }}
        transition={skipFadeIn ? { duration: SKIP_ENTER_DURATION_MS / 1000, ease: "easeOut" } : { duration: 0 }}
        className="relative h-full w-full"
        style={{ willChange: skipFadeIn ? "opacity, filter" : undefined }}
      >
        <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2">
          <motion.div
            ref={wordmarkRef}
            initial={playIntro ? "hidden" : false}
            animate={logoStarted ? "show" : "hidden"}
            variants={{
              hidden: {},
              show: {
                transition: {
                  staggerChildren: LETTER_STAGGER_MS / 1000,
                },
              },
            }}
            className={LANDING_WORDMARK_CLASS}
            style={{ willChange: "transform, opacity, filter" }}
          >
            {letters.map((letter, index) => (
              <motion.span
                key={`${letter}-${index}`}
                className={`inline-block ${LANDING_WORDMARK_LETTER_TEXT_CLASS}`}
                variants={{
                  hidden: {
                    opacity: 0,
                    y: 14,
                    filter: "blur(10px)",
                  },
                  show: {
                    opacity: 1,
                    y: 0,
                    filter: "blur(0px)",
                    transition: {
                      duration: LETTER_DURATION_MS / 1000,
                      ease: "easeOut",
                    },
                  },
                }}
                style={{ willChange: "transform, opacity, filter" }}
              >
                {letter}
              </motion.span>
            ))}
          </motion.div>
        </div>

        <AnimatePresence initial={false}>
          {ctaVisible ? (
            <div
              key="intro-begin-wrap"
              className="absolute left-1/2 top-1/2 -translate-x-1/2"
              style={{ marginTop: ctaOffsetPx }}
            >
              <motion.button
                key="intro-begin"
                type="button"
                onClick={onBegin}
                initial={playIntro ? { opacity: 0, y: 10, filter: "blur(8px)" } : false}
                animate={{ opacity: 1, y: 0, filter: "blur(0px)" }}
                transition={
                  playIntro ? { duration: BUTTON_ENTER_DURATION_MS / 1000, ease: "easeOut" } : { duration: 0 }
                }
                className={`pointer-events-auto inline-flex items-center justify-center px-7 py-4 text-base font-semibold tracking-tight transition-colors hover:border-white/30 hover:shadow-[0_16px_36px_-20px_rgba(15,23,42,0.65)] focus:outline-none focus:ring-2 focus:ring-white/40 sm:text-lg ${UI_LIQUID_GLASS_WHITE}`}
                style={{ willChange: "transform, opacity, filter" }}
              >
                <span className={UI_LIQUID_GLASS_TEXT_WHITE}>Let&apos;s begin!</span>
              </motion.button>
            </div>
          ) : null}
        </AnimatePresence>
      </motion.div>
    </div>
  );
}
