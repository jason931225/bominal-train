import Link from "next/link";

import type { BominalModule } from "@/lib/types";
import type { Locale } from "@/lib/i18n";
import { t } from "@/lib/i18n";
import { UI_CHIP_BRAND, UI_CHIP_MUTED } from "@/lib/ui";

type ProviderBadge = {
  label: string;
  muted?: boolean;
};

const CAPABILITY_LABELS: Record<string, string> = {
  "train.search": "Search",
  "train.tasks.create": "Create Tasks",
  "train.tasks.control": "Control Tasks",
  "train.credentials.manage": "Credentials",
  "train.tickets.manage": "Tickets",
  "wallet.payment_card": "Payment Card",
};

function capabilityLabel(capability: string): string {
  const known = CAPABILITY_LABELS[capability];
  if (known) {
    return known;
  }

  const tail = capability.split(".").pop() ?? capability;
  return tail
    .replace(/[_-]+/g, " ")
    .replace(/\b\w/g, (char) => char.toUpperCase());
}

function providerBadgesForModule(module: BominalModule): ProviderBadge[] {
  if (module.capabilities.length > 0) {
    return module.capabilities.map((capability) => ({
      label: capabilityLabel(capability),
      muted: module.coming_soon || !module.enabled,
    }));
  }
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

function moduleLabel(locale: Locale, module: BominalModule): string {
  if (module.slug === "train") return t(locale, "nav.train");
  if (module.slug === "restaurant") return t(locale, "nav.restaurant");
  if (module.slug === "calendar") return t(locale, "nav.calendar");
  return module.name;
}

export function ModuleTile({ module, locale }: { module: BominalModule; locale: Locale }) {
  const providerBadges = providerBadgesForModule(module);

  return (
    <Link
      href={`/modules/${module.slug}`}
      className="group rounded-3xl border border-blossom-100 bg-white p-6 shadow-petal transition hover:-translate-y-0.5 hover:border-blossom-300"
    >
      <div className="flex items-start justify-between gap-4">
        <div>
          <h3 className="text-lg font-semibold text-slate-800 group-hover:text-blossom-700">
            {moduleLabel(locale, module)}
          </h3>
          <p className="mt-1 text-sm text-slate-500">
            {module.coming_soon ? t(locale, "modules.comingSoon") : t(locale, "modules.available")}
          </p>
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
