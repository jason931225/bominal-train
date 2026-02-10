import Link from "next/link";

export function TopNavBrand({ href, sectionLabel }: { href: string; sectionLabel?: string | null }) {
  return (
    <Link href={href} className="group inline-flex items-center gap-3">
      <span className="font-brand text-2xl font-semibold lowercase tracking-tight text-slate-900 transition group-hover:text-blossom-700">
        bominal
      </span>
      {sectionLabel ? (
        <span className="hidden text-sm font-medium text-slate-600 sm:inline">/ {sectionLabel}</span>
      ) : null}
    </Link>
  );
}
