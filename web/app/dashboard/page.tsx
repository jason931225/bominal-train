import { cookies } from "next/headers";
import { headers } from "next/headers";
import Link from "next/link";

import { ModuleTile } from "@/components/module-tile";
import { serverApiBaseUrl } from "@/lib/api-base";
import { NEXT_PUBLIC_RESTAURANT_MODULE_ENABLED } from "@/lib/feature-flags";
import { localeFromAcceptLanguage, localeFromUser, t } from "@/lib/i18n";
import { UI_BODY_MUTED, UI_CARD_MD, UI_CARD_LG, UI_KICKER, UI_TITLE_LG } from "@/lib/ui";
import { requireApprovedUser } from "@/lib/server-auth";
import type { BominalModule, ModulesResponse } from "@/lib/types";

const TRAIN_FALLBACK_MODULE: BominalModule = {
  slug: "train",
  name: "Train",
  coming_soon: false,
  enabled: true,
  capabilities: [
    "train.search",
    "train.tasks.create",
    "train.tasks.control",
    "train.credentials.manage",
    "train.tickets.manage",
    "wallet.payment_card",
  ],
};

const CALENDAR_FALLBACK_MODULE: BominalModule = {
  slug: "calendar",
  name: "Calendar",
  coming_soon: true,
  enabled: false,
  capabilities: [],
};

const RESTAURANT_FALLBACK_MODULE: BominalModule = {
  slug: "restaurant",
  name: "Restaurant",
  coming_soon: true,
  enabled: false,
  capabilities: [],
};

function fallbackModules(): BominalModule[] {
  if (!NEXT_PUBLIC_RESTAURANT_MODULE_ENABLED) {
    return [TRAIN_FALLBACK_MODULE, CALENDAR_FALLBACK_MODULE];
  }
  return [TRAIN_FALLBACK_MODULE, RESTAURANT_FALLBACK_MODULE, CALENDAR_FALLBACK_MODULE];
}

const DASHBOARD_MODULE_FETCH_TIMEOUT_MS = 3000;

async function fetchWithTimeout(input: string, init: RequestInit): Promise<Response> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), DASHBOARD_MODULE_FETCH_TIMEOUT_MS);
  try {
    return await fetch(input, { ...init, signal: controller.signal });
  } finally {
    clearTimeout(timeoutId);
  }
}

async function getModules() {
  const cookieStore = await cookies();
  const cookieHeader = cookieStore.toString();
  try {
    const response = await fetchWithTimeout(`${serverApiBaseUrl}/api/modules`, {
      headers: { cookie: cookieHeader },
      cache: "no-store",
    });
    if (!response.ok) {
      return fallbackModules();
    }
    const data = (await response.json()) as ModulesResponse;
    return data.modules;
  } catch {
    return fallbackModules();
  }
}

export default async function DashboardPage({
  searchParams,
}: {
  searchParams?: Promise<{ denied?: string }>;
}) {
  const user = await requireApprovedUser();
  const resolvedSearchParams = (await searchParams) ?? {};
  const headerStore = await headers();
  const locale = localeFromUser(user) ?? localeFromAcceptLanguage(headerStore.get("accept-language"));
  const modules = await getModules();

  return (
    <section className="space-y-8">
      {resolvedSearchParams.denied === "1" ? (
        <p className="rounded-xl bg-amber-50 px-3 py-2 text-sm text-amber-700">
          {t(locale, "dashboard.denied")}
        </p>
      ) : null}

      <div className={UI_CARD_LG}>
        <p className={UI_KICKER}>{t(locale, "dashboard.kicker")}</p>
        <h1 className={`mt-2 ${UI_TITLE_LG}`}>
          {t(locale, "dashboard.welcome", { name: user.display_name || user.email })}
        </h1>
        <p className={`mt-3 max-w-2xl ${UI_BODY_MUTED}`}>
          {t(locale, "dashboard.body")}
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {modules.map((module) => (
          <ModuleTile key={module.slug} module={module} locale={locale} />
        ))}
      </div>
    </section>
  );
}
