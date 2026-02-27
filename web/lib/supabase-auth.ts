import { NEXT_PUBLIC_SUPABASE_DIRECT_AUTH_ENABLED } from "@/lib/feature-flags";

const SUPABASE_URL = (process.env.NEXT_PUBLIC_SUPABASE_URL ?? "").trim().replace(/\/+$/, "");
const SUPABASE_ANON_KEY = (process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY ?? "").trim();

export const BOMINAL_SUPABASE_ACCESS_TOKEN_STORAGE_KEY = "bominal_supabase_access_token";
const SUPABASE_JS_AUTH_TOKEN_KEY_SUFFIX = "-auth-token";

function readStorageValue(storage: Storage | null | undefined, key: string): string | null {
  if (!storage) return null;
  try {
    const raw = storage.getItem(key);
    if (!raw) return null;
    const normalized = raw.trim();
    return normalized.length > 0 ? normalized : null;
  } catch {
    return null;
  }
}

function extractAccessTokenFromStructuredPayload(raw: string): string | null {
  try {
    const parsed = JSON.parse(raw) as
      | {
          access_token?: unknown;
          currentSession?: { access_token?: unknown } | null;
        }
      | null;
    if (!parsed || typeof parsed !== "object") return null;

    if (typeof parsed.access_token === "string" && parsed.access_token.trim().length > 0) {
      return parsed.access_token.trim();
    }

    const nestedToken = parsed.currentSession?.access_token;
    if (typeof nestedToken === "string" && nestedToken.trim().length > 0) {
      return nestedToken.trim();
    }
    return null;
  } catch {
    return null;
  }
}

function readSupabaseJsTokenFromStorage(storage: Storage | null | undefined): string | null {
  if (!storage) return null;
  try {
    for (let index = 0; index < storage.length; index += 1) {
      const key = storage.key(index);
      if (!key || !key.endsWith(SUPABASE_JS_AUTH_TOKEN_KEY_SUFFIX)) {
        continue;
      }
      const raw = storage.getItem(key);
      if (!raw) continue;
      const parsedToken = extractAccessTokenFromStructuredPayload(raw);
      if (parsedToken) {
        return parsedToken;
      }
    }
  } catch {
    return null;
  }
  return null;
}

function hasSupabaseBrowserConfig(): boolean {
  return SUPABASE_URL.length > 0 && SUPABASE_ANON_KEY.length > 0;
}

export function isSupabaseDirectAuthEnabled(): boolean {
  return NEXT_PUBLIC_SUPABASE_DIRECT_AUTH_ENABLED && hasSupabaseBrowserConfig();
}

export async function getSupabaseAccessToken(): Promise<string | null> {
  if (typeof window === "undefined") {
    return null;
  }

  // Recovery/magic-link flows may cache token even when direct-auth feature
  // flags are disabled, but Data API/GraphQL callers must remain gated.
  const cachedToken =
    readStorageValue(window.sessionStorage, BOMINAL_SUPABASE_ACCESS_TOKEN_STORAGE_KEY) ??
    readStorageValue(window.localStorage, BOMINAL_SUPABASE_ACCESS_TOKEN_STORAGE_KEY);
  if (cachedToken) {
    return cachedToken;
  }
  if (!isSupabaseDirectAuthEnabled()) {
    return null;
  }

  // Prefer the explicit token cache when direct-auth bootstrap writes it.
  // Fallback for future/alternate clients using default supabase-js storage keys.
  return readSupabaseJsTokenFromStorage(window.sessionStorage) ?? readSupabaseJsTokenFromStorage(window.localStorage);
}

export function cacheSupabaseAccessToken(accessToken: string): void {
  if (typeof window === "undefined") {
    return;
  }
  const normalized = accessToken.trim();
  if (!normalized) {
    return;
  }
  try {
    window.sessionStorage.setItem(BOMINAL_SUPABASE_ACCESS_TOKEN_STORAGE_KEY, normalized);
  } catch {
    // Ignore storage failures in restricted browser contexts.
  }
}

export function clearSupabaseAccessToken(): void {
  if (typeof window === "undefined") {
    return;
  }
  try {
    window.sessionStorage.removeItem(BOMINAL_SUPABASE_ACCESS_TOKEN_STORAGE_KEY);
  } catch {
    // Ignore storage failures in restricted browser contexts.
  }
  try {
    window.localStorage.removeItem(BOMINAL_SUPABASE_ACCESS_TOKEN_STORAGE_KEY);
  } catch {
    // Ignore storage failures in restricted browser contexts.
  }
}
