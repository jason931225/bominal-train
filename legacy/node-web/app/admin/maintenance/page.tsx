import { headers } from "next/headers";
import { Suspense } from "react";

import { serverApiBaseUrl } from "@/lib/api-base";
import { requireAdminUser } from "@/lib/server-auth";
import { UI_CARD_MD, UI_KICKER, UI_TITLE_LG, UI_BUTTON_OUTLINE_SM } from "@/lib/ui";
import { SystemStatsCard } from "@/components/admin/system-stats-card";
import { UserManagement } from "@/components/admin/user-management";
import { localeFromAcceptLanguage, localeFromUser, t } from "@/lib/i18n";
import { OpsStatusCard } from "@/components/admin/ops-status-card";
import { StaleTasksTable } from "@/components/admin/stale-tasks-table";
import { RecentFailuresTable } from "@/components/admin/recent-failures-table";
import { PaymentKillSwitchCard } from "@/components/admin/payment-settings-card";

export const dynamic = "force-dynamic";

export default async function MaintenancePage() {
  const user = await requireAdminUser();
  const headerStore = await headers();
  const locale = localeFromUser(user) ?? localeFromAcceptLanguage(headerStore.get("accept-language"));

  return (
    <div className="mx-auto max-w-6xl space-y-8">
      {/* Header */}
      <section className={UI_CARD_MD}>
        <div className="flex items-start justify-between">
          <div>
            <p className={UI_KICKER}>{t(locale, "admin.kicker")}</p>
            <h1 className={`mt-2 ${UI_TITLE_LG}`}>{t(locale, "admin.maintenanceTitle")}</h1>
            <p className="mt-2 text-sm text-slate-500">
              {t(locale, "admin.maintenanceBody")}
            </p>
          </div>
          <a
            href={`${serverApiBaseUrl}/api/docs`}
            target="_blank"
            rel="noopener noreferrer"
            className={UI_BUTTON_OUTLINE_SM}
          >
            {t(locale, "admin.apiDocs")} ↗
          </a>
        </div>
      </section>

      {/* System Stats */}
      <Suspense
        fallback={
          <div className={UI_CARD_MD}>
            <div className="h-32 animate-pulse rounded-xl bg-slate-100" />
          </div>
        }
      >
        <SystemStatsCard />
      </Suspense>

      {/* Ops */}
      <Suspense
        fallback={
          <div className={UI_CARD_MD}>
            <div className="h-32 animate-pulse rounded-xl bg-slate-100" />
          </div>
        }
      >
        <OpsStatusCard />
      </Suspense>

      <Suspense
        fallback={
          <div className={UI_CARD_MD}>
            <div className="h-64 animate-pulse rounded-xl bg-slate-100" />
          </div>
        }
      >
        <StaleTasksTable />
      </Suspense>

      <Suspense
        fallback={
          <div className={UI_CARD_MD}>
            <div className="h-64 animate-pulse rounded-xl bg-slate-100" />
          </div>
        }
      >
        <RecentFailuresTable />
      </Suspense>

      <Suspense
        fallback={
          <div className={UI_CARD_MD}>
            <div className="h-64 animate-pulse rounded-xl bg-slate-100" />
          </div>
        }
      >
        <PaymentKillSwitchCard />
      </Suspense>

      {/* User Management */}
      <Suspense
        fallback={
          <div className={UI_CARD_MD}>
            <div className="h-64 animate-pulse rounded-xl bg-slate-100" />
          </div>
        }
      >
        <UserManagement />
      </Suspense>
    </div>
  );
}
