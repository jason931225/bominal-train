"use client";

import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { useRouter } from "next/navigation";
import { useCallback, useEffect, useMemo, useState } from "react";

import { UI_LIQUID_GLASS_TEXT_WHITE, UI_LIQUID_GLASS_WHITE } from "@/lib/ui";

const INTRO_SHOWN_KEY = "bominal:intro_shown:v1";
const WORDMARK = "bominal";

const LOGO_DELAY_MS = 800;
// Cinematic pacing (roughly 3.5s from the first letter to the CTA fully appearing).
const LETTER_STAGGER_MS = 130;
const LETTER_DURATION_MS = 620;
const LOGO_HOLD_MS = 700;
const BUTTON_ENTER_DURATION_MS = 600;

type IntroMode = "boot" | "play" | "skip";

function safeGetSessionStorageItem(key: string) {
  try {
    return sessionStorage.getItem(key);
  } catch {
    return null;
  }
}

function safeSetSessionStorageItem(key: string, value: string) {
  try {
    sessionStorage.setItem(key, value);
  } catch {
    // Ignore if storage is unavailable (privacy mode, blocked cookies, etc).
  }
}

export function LandingIntroOverlay() {
  const router = useRouter();
  const reduceMotion = useReducedMotion();

  const letters = useMemo(() => Array.from(WORDMARK), []);
  const [mode, setMode] = useState<IntroMode>("boot");
  const [logoStarted, setLogoStarted] = useState(false);
  const [ctaVisible, setCtaVisible] = useState(false);

  useEffect(() => {
    // Avoid drawing an overlay on the first paint until we know if we're skipping.
    if (reduceMotion) {
      safeSetSessionStorageItem(INTRO_SHOWN_KEY, "1");
      setMode("skip");
      setLogoStarted(true);
      setCtaVisible(true);
      return;
    }

    const alreadyShown = safeGetSessionStorageItem(INTRO_SHOWN_KEY) != null;
    if (alreadyShown) {
      setMode("skip");
      setLogoStarted(true);
      setCtaVisible(true);
      return;
    }

    setMode("play");
    setLogoStarted(false);
    setCtaVisible(false);
  }, [reduceMotion]);

  useEffect(() => {
    if (mode !== "play") {
      return;
    }

    const startTimer = window.setTimeout(() => {
      // Mark the intro as shown when it actually begins so React Strict Mode's
      // dev double-mount doesn't cause the second mount to skip the animation.
      safeSetSessionStorageItem(INTRO_SHOWN_KEY, "1");
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

  const onBegin = useCallback(() => {
    router.push("/login");
  }, [router]);

  if (mode === "boot") {
    return null;
  }

  const playIntro = mode === "play" && !reduceMotion;

  return (
    <div className="pointer-events-none absolute inset-0 z-10 flex items-center justify-center">
      <div className="flex flex-col items-center justify-center gap-8">
        <motion.div
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
          className={`select-none font-brand whitespace-nowrap text-[clamp(3.5rem,12vw,7rem)] font-semibold lowercase leading-none tracking-tight ${UI_LIQUID_GLASS_TEXT_WHITE}`}
          style={{ willChange: "transform, opacity, filter" }}
        >
          {letters.map((letter, index) => (
            <motion.span
              key={`${letter}-${index}`}
              className="inline-block"
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

        <AnimatePresence initial={false}>
          {ctaVisible ? (
            <motion.button
              key="intro-begin"
              type="button"
              onClick={onBegin}
              initial={playIntro ? { opacity: 0, y: 10, filter: "blur(8px)" } : false}
              animate={{ opacity: 1, y: 0, filter: "blur(0px)" }}
              transition={
                playIntro ? { duration: BUTTON_ENTER_DURATION_MS / 1000, ease: "easeOut" } : { duration: 0 }
              }
              className={`pointer-events-auto inline-flex items-center justify-center px-7 py-4 text-base font-semibold tracking-tight transition hover:border-white/30 hover:from-white/25 hover:to-white/10 focus:outline-none focus:ring-2 focus:ring-white/40 sm:text-lg ${UI_LIQUID_GLASS_WHITE} ${UI_LIQUID_GLASS_TEXT_WHITE}`}
              style={{ willChange: "transform, opacity, filter" }}
            >
              Let&apos;s begin!
            </motion.button>
          ) : null}
        </AnimatePresence>
      </div>
    </div>
  );
}
