import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  isPasskeySupported,
  listPasskeysFromSession,
  registerPasskeyFromSession,
  removePasskeyFromSession,
  signInWithPasskey,
} from "@/lib/passkey";

const originalPublicKeyCredential = globalThis.PublicKeyCredential;

function setNavigatorCredentials(mock: { create?: unknown; get?: unknown }): void {
  Object.defineProperty(globalThis.navigator, "credentials", {
    configurable: true,
    value: mock,
  });
}

function installPasskeySupport() {
  class FakePublicKeyCredential {}
  Object.defineProperty(globalThis, "PublicKeyCredential", {
    configurable: true,
    value: FakePublicKeyCredential,
  });
}

function fakeRegistrationCredential() {
  return {
    id: "cred-1",
    rawId: Uint8Array.from([1, 2, 3]).buffer,
    type: "public-key",
    response: {
      attestationObject: Uint8Array.from([4, 5]).buffer,
      clientDataJSON: Uint8Array.from([6, 7]).buffer,
      getTransports: () => ["internal"],
    },
    getClientExtensionResults: () => ({}),
  };
}

function fakeRegistrationCredentialWithoutTransports() {
  return {
    id: "cred-2",
    rawId: Uint8Array.from([9, 8, 7]).buffer,
    type: "public-key",
    response: {
      attestationObject: Uint8Array.from([6, 5]).buffer,
      clientDataJSON: Uint8Array.from([4, 3]).buffer,
    },
    getClientExtensionResults: () => ({ appid: true }),
  };
}

function fakeAuthenticationCredential() {
  return {
    id: "cred-1",
    rawId: Uint8Array.from([1, 2, 3]).buffer,
    type: "public-key",
    response: {
      authenticatorData: Uint8Array.from([4]).buffer,
      clientDataJSON: Uint8Array.from([5]).buffer,
      signature: Uint8Array.from([6]).buffer,
      userHandle: null,
    },
    getClientExtensionResults: () => ({}),
  };
}

function fakeAuthenticationCredentialWithUserHandle() {
  return {
    id: "cred-2",
    rawId: Uint8Array.from([3, 2, 1]).buffer,
    type: "public-key",
    response: {
      authenticatorData: Uint8Array.from([9]).buffer,
      clientDataJSON: Uint8Array.from([8]).buffer,
      signature: Uint8Array.from([7]).buffer,
      userHandle: Uint8Array.from([6]).buffer,
    },
    getClientExtensionResults: () => ({ uv: true }),
  };
}

function installFetchMock(sequence: Array<Response | Promise<Response>>) {
  const mock = vi.fn();
  for (const item of sequence) {
    mock.mockImplementationOnce(async () => item);
  }
  vi.stubGlobal("fetch", mock as unknown as typeof fetch);
  return mock;
}

describe("passkey helpers", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  afterEach(() => {
    Object.defineProperty(globalThis, "PublicKeyCredential", {
      configurable: true,
      value: originalPublicKeyCredential,
    });
  });

  it("reports unsupported when credentials API is unavailable", () => {
    Object.defineProperty(globalThis, "PublicKeyCredential", {
      configurable: true,
      value: undefined,
    });
    setNavigatorCredentials({});
    expect(isPasskeySupported()).toBe(false);
  });

  it("returns unsupported errors when register/sign-in are attempted without passkey support", async () => {
    Object.defineProperty(globalThis, "PublicKeyCredential", {
      configurable: true,
      value: undefined,
    });
    setNavigatorCredentials({});

    await expect(registerPasskeyFromSession("http://api")).resolves.toEqual({
      ok: false,
      error: "Passkeys are not supported on this device.",
    });
    await expect(signInWithPasskey("http://api", { email: "user@example.com", rememberMe: false })).resolves.toEqual({
      ok: false,
      error: "Passkeys are not supported on this device.",
    });
  });

  it("registers a passkey with session credentials", async () => {
    installPasskeySupport();
    setNavigatorCredentials({
      create: vi.fn().mockResolvedValue(fakeRegistrationCredential()),
    });

    const fetchMock = installFetchMock([
      new Response(
        JSON.stringify({ challenge_id: "c1", public_key: { challenge: "AQID", user: { id: "AQID" } } }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
      new Response(JSON.stringify({ id: "pk1" }), { status: 200, headers: { "Content-Type": "application/json" } }),
    ]);

    const result = await registerPasskeyFromSession("http://api");
    expect(result.ok).toBe(true);
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it("registers with excludeCredentials and no transports function", async () => {
    installPasskeySupport();
    setNavigatorCredentials({
      create: vi.fn().mockResolvedValue(fakeRegistrationCredentialWithoutTransports()),
    });

    installFetchMock([
      new Response(
        JSON.stringify({
          challenge_id: "c2",
          public_key: {
            challenge: "AQID",
            user: { id: "AQID" },
            excludeCredentials: [{ id: "AQID", type: "public-key" }],
          },
        }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
      new Response(JSON.stringify({ id: "pk2" }), { status: 200, headers: { "Content-Type": "application/json" } }),
    ]);

    const result = await registerPasskeyFromSession("http://api");
    expect(result).toEqual({ ok: true });
  });

  it("handles passkey registration failures", async () => {
    installPasskeySupport();
    setNavigatorCredentials({ create: vi.fn().mockResolvedValue(null) });
    installFetchMock([
      new Response(
        JSON.stringify({ challenge_id: "c1", public_key: { challenge: "AQID", user: { id: "AQID" } } }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
    ]);

    let result = await registerPasskeyFromSession("http://api");
    expect(result.ok).toBe(false);

    setNavigatorCredentials({ create: vi.fn().mockResolvedValue(fakeRegistrationCredential()) });
    installFetchMock([
      new Response(JSON.stringify({ detail: "nope" }), { status: 400, headers: { "Content-Type": "application/json" } }),
    ]);
    result = await registerPasskeyFromSession("http://api");
    expect(result.ok).toBe(false);
    expect(result.error).toContain("nope");

    installFetchMock([
      new Response(
        JSON.stringify({ challenge_id: "c1", public_key: { challenge: "AQID", user: { id: "AQID" } } }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
      new Response("verify failed", { status: 500, headers: { "Content-Type": "text/plain" } }),
    ]);
    result = await registerPasskeyFromSession("http://api");
    expect(result.ok).toBe(false);
    expect(result.error).toContain("verify failed");
  });

  it("signs in with passkey", async () => {
    installPasskeySupport();
    setNavigatorCredentials({
      get: vi.fn().mockResolvedValue(fakeAuthenticationCredential()),
    });

    installFetchMock([
      new Response(
        JSON.stringify({
          challenge_id: "auth-1",
          public_key: {
            challenge: "AQID",
            allowCredentials: [{ id: "AQID", type: "public-key" }],
          },
        }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
      new Response(JSON.stringify({ user: { id: "u1" } }), { status: 200, headers: { "Content-Type": "application/json" } }),
    ]);

    const result = await signInWithPasskey("http://api", { email: "user@example.com", rememberMe: true });
    expect(result.ok).toBe(true);
  });

  it("signs in with passkey when assertion includes userHandle", async () => {
    installPasskeySupport();
    setNavigatorCredentials({
      get: vi.fn().mockResolvedValue(fakeAuthenticationCredentialWithUserHandle()),
    });

    installFetchMock([
      new Response(
        JSON.stringify({
          challenge_id: "auth-2",
          public_key: {
            challenge: "AQID",
            allowCredentials: [{ id: "AQID", type: "public-key" }],
          },
        }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
      new Response(JSON.stringify({ user: { id: "u2" } }), { status: 200, headers: { "Content-Type": "application/json" } }),
    ]);

    const result = await signInWithPasskey("http://api", { email: "user2@example.com", rememberMe: false });
    expect(result).toEqual({ ok: true });
  });

  it("handles passkey sign-in failures", async () => {
    installPasskeySupport();
    setNavigatorCredentials({ get: vi.fn().mockResolvedValue(null) });
    installFetchMock([
      new Response(
        JSON.stringify({ challenge_id: "auth-1", public_key: { challenge: "AQID", allowCredentials: [] } }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
    ]);

    let result = await signInWithPasskey("http://api", { email: "user@example.com", rememberMe: false });
    expect(result.ok).toBe(false);

    setNavigatorCredentials({ get: vi.fn().mockResolvedValue(fakeAuthenticationCredential()) });
    installFetchMock([
      new Response(JSON.stringify({ detail: "missing passkey" }), { status: 400, headers: { "Content-Type": "application/json" } }),
    ]);
    result = await signInWithPasskey("http://api", { email: "user@example.com", rememberMe: false });
    expect(result.ok).toBe(false);
    expect(result.error).toContain("missing passkey");

    installFetchMock([
      new Response(
        JSON.stringify({ challenge_id: "auth-1", public_key: { challenge: "AQID", allowCredentials: [] } }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      ),
      new Response("bad verify", { status: 500, headers: { "Content-Type": "text/plain" } }),
    ]);
    result = await signInWithPasskey("http://api", { email: "user@example.com", rememberMe: false });
    expect(result.ok).toBe(false);
    expect(result.error).toContain("bad verify");
  });

  it("uses fallback error messages for malformed JSON error payloads", async () => {
    installPasskeySupport();
    setNavigatorCredentials({});

    installFetchMock([
      new Response("not-json", { status: 500, headers: { "Content-Type": "application/json" } }),
    ]);
    const registerResult = await registerPasskeyFromSession("http://api");
    expect(registerResult).toEqual({
      ok: false,
      error: "Could not start passkey setup.",
    });

    installFetchMock([
      new Response("still-not-json", { status: 400, headers: { "Content-Type": "application/json" } }),
    ]);
    const signInResult = await signInWithPasskey("http://api", { email: "user@example.com", rememberMe: false });
    expect(signInResult).toEqual({
      ok: false,
      error: "Could not start passkey sign in.",
    });
  });

  it("loads and removes passkeys from session", async () => {
    installPasskeySupport();
    setNavigatorCredentials({});

    installFetchMock([
      new Response(JSON.stringify({ credentials: [{ id: "pk-1", created_at: "2026-01-01T00:00:00Z", last_used_at: null }] }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    ]);
    const list = await listPasskeysFromSession("http://api");
    expect(list.credentials).toHaveLength(1);

    installFetchMock([new Response(null, { status: 200 })]);
    await expect(removePasskeyFromSession("http://api", "pk-1")).resolves.toBeUndefined();
  });

  it("propagates list/remove errors with parsed details", async () => {
    installPasskeySupport();
    setNavigatorCredentials({});

    installFetchMock([new Response(JSON.stringify({ detail: "list failed" }), { status: 500, headers: { "Content-Type": "application/json" } })]);
    await expect(listPasskeysFromSession("http://api")).rejects.toThrow("list failed");

    installFetchMock([new Response("remove failed", { status: 400, headers: { "Content-Type": "text/plain" } })]);
    await expect(removePasskeyFromSession("http://api", "pk-1")).rejects.toThrow("remove failed");
  });
});
