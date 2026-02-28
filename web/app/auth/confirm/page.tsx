"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { ROUTES } from "@/lib/routes";
import { cacheSupabaseAccessToken } from "@/lib/supabase-auth";
import { resolveSupabaseConfirmPayload } from "@/lib/supabase-callback";
import { UI_BODY_MUTED, UI_CARD_LG, UI_TITLE_LG } from "@/lib/ui";

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

  useEffect(() => {
    let cancelled = false;

    async function runConfirm() {
      const confirmPayload = resolveSupabaseConfirmPayload(
        new URLSearchParams(searchParams.toString()),
      );
      const nextPath = searchParams.get("next")?.trim();
      const normalizedNext = nextPath && nextPath.startsWith("/") ? nextPath : null;
      if (!confirmPayload) {
        setErrorMessage(t("auth.callbackInvalidLink"));
        return;
      }

      try {
        const response = await fetch(`${clientApiBaseUrl}/api/auth/supabase/confirm`, {
          method: "POST",
          credentials: "include",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(confirmPayload),
        });
        if (!response.ok) {
          if (!cancelled) {
            setErrorMessage(t("auth.callbackFailed"));
          }
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
        if (!cancelled) {
          setErrorMessage(t("auth.callbackFailed"));
        }
      }
    }

    void runConfirm();
    return () => {
      cancelled = true;
    };
  }, [router, searchParams, t]);

  return (
    <section className={`mx-auto w-full max-w-md ${UI_CARD_LG}`}>
      <h1 className={UI_TITLE_LG}>{t("auth.signIn")}</h1>
      {errorMessage ? (
        <>
          <p className={`mt-2 ${UI_BODY_MUTED}`}>{errorMessage}</p>
          <div className="mt-6 space-y-3 text-sm">
            <Link href={ROUTES.forgotPassword} className="block font-medium text-blossom-600 hover:text-blossom-700">
              {t("auth.requestPasswordReset")}
            </Link>
            <Link href={ROUTES.login} className="block font-medium text-blossom-600 hover:text-blossom-700">
              {t("auth.signIn")}
            </Link>
          </div>
        </>
      ) : (
        <p className={`mt-2 ${UI_BODY_MUTED}`}>{t("auth.callbackProcessing")}</p>
      )}
    </section>
  );
}
