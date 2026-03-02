import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { SessionStore } from "../src/session-store.mjs";

describe("SessionStore", () => {
  it("returns full pan only once when reveal mode is enabled", () => {
    let nowMs = Date.UTC(2026, 2, 2, 0, 0, 0);
    const store = new SessionStore({
      ttlSeconds: 120,
      now: () => nowMs,
      randomUUID: () => "session-1",
      randomNonce: () => "nonce-1",
    });

    const created = store.createSession({ expectedLast4: "1111", revealFullOnce: true });
    store.recordRelayDispatch(created.id, { dispatchStatusCode: 200, relayId: "relay_1", relayDomain: "demo.relay.evervault.app" });

    const received = store.recordListenerReceipt({
      sessionId: created.id,
      nonce: created.nonce,
      decryptedPan: "4111111111111111",
    });
    assert.equal(received.ok, true);

    const firstResult = store.getResult(created.id);
    assert.equal(firstResult.status, "received");
    assert.equal(firstResult.proof.full_pan_once, "4111111111111111");

    const secondResult = store.getResult(created.id);
    assert.equal(secondResult.status, "received");
    assert.equal("full_pan_once" in secondResult.proof, false);
  });

  it("marks stale pending sessions as expired", () => {
    let nowMs = Date.UTC(2026, 2, 2, 0, 0, 0);
    const store = new SessionStore({
      ttlSeconds: 2,
      now: () => nowMs,
      randomUUID: () => "session-2",
      randomNonce: () => "nonce-2",
    });

    const created = store.createSession({ expectedLast4: "1111", revealFullOnce: false });

    nowMs += 3000;
    const expired = store.getResult(created.id);
    assert.equal(expired.status, "expired");
    assert.match(expired.error, /expired/i);
  });
});
