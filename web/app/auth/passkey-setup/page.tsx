import { redirect } from "next/navigation";

import { PasskeySetupOffer } from "@/components/auth/passkey-setup-offer";
import { getServerT } from "@/lib/i18n-server";
import { ROUTES } from "@/lib/routes";
import { requireUser } from "@/lib/server-auth";
import { UI_BODY_MUTED, UI_CARD_LG, UI_TITLE_LG } from "@/lib/ui";

type SearchParams = {
  source?: string;
  next?: string;
};

function normalizeNextPath(raw: string | undefined): string {
  const value = (raw ?? "").trim();
  if (!value || !value.startsWith("/")) return ROUTES.modules.train;
  return value;
}

function normalizeSource(raw: string | undefined): "signup" | "reset" | "magiclink" | "unknown" {
  const value = (raw ?? "").trim().toLowerCase();
  if (value === "signup" || value === "reset" || value === "magiclink") return value;
  return "unknown";
}

export default async function PasskeySetupPage({
  searchParams,
}: {
  searchParams?: Promise<SearchParams>;
}) {
  const user = await requireUser();
  if (!user) {
    redirect(ROUTES.login);
  }
  const { t } = await getServerT();
  const resolvedSearchParams = (await searchParams) ?? {};
  const nextPath = normalizeNextPath(resolvedSearchParams.next);
  const source = normalizeSource(resolvedSearchParams.source);

  return (
    <section className={`mx-auto w-full max-w-md ${UI_CARD_LG}`}>
      <h1 className={UI_TITLE_LG}>{t("auth.passkeyOfferHeading")}</h1>
      <p className={`mt-2 ${UI_BODY_MUTED}`}>{t("auth.passkeyOfferSubheading")}</p>
      <PasskeySetupOffer source={source} nextPath={nextPath} />
    </section>
  );
}
