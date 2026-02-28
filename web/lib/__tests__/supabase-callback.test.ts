import { describe, expect, it } from "vitest";

import { resolveSupabaseConfirmPayload } from "@/lib/supabase-callback";

describe("resolveSupabaseConfirmPayload", () => {
  it("uses query token_hash and type when present", () => {
    const payload = resolveSupabaseConfirmPayload(
      new URLSearchParams("token_hash=hash-abc123&type=magiclink"),
    );
    expect(payload).toEqual({ token_hash: "hash-abc123", type: "magiclink" });
  });

  it("supports recovery type", () => {
    const payload = resolveSupabaseConfirmPayload(
      new URLSearchParams("token_hash=hash-abc123&type=recovery"),
    );
    expect(payload).toEqual({ token_hash: "hash-abc123", type: "recovery" });
  });

  it("returns null when token_hash is missing", () => {
    const payload = resolveSupabaseConfirmPayload(
      new URLSearchParams("type=magiclink"),
    );
    expect(payload).toBeNull();
  });

  it("returns null for unsupported type", () => {
    const payload = resolveSupabaseConfirmPayload(
      new URLSearchParams("token_hash=hash-abc123&type=unknown"),
    );
    expect(payload).toBeNull();
  });

  it("returns null when token_hash is too short", () => {
    const payload = resolveSupabaseConfirmPayload(
      new URLSearchParams("token_hash=short&type=magiclink"),
    );
    expect(payload).toBeNull();
  });
});
