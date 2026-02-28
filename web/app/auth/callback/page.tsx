import Link from "next/link";

import { getServerT } from "@/lib/i18n-server";
import { ROUTES } from "@/lib/routes";
import { UI_BODY_MUTED, UI_BUTTON_OUTLINE_TOUCH, UI_CARD_LG, UI_TITLE_LG } from "@/lib/ui";

export default async function AuthCallbackLegacyPage() {
  const { t } = await getServerT();
  return (
    <section className={`mx-auto w-full max-w-md ${UI_CARD_LG}`}>
      <h1 className={UI_TITLE_LG}>{t("auth.signIn")}</h1>
      <p className={`mt-2 ${UI_BODY_MUTED}`}>{t("auth.callbackInvalidLink")}</p>
      <div className="mt-6 space-y-3">
        <Link href={ROUTES.authVerify} className={`w-full ${UI_BUTTON_OUTLINE_TOUCH}`}>
          {t("auth.signIn")}
        </Link>
        <Link href={ROUTES.forgotPassword} className={`w-full ${UI_BUTTON_OUTLINE_TOUCH}`}>
          {t("auth.requestPasswordReset")}
        </Link>
      </div>
    </section>
  );
}
