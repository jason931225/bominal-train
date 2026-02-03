import Link from "next/link";

import type { BominalModule } from "@/lib/types";
import { UI_CHIP_BRAND, UI_CHIP_MUTED } from "@/lib/ui";

type ProviderBadge = {
  label: string;
  muted?: boolean;
};

function providerBadgesForModule(module: BominalModule): ProviderBadge[] {
  if (module.slug === "train") {
    return [{ label: "KTX" }, { label: "SRT" }];
  }
  if (module.slug === "restaurant") {
    return [{ label: "Catchtable", muted: true }, { label: "Resy", muted: true }, { label: "OpenTable", muted: true }];
  }
  if (module.slug === "calendar") {
    return [{ label: "None", muted: true }];
  }
  return [];
}

export function ModuleTile({ module }: { module: BominalModule }) {
  const providerBadges = providerBadgesForModule(module);

  return (
    <Link
      href={`/modules/${module.slug}`}
      className="group rounded-3xl border border-blossom-100 bg-white p-6 shadow-petal transition hover:-translate-y-0.5 hover:border-blossom-300"
    >
      <div className="flex items-start justify-between gap-4">
        <div>
          <h3 className="text-lg font-semibold text-slate-800 group-hover:text-blossom-700">{module.name}</h3>
          <p className="mt-1 text-sm text-slate-500">{module.coming_soon ? "Coming soon" : "Available"}</p>
        </div>
        {providerBadges.length > 0 ? (
          <div className="flex flex-wrap justify-end gap-2">
            {providerBadges.map((badge) => (
              <span
                key={`${module.slug}-${badge.label}`}
                className={badge.muted ? UI_CHIP_MUTED : UI_CHIP_BRAND}
              >
                {badge.label}
              </span>
            ))}
          </div>
        ) : (
          <span className={UI_CHIP_BRAND}>
            {module.slug}
          </span>
        )}
      </div>
    </Link>
  );
}
