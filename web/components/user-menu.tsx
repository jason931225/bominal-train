import Link from "next/link";
import { LogoutButton } from "@/components/logout-button";

type UserMenuProps = {
  displayName: string;
};

export function UserMenu({ displayName }: UserMenuProps) {
  return (
    <details className="group relative [&_summary::-webkit-details-marker]:hidden">
      <summary className="inline-flex cursor-pointer list-none items-center gap-2 rounded-full border border-blossom-200 bg-white px-3 py-1.5 text-sm font-medium text-slate-700 transition hover:bg-blossom-50 focus:outline-none focus:ring-2 focus:ring-blossom-100">
        <span>{displayName}</span>
        <svg
          viewBox="0 0 20 20"
          className="h-4 w-4 transition group-open:rotate-180"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.8"
        >
          <path d="m5 7 5 6 5-6" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </summary>

      <div className="absolute right-0 z-30 mt-2 w-44 rounded-xl border border-blossom-100 bg-white p-1 shadow-lg">
        <Link
          href="/settings/account"
          className="block rounded-lg px-3 py-2 text-sm text-slate-700 transition hover:bg-blossom-50"
        >
          Account settings
        </Link>
        <Link
          href="/settings/payment"
          className="block rounded-lg px-3 py-2 text-sm text-slate-700 transition hover:bg-blossom-50"
        >
          Payment settings
        </Link>
        <div className="mt-1 border-t border-blossom-100 pt-1">
          <LogoutButton variant="menu" />
        </div>
      </div>
    </details>
  );
}
