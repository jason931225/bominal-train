import { cookies } from "next/headers";
import { Suspense } from "react";

import { serverApiBaseUrl } from "@/lib/api-base";
import { requireAdminUser } from "@/lib/server-auth";
import { UI_CARD_MD, UI_KICKER, UI_TITLE_LG } from "@/lib/ui";
import { SystemStatsCard } from "@/components/admin/system-stats-card";
import { UserManagement } from "@/components/admin/user-management";

export const dynamic = "force-dynamic";

export default async function MaintenancePage() {
  const user = await requireAdminUser();

  return (
    <div className="mx-auto max-w-6xl space-y-8">
      {/* Header */}
      <section className={UI_CARD_MD}>
        <p className={UI_KICKER}>Admin</p>
        <h1 className={`mt-2 ${UI_TITLE_LG}`}>Maintenance Dashboard</h1>
        <p className="mt-2 text-sm text-slate-500">
          Manage users, view system stats, and perform administrative tasks.
        </p>
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
