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

export const NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED = envFlagEnabled(
  process.env.NEXT_PUBLIC_TRAIN_AUTO_PAY_ENABLED,
  false,
);

export const NEXT_PUBLIC_RESTAURANT_MODULE_ENABLED = envFlagEnabled(
  process.env.NEXT_PUBLIC_RESTAURANT_MODULE_ENABLED,
  true,
);
