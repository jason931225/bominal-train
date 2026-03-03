const SUPABASE_CONFIRM_TYPES = new Set(["recovery", "magiclink", "email", "signup"] as const);

type SupabaseConfirmType = "recovery" | "magiclink" | "email" | "signup";

export type SupabaseConfirmPayload = { token_hash: string; type: SupabaseConfirmType };

function normalizeType(raw: string | null | undefined): SupabaseConfirmType | null {
  const normalized = (raw ?? "").trim().toLowerCase();
  if (!normalized || !SUPABASE_CONFIRM_TYPES.has(normalized as SupabaseConfirmType)) {
    return null;
  }
  return normalized as SupabaseConfirmType;
}

export function resolveSupabaseConfirmPayload(searchParams: URLSearchParams): SupabaseConfirmPayload | null {
  const tokenHash = searchParams.get("token_hash")?.trim() ?? "";
  const resolvedType = normalizeType(searchParams.get("type"));
  if (!resolvedType || tokenHash.length < 8) {
    return null;
  }
  return { token_hash: tokenHash, type: resolvedType };
}
