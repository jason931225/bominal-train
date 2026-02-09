import Link from "next/link";
import { redirect } from "next/navigation";

import { LoginForm } from "@/components/login-form";
import { getServerT } from "@/lib/i18n-server";
import { ROUTES } from "@/lib/routes";
import { UI_BODY_MUTED, UI_CARD_LG, UI_TITLE_LG } from "@/lib/ui";
import { getOptionalUser } from "@/lib/server-auth";

export default async function LoginPage({
  searchParams,
}: {
  searchParams?: { registered?: string };
}) {
  const user = await getOptionalUser();
  if (user) {
    redirect(ROUTES.dashboard);
  }
  const { t } = await getServerT();

  return (
    <section className={`mx-auto w-full max-w-md ${UI_CARD_LG}`}>
      <h1 className={UI_TITLE_LG}>bominal</h1>
      <p className={`mt-2 ${UI_BODY_MUTED}`}>{t("auth.loginSubtitle")}</p>

      {searchParams?.registered === "1" ? (
        <p className="mt-4 rounded-xl bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
          {t("auth.accountCreatedPleaseSignIn")}
        </p>
      ) : null}

      <div className="mt-6">
        <LoginForm />
      </div>

      <p className="mt-6 text-sm text-slate-600">
        {t("auth.newHere")}{" "}
        <Link href={ROUTES.register} className="font-medium text-blossom-600 hover:text-blossom-700">
          {t("auth.createAnAccount")}
        </Link>
      </p>
    </section>
  );
}
