"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { ROUTES } from "@/lib/routes";
import { UI_BODY_MUTED, UI_BUTTON_OUTLINE_TOUCH, UI_CARD_LG, UI_TITLE_LG } from "@/lib/ui";

export default function AuthMagicLinkPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { t } = useLocale();
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const email = useMemo(() => (searchParams.get("email") ?? "").trim(), [searchParams]);
  const code = useMemo(() => (searchParams.get("code") ?? "").trim(), [searchParams]);

  async function onContinue() {
    if (!email || !code) {
      setError(t("auth.callbackInvalidLink"));
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/auth/magic-link/confirm`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, code }),
      });
      if (!response.ok) {
        const body = (await response.json().catch(() => null)) as { detail?: string } | null;
        setError(body?.detail ?? t("auth.callbackFailed"));
        return;
      }
      router.replace(`${ROUTES.authPasskeySetup}?source=magiclink&next=${encodeURIComponent(ROUTES.modules.train)}`);
    } catch {
      setError(t("auth.callbackFailed"));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <section className={`mx-auto w-full max-w-md ${UI_CARD_LG}`}>
      <h1 className={UI_TITLE_LG}>{t("auth.signIn")}</h1>
      {error ? (
        <>
          <p className={`mt-2 ${UI_BODY_MUTED}`}>{error}</p>
          <div className="mt-6 space-y-3">
            <Link href={ROUTES.forgotPassword} className={`w-full ${UI_BUTTON_OUTLINE_TOUCH}`}>
              {t("auth.requestPasswordReset")}
            </Link>
            <Link href={ROUTES.login} className={`w-full ${UI_BUTTON_OUTLINE_TOUCH}`}>
              {t("auth.signIn")}
            </Link>
          </div>
        </>
      ) : (
        <>
          <p className={`mt-2 ${UI_BODY_MUTED}`}>{t("auth.callbackProcessing")}</p>
          <button
            type="button"
            className="mt-6 inline-flex w-full items-center justify-center rounded-xl bg-blossom-600 px-4 py-3 text-sm font-semibold text-white transition hover:bg-blossom-700 disabled:cursor-not-allowed disabled:opacity-60"
            onClick={() => {
              void onContinue();
            }}
            disabled={submitting}
          >
            {submitting ? t("auth.callbackProcessing") : t("auth.continue")}
          </button>
        </>
      )}
    </section>
  );
}
