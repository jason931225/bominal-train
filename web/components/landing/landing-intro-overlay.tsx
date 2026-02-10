"use client";

import { AnimatePresence, motion, useReducedMotion } from "framer-motion";
import { useRouter } from "next/navigation";
import { useCallback, useEffect, useMemo, useState } from "react";

import { UI_LIQUID_GLASS_TEXT_WHITE, UI_LIQUID_GLASS_WHITE } from "@/lib/ui";

const INTRO_SHOWN_KEY = "bominal:intro_shown:v1";
const WORDMARK = "bominal";

const LOGO_DELAY_MS = 800;
const LETTER_STAGGER_MS = 55;
const LETTER_DURATION_MS = 260;
const LOGO_HOLD_MS = 320;

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
  const [stage, setStage] = useState<"logo" | "button">("logo");
  const [logoStarted, setLogoStarted] = useState(false);

  useEffect(() => {
    if (reduceMotion) {
      safeSetSessionStorageItem(INTRO_SHOWN_KEY, "1");
      setLogoStarted(false);
      setStage("button");
      return;
    }

    const alreadyShown = safeGetSessionStorageItem(INTRO_SHOWN_KEY) != null;
    if (alreadyShown) {
      setLogoStarted(false);
      setStage("button");
      return;
    }

    safeSetSessionStorageItem(INTRO_SHOWN_KEY, "1");

    const startTimer = window.setTimeout(() => {
      setLogoStarted(true);
    }, LOGO_DELAY_MS);

    const lettersMs = LETTER_DURATION_MS + Math.max(0, letters.length - 1) * LETTER_STAGGER_MS;
    const exitTriggerTimer = window.setTimeout(() => {
      setStage("button");
    }, LOGO_DELAY_MS + lettersMs + LOGO_HOLD_MS);

    return () => {
      window.clearTimeout(startTimer);
      window.clearTimeout(exitTriggerTimer);
    };
  }, [letters.length, reduceMotion]);

  const onBegin = useCallback(() => {
    router.push("/login");
  }, [router]);

  return (
    <div className="absolute inset-0 flex items-center justify-center">
      <div className="absolute inset-0" aria-hidden="true" />

      <div className="pointer-events-none relative z-10 flex items-center justify-center">
        <AnimatePresence mode="wait" initial={false}>
          {stage === "logo" ? (
            <motion.div
              key="intro-logo"
              exit={{ opacity: 0, filter: "blur(8px)" }}
              transition={{ duration: 0.42, ease: "easeInOut" }}
              className="select-none"
              style={{ willChange: "opacity, filter" }}
              aria-hidden="true"
            >
              <motion.div
                initial="hidden"
                animate={logoStarted ? "show" : "hidden"}
                variants={{
                  hidden: {},
                  show: {
                    transition: {
                      staggerChildren: LETTER_STAGGER_MS / 1000,
                    },
                  },
                }}
                className="font-brand text-5xl font-semibold lowercase leading-none tracking-tight text-white sm:text-6xl"
              >
                {letters.map((letter, index) => (
                  <motion.span
                    key={`${letter}-${index}`}
                    className="inline-block"
                    variants={{
                      hidden: {
                        opacity: 0,
                        y: 10,
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
            </motion.div>
          ) : (
            <motion.button
              key="intro-begin"
              type="button"
              onClick={onBegin}
              initial={reduceMotion ? false : { opacity: 0, y: 10, filter: "blur(8px)" }}
              animate={{ opacity: 1, y: 0, filter: "blur(0px)" }}
              transition={reduceMotion ? { duration: 0 } : { duration: 0.36, ease: "easeOut" }}
              className={`pointer-events-auto inline-flex items-center justify-center px-7 py-4 text-base font-semibold tracking-tight transition hover:border-white/30 hover:from-white/25 hover:to-white/10 focus:outline-none focus:ring-2 focus:ring-white/40 sm:text-lg ${UI_LIQUID_GLASS_WHITE} ${UI_LIQUID_GLASS_TEXT_WHITE}`}
              style={{ willChange: "transform, opacity, filter" }}
            >
              Let&apos;s begin!
            </motion.button>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}

