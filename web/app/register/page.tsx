import Link from "next/link";
import { redirect } from "next/navigation";

import { RegisterForm } from "@/components/register-form";
import { getServerT } from "@/lib/i18n-server";
import { ROUTES } from "@/lib/routes";
import { UI_BODY_MUTED, UI_CARD_LG, UI_TITLE_LG } from "@/lib/ui";
import { getOptionalUser, postLoginRouteForUser } from "@/lib/server-auth";

export default async function RegisterPage() {
  const user = await getOptionalUser();
  if (user) {
    redirect(postLoginRouteForUser(user));
  }
  const { t } = await getServerT();

  return (
    <section className={`mx-auto w-full max-w-md ${UI_CARD_LG}`}>
      <h1 className={UI_TITLE_LG}>{t("auth.registerTitle")}</h1>
      <p className={`mt-2 ${UI_BODY_MUTED}`}>{t("auth.registerSubtitle")}</p>

      <div className="mt-6">
        <RegisterForm />
      </div>

      <p className="mt-6 text-sm text-slate-600">
        {t("auth.alreadyHaveAccount")}{" "}
        <Link href={ROUTES.login} className="font-medium text-blossom-600 hover:text-blossom-700">
          {t("auth.signIn")}
        </Link>
      </p>
    </section>
  );
}
