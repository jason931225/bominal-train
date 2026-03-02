import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { buildRuntimeConfig } from "../src/config.mjs";

describe("buildRuntimeConfig", () => {
  it("falls back to NEXT_PUBLIC Evervault vars from prod web env", () => {
    const config = buildRuntimeConfig({
      PORT: "8787",
      NEXT_PUBLIC_EVERVAULT_TEAM_ID: "team_from_web_env",
      NEXT_PUBLIC_EVERVAULT_APP_ID: "app_from_web_env",
      EVERVAULT_API_KEY: "key_from_api_env",
      EV_TEST_SHARED_SECRET: "secret",
    });

    assert.equal(config.evervaultTeamId, "team_from_web_env");
    assert.equal(config.evervaultAppId, "app_from_web_env");
  });

  it("uses caddy site address as destination domain when explicit test domain missing", () => {
    const config = buildRuntimeConfig({
      NEXT_PUBLIC_EVERVAULT_TEAM_ID: "team",
      EVERVAULT_APP_ID: "app",
      EVERVAULT_API_KEY: "key",
      EV_TEST_SHARED_SECRET: "secret",
      CADDY_SITE_ADDRESS: "https://www.bominal.com",
    });

    assert.equal(config.destinationDomain, "www.bominal.com");
  });
});
