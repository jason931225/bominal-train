import Link from "next/link";
import { redirect } from "next/navigation";

import { PasswordResetRequestForm } from "@/components/auth/password-reset-request-form";
import { getServerT } from "@/lib/i18n-server";
import { ROUTES } from "@/lib/routes";
import { getOptionalUser } from "@/lib/server-auth";
import { UI_BODY_MUTED, UI_CARD_LG, UI_TITLE_LG } from "@/lib/ui";

export default async function ForgotPasswordPage() {
  const user = await getOptionalUser();
  if (user) {
    redirect(ROUTES.dashboard);
  }

  const { t } = await getServerT();

  return (
    <section className={`mx-auto w-full max-w-md ${UI_CARD_LG}`}>
      <h1 className={UI_TITLE_LG}>{t("auth.forgotPasswordTitle")}</h1>
      <p className={`mt-2 ${UI_BODY_MUTED}`}>{t("auth.forgotPasswordSubtitle")}</p>

      <div className="mt-6">
        <PasswordResetRequestForm />
      </div>

      <p className="mt-6 text-sm text-slate-600">
        <Link href={ROUTES.login} className="font-medium text-blossom-600 hover:text-blossom-700">
          {t("auth.signIn")}
        </Link>
      </p>
    </section>
  );
}
