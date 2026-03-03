"use client";

import { useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import { useLocale } from "@/components/locale-provider";
import { LogoutButton } from "@/components/logout-button";
import { useTheme } from "@/components/theme-provider";
import { clientApiBaseUrl } from "@/lib/api-base";
import { ROUTES } from "@/lib/routes";
import { UI_BUTTON_DANGER, UI_CARD_LG } from "@/lib/ui";

const LANDING_VIDEO_BASE_URL =
  process.env.NEXT_PUBLIC_LANDING_VIDEO_BASE_URL ??
  "https://github.com/jason931225/bominal.github.io/raw/refs/heads/main/public/video";

export function ApplicationReviewGate({ email }: { email: string }) {
  const { t } = useLocale();
  const { theme } = useTheme();
  const router = useRouter();
  const [deleting, setDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const videoSrc = useMemo(() => `${LANDING_VIDEO_BASE_URL}/${theme}.mp4`, [theme]);

  const onDeleteAccount = async () => {
    setError(null);
    if (!window.confirm(t("review.deleteConfirm"))) return;

    setDeleting(true);
    try {
      const response = await fetch(`${clientApiBaseUrl}/api/auth/account`, {
        method: "DELETE",
        credentials: "include",
      });
      if (!response.ok) {
        setError(t("review.deleteFailed"));
        return;
      }
      window.location.assign(ROUTES.login);
    } catch {
      setError(t("review.deleteFailed"));
    } finally {
      setDeleting(false);
      router.refresh();
    }
  };

  return (
    <section className="relative min-h-[100dvh] w-full overflow-hidden">
      <div className="absolute inset-0">
        <video
          src={videoSrc}
          muted
          loop
          playsInline
          autoPlay
          preload="auto"
          className="h-full w-full object-cover object-center"
        />
        <div className="absolute inset-0 bg-slate-900/55" />
      </div>

      <div className="relative z-10 mx-auto flex min-h-[100dvh] w-full max-w-3xl items-center justify-center px-4 py-12 sm:px-6">
        <div className={`${UI_CARD_LG} w-full max-w-2xl border border-blossom-200/70 bg-white/95 backdrop-blur`}>
          <p className="text-xs font-semibold uppercase tracking-[0.14em] text-blossom-500">{t("review.kicker")}</p>
          <h1 className="mt-2 font-display text-3xl font-semibold text-slate-900">{t("review.title")}</h1>
          <p className="mt-3 text-sm leading-relaxed text-slate-700">{t("review.body")}</p>
          <p className="mt-3 rounded-xl bg-blossom-50 px-3 py-2 text-xs text-blossom-700">{t("review.signedInAs", { email })}</p>

          {error ? <p className="mt-3 rounded-xl bg-rose-50 px-3 py-2 text-sm text-rose-700">{error}</p> : null}

          <div className="mt-5 flex flex-wrap gap-2">
            <LogoutButton variant="pill" />
            <button
              type="button"
              onClick={onDeleteAccount}
              disabled={deleting}
              className={UI_BUTTON_DANGER}
            >
              {deleting ? t("review.deleting") : t("review.deleteAccount")}
            </button>
          </div>
        </div>
      </div>
    </section>
  );
}
