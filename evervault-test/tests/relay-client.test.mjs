import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { buildExpectedRelayDefinition } from "../src/evervault-relay-client.mjs";

describe("buildExpectedRelayDefinition", () => {
  it("builds decrypt routes for PAN, UI Card, and mock SRT flows", () => {
    const payload = buildExpectedRelayDefinition({
      destinationDomain: "www.bominal.com",
      listenerPath: "/evervault-test/relay-listener",
      cardListenerPath: "/evervault-test/relay-listener-card",
      srtListenerPath: "/evervault-test/srt-listener",
    });

    assert.equal(payload.destinationDomain, "www.bominal.com");
    assert.equal(payload.authentication, "api-key");
    assert.equal(payload.encryptEmptyStrings, true);
    assert.equal(payload.routes.length, 3);
    assert.equal(payload.routes[0].method, "POST");
    assert.equal(payload.routes[0].path, "/evervault-test/relay-listener");

    const selectors = payload.routes[0].request[0].selections;
    assert.deepEqual(selectors, [{ type: "form", selector: "encrypted_card_number" }]);

    assert.equal(payload.routes[1].path, "/evervault-test/relay-listener-card");
    assert.deepEqual(payload.routes[1].request[0].selections, [
      { type: "form", selector: "encrypted_card_number" },
      { type: "form", selector: "encrypted_card_cvc" },
    ]);

    assert.equal(payload.routes[2].path, "/evervault-test/srt-listener");
    assert.deepEqual(payload.routes[2].request[0].selections, [
      { type: "json", selector: "$.payment.card.number" },
      { type: "json", selector: "$.payment.card.expiry_month" },
      { type: "json", selector: "$.payment.card.expiry_year" },
      { type: "json", selector: "$.payment.card.cvc" },
      { type: "header", selector: "authorization" },
      { type: "header", selector: "x-srt-encrypted-pan" },
    ]);
  });
});
