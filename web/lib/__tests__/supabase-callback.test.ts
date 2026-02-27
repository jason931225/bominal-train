import { describe, expect, it } from "vitest";

import { resolveSupabaseCallbackExchangePayload } from "@/lib/supabase-callback";

describe("resolveSupabaseCallbackExchangePayload", () => {
  it("uses query token_hash and type when present", () => {
    const payload = resolveSupabaseCallbackExchangePayload(
      new URLSearchParams("token_hash=hash-abc123&type=magiclink"),
      "",
    );
    expect(payload).toEqual({ token_hash: "hash-abc123", type: "magiclink" });
  });

  it("falls back to hash access token flow", () => {
    const payload = resolveSupabaseCallbackExchangePayload(
      new URLSearchParams(""),
      "#access_token=jwt-token-abc&type=recovery",
    );
    expect(payload).toEqual({ access_token: "jwt-token-abc", type: "recovery" });
  });

  it("returns null when type is missing", () => {
    const payload = resolveSupabaseCallbackExchangePayload(
      new URLSearchParams("token_hash=hash-abc123"),
      "#access_token=jwt-token-abc",
    );
    expect(payload).toBeNull();
  });

  it("returns null for unsupported type", () => {
    const payload = resolveSupabaseCallbackExchangePayload(
      new URLSearchParams("token_hash=hash-abc123&type=unknown"),
      "",
    );
    expect(payload).toBeNull();
  });
});
