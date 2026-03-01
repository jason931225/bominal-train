import Link from "next/link";
import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { PasswordResetConfirmForm } from "@/components/auth/password-reset-confirm-form";
import { getServerT } from "@/lib/i18n-server";
import { ROUTES } from "@/lib/routes";
import { getOptionalUser, postLoginRouteForUser } from "@/lib/server-auth";
import { UI_BODY_MUTED, UI_BUTTON_OUTLINE_TOUCH, UI_CARD_LG, UI_TITLE_LG } from "@/lib/ui";

export default async function ResetPasswordPage({
  searchParams,
}: {
  searchParams?: Promise<{ email?: string; code?: string }>;
}) {
  const cookieStore = await cookies();
  const hasSupabaseRecoveryContext = Boolean(cookieStore.get("bominal_supabase_recovery_ctx")?.value);
  const hasPasskeySetupContext = Boolean(cookieStore.get("bominal_passkey_setup_ctx")?.value);
  const user = await getOptionalUser();
  if (user) {
    if (hasPasskeySetupContext) {
      redirect(`${ROUTES.authPasskeyAdd}?source=reset&next=${encodeURIComponent(ROUTES.modules.train)}`);
    }
    if (!hasSupabaseRecoveryContext) {
      redirect(postLoginRouteForUser(user));
    }
  }

  const { t } = await getServerT();
  const resolvedSearchParams = (await searchParams) ?? {};
  const initialEmail = resolvedSearchParams.email ?? "";
  const initialCode = resolvedSearchParams.code ?? "";
  const mode = hasSupabaseRecoveryContext ? "supabase" : "otp";
  const subtitle = mode === "supabase" ? t("auth.resetPasswordSupabaseSubtitle") : t("auth.resetPasswordSubtitle");

  return (
    <section className={`mx-auto w-full max-w-md ${UI_CARD_LG}`}>
      <h1 className={UI_TITLE_LG}>{t("auth.resetPasswordTitle")}</h1>
      <p className={`mt-2 ${UI_BODY_MUTED}`}>{subtitle}</p>

      <div className="mt-6">
        <PasswordResetConfirmForm initialEmail={initialEmail} initialCode={initialCode} mode={mode} />
      </div>

      <div className="mt-6">
        <Link href={ROUTES.login} className={`w-full ${UI_BUTTON_OUTLINE_TOUCH}`}>
          {t("auth.signIn")}
        </Link>
      </div>
    </section>
  );
}
