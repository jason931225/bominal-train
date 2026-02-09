import Link from "next/link";
import { redirect } from "next/navigation";

import { getOptionalUser } from "@/lib/server-auth";
import { getServerT } from "@/lib/i18n-server";
import { ROUTES } from "@/lib/routes";
import { UI_BODY_MUTED, UI_BUTTON_OUTLINE, UI_BUTTON_PRIMARY, UI_CARD_LG, UI_KICKER, UI_TITLE_LG } from "@/lib/ui";

export default async function HomePage() {
  const user = await getOptionalUser();
  if (user) {
    redirect(ROUTES.dashboard);
  }
  const { t } = await getServerT();

  return (
    <div className="space-y-10">
      <section className="relative overflow-hidden rounded-3xl border border-blossom-100 bg-white/75 shadow-petal">
        <div className="pointer-events-none absolute inset-0">
          <div className="absolute -left-24 -top-24 h-72 w-72 rounded-full bg-blossom-200/55 blur-3xl" />
          <div className="absolute -bottom-24 -right-24 h-72 w-72 rounded-full bg-blossom-100/70 blur-3xl" />
          <div className="absolute inset-0 bg-[linear-gradient(to_right,rgba(15,23,42,0.03)_1px,transparent_1px),linear-gradient(to_bottom,rgba(15,23,42,0.03)_1px,transparent_1px)] bg-[size:28px_28px]" />
        </div>

        <div className="relative p-8 sm:p-12">
          <p className={UI_KICKER}>{t("landing.kicker")}</p>
          <h1 className={`mt-3 ${UI_TITLE_LG}`}>
            {t("app.tagline")}
          </h1>
          <p className={`mt-4 max-w-2xl ${UI_BODY_MUTED}`}>
            {t("app.taglineDetail")}
          </p>

          <div className="mt-7 flex flex-wrap gap-3">
            <Link href={ROUTES.register} className={UI_BUTTON_PRIMARY}>
              {t("landing.ctaCreate")}
            </Link>
            <Link href={ROUTES.login} className={UI_BUTTON_OUTLINE}>
              {t("landing.ctaSignIn")}
            </Link>
          </div>
        </div>
      </section>

      <section className="grid gap-4 md:grid-cols-3">
        <div className={UI_CARD_LG}>
          <p className={UI_KICKER}>{t("landing.cards.trainKicker")}</p>
          <h2 className="mt-2 font-display text-xl font-semibold tracking-tight text-slate-900">
            {t("landing.cards.trainTitle")}
          </h2>
          <p className={`mt-2 ${UI_BODY_MUTED}`}>
            {t("landing.cards.trainBody")}
          </p>
        </div>
        <div className={UI_CARD_LG}>
          <p className={UI_KICKER}>{t("landing.cards.securityKicker")}</p>
          <h2 className="mt-2 font-display text-xl font-semibold tracking-tight text-slate-900">
            {t("landing.cards.securityTitle")}
          </h2>
          <p className={`mt-2 ${UI_BODY_MUTED}`}>
            {t("landing.cards.securityBody")}
          </p>
        </div>
        <div className={UI_CARD_LG}>
          <p className={UI_KICKER}>{t("landing.cards.opsKicker")}</p>
          <h2 className="mt-2 font-display text-xl font-semibold tracking-tight text-slate-900">
            {t("landing.cards.opsTitle")}
          </h2>
          <p className={`mt-2 ${UI_BODY_MUTED}`}>
            {t("landing.cards.opsBody")}
          </p>
        </div>
      </section>
    </div>
  );
}
