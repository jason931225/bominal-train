import Link from "next/link";

export function TopNavBrand({ href, sectionLabel }: { href: string; sectionLabel?: string | null }) {
  return (
    <Link href={href} className="group inline-flex items-center gap-3">
      {/*
        Wordmark spec:
        - Size: 20% larger than Tailwind `text-2xl` (1.5rem -> 1.8rem).
        - Color: theme-aware via `blossom-*` CSS vars (see `web/app/globals.css` theme palettes).
          - Default: `text-blossom-800`
          - Hover: `text-blossom-700` (slightly brighter within the same theme)
      */}
      <span className="font-brand text-[1.8rem] font-semibold lowercase leading-none tracking-tight text-blossom-800 transition group-hover:text-blossom-700">
        bominal
      </span>
      {sectionLabel ? (
        <span className="hidden text-sm font-medium text-slate-600 sm:inline">/ {sectionLabel}</span>
      ) : null}
    </Link>
  );
}
