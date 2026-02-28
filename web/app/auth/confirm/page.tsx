"use client";

import Link from "next/link";
import { useMemo, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { ROUTES } from "@/lib/routes";
import { cacheSupabaseAccessToken } from "@/lib/supabase-auth";
import { resolveSupabaseConfirmPayload } from "@/lib/supabase-callback";
import { UI_BODY_MUTED, UI_BUTTON_OUTLINE_TOUCH, UI_CARD_LG, UI_TITLE_LG } from "@/lib/ui";

type SupabaseConfirmResponse = {
  mode: "recovery" | "magiclink";
  redirect_to?: string | null;
  access_token?: string | null;
};

export default function AuthConfirmPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { t } = useLocale();
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const confirmPayload = useMemo(
    () => resolveSupabaseConfirmPayload(new URLSearchParams(searchParams.toString())),
    [searchParams],
  );
  const normalizedNext = useMemo(() => {
    const nextPath = searchParams.get("next")?.trim();
    return nextPath && nextPath.startsWith("/") ? nextPath : null;
  }, [searchParams]);

  async function onContinue() {
    if (!confirmPayload) {
      setErrorMessage(t("auth.callbackInvalidLink"));
      return;
    }

    setSubmitting(true);
    setErrorMessage(null);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/auth/supabase/confirm`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(confirmPayload),
      });
      if (!response.ok) {
        setErrorMessage(t("auth.callbackFailed"));
        return;
      }

      const payload = (await response.json()) as SupabaseConfirmResponse;
      if (payload.access_token) {
        cacheSupabaseAccessToken(payload.access_token);
      }
      if (payload.mode === "recovery") {
        router.replace(`${ROUTES.resetPassword}?mode=supabase`);
        return;
      }

      const destination = payload.redirect_to?.trim() || normalizedNext || ROUTES.modules.train;
      router.replace(destination);
    } catch {
      setErrorMessage(t("auth.callbackFailed"));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <section className={`mx-auto w-full max-w-md ${UI_CARD_LG}`}>
      <h1 className={UI_TITLE_LG}>{t("auth.signIn")}</h1>
      {errorMessage ? (
        <>
          <p className={`mt-2 ${UI_BODY_MUTED}`}>{errorMessage}</p>
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
