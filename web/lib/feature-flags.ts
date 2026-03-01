const TRUTHY_ENV_VALUES = new Set(["1", "true", "yes", "on"]);

function envFlagEnabled(rawValue: string | undefined, defaultValue: boolean): boolean {
  if (rawValue == null) {
    return defaultValue;
  }

  const normalized = rawValue.trim().toLowerCase();
  if (!normalized) {
    return defaultValue;
  }

  return TRUTHY_ENV_VALUES.has(normalized);
}

function envIntegerValue(rawValue: string | undefined, defaultValue: number, min: number, max: number): number {
  if (rawValue == null) {
    return defaultValue;
  }
  const normalized = rawValue.trim();
  if (!normalized) {
    return defaultValue;
  }
  const parsed = Number.parseInt(normalized, 10);
  if (!Number.isFinite(parsed)) {
    return defaultValue;
  }
  return Math.min(max, Math.max(min, parsed));
}

export const NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED = envFlagEnabled(
  process.env.NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED,
  false,
);

export const NEXT_PUBLIC_RESTAURANT_MODULE_ENABLED = envFlagEnabled(
  process.env.NEXT_PUBLIC_RESTAURANT_MODULE_ENABLED,
  true,
);

export const NEXT_PUBLIC_SUPABASE_DIRECT_AUTH_ENABLED = envFlagEnabled(
  process.env.NEXT_PUBLIC_SUPABASE_DIRECT_AUTH_ENABLED,
  false,
);

export const NEXT_PUBLIC_SUPABASE_REALTIME_ENABLED = envFlagEnabled(
  process.env.NEXT_PUBLIC_SUPABASE_REALTIME_ENABLED,
  false,
);

export const NEXT_PUBLIC_SUPABASE_REALTIME_DELTA_READ_ENABLED = envFlagEnabled(
  process.env.NEXT_PUBLIC_SUPABASE_REALTIME_DELTA_READ_ENABLED,
  true,
);

export const NEXT_PUBLIC_TRAIN_READS_VIA_DATA_API = envFlagEnabled(
  process.env.NEXT_PUBLIC_TRAIN_READS_VIA_DATA_API,
  false,
);

export const NEXT_PUBLIC_TRAIN_DETAIL_VIA_GRAPHQL = envFlagEnabled(
  process.env.NEXT_PUBLIC_TRAIN_DETAIL_VIA_GRAPHQL,
  false,
);

export const NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_ENABLED = envFlagEnabled(
  process.env.NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_ENABLED,
  false,
);

export const NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_CANARY_PERCENT = envIntegerValue(
  process.env.NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_CANARY_PERCENT,
  0,
  0,
  100,
);

export const NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_RETRY_SECONDS = envIntegerValue(
  process.env.NEXT_PUBLIC_TRAIN_EVENTS_REALTIME_RETRY_SECONDS,
  60,
  5,
  600,
);
