import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { SessionStore } from "../src/session-store.mjs";

describe("SessionStore", () => {
  it("returns encrypted + decrypted proof values after listener receipt", () => {
    let nowMs = Date.UTC(2026, 2, 2, 0, 0, 0);
    const store = new SessionStore({
      ttlSeconds: 120,
      now: () => nowMs,
      randomUUID: () => "session-1",
      randomNonce: () => "nonce-1",
    });

    const encryptedToken = "ev:abcd1234";
    const created = store.createSession({ expectedLast4: "1111", browserEncryptedPan: encryptedToken });
    store.recordRelayDispatch(created.id, { dispatchStatusCode: 200, relayId: "relay_1", relayDomain: "demo.relay.evervault.app" });

    const received = store.recordListenerReceipt({
      sessionId: created.id,
      nonce: created.nonce,
      decryptedPan: "4111111111111111",
      relayEchoEncryptedPan: encryptedToken,
    });
    assert.equal(received.ok, true);

    const result = store.getResult(created.id);
    assert.equal(result.status, "received");
    assert.equal(result.proof.browser_encrypted_pan, encryptedToken);
    assert.equal(result.proof.relay_echo_encrypted_pan, encryptedToken);
    assert.equal(result.proof.decrypted_pan, "4111111111111111");
  });

  it("marks stale pending sessions as expired", () => {
    let nowMs = Date.UTC(2026, 2, 2, 0, 0, 0);
    const store = new SessionStore({
      ttlSeconds: 2,
      now: () => nowMs,
      randomUUID: () => "session-2",
      randomNonce: () => "nonce-2",
    });

    const created = store.createSession({ expectedLast4: "1111", browserEncryptedPan: "ev:xyz" });

    nowMs += 3000;
    const expired = store.getResult(created.id);
    assert.equal(expired.status, "expired");
    assert.match(expired.error, /expired/i);
  });
});
