const SUPABASE_CALLBACK_TYPES = new Set(["recovery", "magiclink", "email", "signup"] as const);

type SupabaseCallbackType = "recovery" | "magiclink" | "email" | "signup";

export type SupabaseCallbackExchangePayload =
  | { token_hash: string; type: SupabaseCallbackType }
  | { access_token: string; type: SupabaseCallbackType };

function normalizeType(raw: string | null | undefined): SupabaseCallbackType | null {
  const normalized = (raw ?? "").trim().toLowerCase();
  if (!normalized || !SUPABASE_CALLBACK_TYPES.has(normalized as SupabaseCallbackType)) {
    return null;
  }
  return normalized as SupabaseCallbackType;
}

export function resolveSupabaseCallbackExchangePayload(
  searchParams: URLSearchParams,
  locationHash: string,
): SupabaseCallbackExchangePayload | null {
  const queryTokenHash = searchParams.get("token_hash")?.trim() ?? "";
  const queryType = normalizeType(searchParams.get("type"));

  if (queryTokenHash && queryType) {
    return { token_hash: queryTokenHash, type: queryType };
  }

  const hashParams = new URLSearchParams(locationHash.startsWith("#") ? locationHash.slice(1) : locationHash);
  const hashAccessToken = hashParams.get("access_token")?.trim() ?? "";
  const hashType = normalizeType(hashParams.get("type"));
  const resolvedType = queryType ?? hashType;
  if (!resolvedType) {
    return null;
  }
  if (queryTokenHash) {
    return { token_hash: queryTokenHash, type: resolvedType };
  }
  if (hashAccessToken) {
    return { access_token: hashAccessToken, type: resolvedType };
  }
  return null;
}
