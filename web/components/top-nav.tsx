import { headers } from "next/headers";
import Link from "next/link";

import { NavBurgerMenu } from "@/components/nav-burger-menu";
import { TopNavBrand } from "@/components/top-nav-brand";
import { UI_BUTTON_OUTLINE, UI_BUTTON_PRIMARY } from "@/lib/ui";
import { getOptionalUser } from "@/lib/server-auth";

const MODULE_LABELS: Record<string, string> = {
  "/dashboard": "Home",
  "/modules/train": "Train",
  "/modules/restaurant": "Restaurant",
  "/modules/calendar": "Calendar",
  "/settings/account": "Account",
  "/settings/payment": "Payment",
  "/admin/maintenance": "Maintenance",
};

function getModuleLabel(pathname: string): string {
  // Exact match first
  if (MODULE_LABELS[pathname]) return MODULE_LABELS[pathname];
  // Prefix match for nested routes
  for (const [path, label] of Object.entries(MODULE_LABELS)) {
    if (pathname.startsWith(path)) return label;
  }
  return "Home";
}

export async function TopNav() {
  const user = await getOptionalUser();
  const headersList = headers();
  const pathname = headersList.get("x-pathname") || "/dashboard";
  const moduleLabel = getModuleLabel(pathname);

  return (
    <header className="sticky top-0 z-20 border-b border-blossom-100/80 bg-white/90 backdrop-blur">
      <div className="mx-auto flex w-full max-w-5xl items-center justify-between px-4 py-3.5 sm:px-6">
        <TopNavBrand href={user ? "/dashboard" : "/login"} />

        {user ? (
          <div className="flex items-center gap-3">
            <span className="inline-flex h-9 items-center rounded-full border border-blossom-200 bg-white px-3 text-sm font-medium text-slate-700 shadow-sm">
              {moduleLabel}
            </span>
            <NavBurgerMenu isAdmin={user.role === "admin"} />
          </div>
        ) : (
          <div className="flex items-center gap-2">
            <Link href="/login" className={UI_BUTTON_OUTLINE}>
              Login
            </Link>
            <Link href="/register" className={UI_BUTTON_PRIMARY}>
              Register
            </Link>
          </div>
        )}
      </div>
    </header>
  );
}
