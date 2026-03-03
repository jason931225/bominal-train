import Link from "next/link";
import { redirect } from "next/navigation";

import { PasswordResetRequestForm } from "@/components/auth/password-reset-request-form";
import { getServerT } from "@/lib/i18n-server";
import { ROUTES } from "@/lib/routes";
import { getOptionalUser, postLoginRouteForUser } from "@/lib/server-auth";
import { UI_BODY_MUTED, UI_BUTTON_OUTLINE_TOUCH, UI_CARD_LG, UI_TITLE_LG } from "@/lib/ui";

export default async function ForgotPasswordPage({
  searchParams,
}: {
  searchParams?: Promise<{ email?: string }>;
}) {
  const user = await getOptionalUser();
  if (user) {
    redirect(postLoginRouteForUser(user));
  }

  const { t } = await getServerT();
  const resolvedSearchParams = (await searchParams) ?? {};
  const initialEmail = resolvedSearchParams.email ?? "";

  return (
    <section className={`mx-auto w-full max-w-md ${UI_CARD_LG}`}>
      <h1 className={UI_TITLE_LG}>{t("auth.forgotPasswordTitle")}</h1>
      <p className={`mt-2 ${UI_BODY_MUTED}`}>{t("auth.forgotPasswordSubtitle")}</p>

      <div className="mt-6">
        <PasswordResetRequestForm initialEmail={initialEmail} />
      </div>

      <div className="mt-6">
        <Link href={ROUTES.login} className={`w-full ${UI_BUTTON_OUTLINE_TOUCH}`}>
          {t("auth.signIn")}
        </Link>
      </div>
    </section>
  );
}
