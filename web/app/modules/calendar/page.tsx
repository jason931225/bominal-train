import Link from "next/link";

import { UI_CARD_LG, UI_KICKER, UI_TITLE_LG } from "@/lib/ui";
import { getServerT } from "@/lib/i18n-server";
import { ROUTES } from "@/lib/routes";
import { requireUser } from "@/lib/server-auth";

export default async function CalendarModulePage() {
  await requireUser();
  const { t } = await getServerT();

  return (
    <section className={`mx-auto max-w-3xl ${UI_CARD_LG}`}>
      <p className={UI_KICKER}>{t("modules.moduleKicker")}</p>
      <h1 className={`mt-2 ${UI_TITLE_LG}`}>{t("modules.calendarTitle")}</h1>
      <p className="mt-3 text-slate-600">
        {t("modules.calendarComingSoon")}
      </p>
      <Link href={ROUTES.dashboard} className="mt-6 inline-block text-sm font-medium text-blossom-600 hover:text-blossom-700">
        {t("modules.backToDashboard")}
      </Link>
    </section>
  );
}
