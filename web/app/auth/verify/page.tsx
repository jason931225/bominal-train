import Link from "next/link";
import { redirect } from "next/navigation";

import { getServerT } from "@/lib/i18n-server";
import { ROUTES } from "@/lib/routes";
import { UI_BODY_MUTED, UI_CARD_LG, UI_TITLE_LG } from "@/lib/ui";

const SUPABASE_TYPES = new Set(["recovery", "magiclink", "email", "signup"]);

export default async function AuthVerifyPage({
  searchParams,
}: {
  searchParams?: Promise<{ token_hash?: string; type?: string; next?: string }>;
}) {
  const resolvedSearchParams = (await searchParams) ?? {};
  const tokenHash = resolvedSearchParams.token_hash?.trim() ?? "";
  const callbackType = resolvedSearchParams.type?.trim().toLowerCase() ?? "";
  const nextPath = resolvedSearchParams.next?.trim();

  if (tokenHash.length >= 8 && SUPABASE_TYPES.has(callbackType)) {
    const params = new URLSearchParams({
      token_hash: tokenHash,
      type: callbackType,
    });
    if (nextPath && nextPath.startsWith("/")) {
      params.set("next", nextPath);
    }
    redirect(`${ROUTES.authConfirm}?${params.toString()}`);
  }

  const { t } = await getServerT();
  return (
    <section className={`mx-auto w-full max-w-md ${UI_CARD_LG}`}>
      <h1 className={UI_TITLE_LG}>{t("auth.signIn")}</h1>
      <p className={`mt-2 ${UI_BODY_MUTED}`}>{t("auth.callbackInvalidLink")}</p>
      <div className="mt-6 space-y-3 text-sm">
        <Link href={ROUTES.forgotPassword} className="block font-medium text-blossom-600 hover:text-blossom-700">
          {t("auth.requestPasswordReset")}
        </Link>
        <Link href={ROUTES.login} className="block font-medium text-blossom-600 hover:text-blossom-700">
          {t("auth.signIn")}
        </Link>
      </div>
    </section>
  );
}
