"use client";

import { useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import { useLocale } from "@/components/locale-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { listPasskeysFromSession, registerPasskeyFromSession } from "@/lib/passkey";
import { UI_BUTTON_PRIMARY } from "@/lib/ui";

type PasskeySetupOfferProps = {
  source: "signup" | "reset" | "magiclink" | "unknown";
  nextPath: string;
};

export function PasskeySetupOffer({ source, nextPath }: PasskeySetupOfferProps) {
  const router = useRouter();
  const { t } = useLocale();
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let mounted = true;
    void (async () => {
      try {
        const result = await listPasskeysFromSession(clientApiBaseUrl);
        if (!mounted) return;
        if (result.credentials.length > 0) {
          router.replace(nextPath);
          return;
        }
      } catch {
        // Keep the interstitial available even if passkey list fetch fails.
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    })();
    return () => {
      mounted = false;
    };
  }, [nextPath, router]);

  const sourceMessage = useMemo(() => {
    if (source === "signup") return t("auth.passkeyOfferSourceSignup");
    if (source === "reset") return t("auth.passkeyOfferSourceReset");
    if (source === "magiclink") return t("auth.passkeyOfferSourceMagiclink");
    return t("auth.passkeyOfferSourceGeneric");
  }, [source, t]);

  const onAddPasskey = async () => {
    setSubmitting(true);
    setError(null);
    try {
      const result = await registerPasskeyFromSession(clientApiBaseUrl);
      if (!result.ok) {
        setError(result.error ?? t("auth.passkeySetupFailed"));
        return;
      }
      router.replace(nextPath);
    } catch {
      setError(t("auth.passkeySetupFailed"));
    } finally {
      setSubmitting(false);
    }
  };

  const onSkip = () => {
    router.replace(nextPath);
  };

  if (loading) {
    return (
      <div className="mt-6 rounded-xl border border-blossom-100 bg-white p-6 text-center text-sm text-slate-600">
        {t("common.loading")}
      </div>
    );
  }

  return (
    <div className="mt-6 rounded-2xl border border-blossom-100 bg-white p-6 text-center shadow-sm">
      <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-blossom-50 text-blossom-700">
        <svg viewBox="0 0 24 24" className="h-7 w-7" fill="none" stroke="currentColor" strokeWidth="1.8" aria-hidden="true">
          <rect x="5" y="10" width="14" height="10" rx="2" />
          <path d="M8 10V8a4 4 0 1 1 8 0v2" />
        </svg>
      </div>
      <p className="mt-4 text-sm text-slate-600">{sourceMessage}</p>
      <h2 className="mt-2 text-lg font-semibold text-slate-900">{t("auth.passkeyOfferTitle")}</h2>
      <p className="mt-2 text-sm text-slate-600">{t("auth.passkeyOfferBody")}</p>

      {error ? <p className="mt-4 rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}

      <div className="mt-5 space-y-3">
        <button type="button" onClick={() => void onAddPasskey()} disabled={submitting} className={`w-full ${UI_BUTTON_PRIMARY}`}>
          {submitting ? t("auth.settingUpPasskey") : t("auth.addPasskeyNow")}
        </button>
        <button
          type="button"
          onClick={onSkip}
          disabled={submitting}
          className="w-full rounded-xl border border-slate-300 px-4 py-3 text-sm font-medium text-slate-700 transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-60"
        >
          {t("auth.skipPasskeyForNow")}
        </button>
      </div>
    </div>
  );
}
