function clean(value) {
  return String(value || "").trim();
}

function normalizeHost(value) {
  const raw = clean(value);
  if (!raw) {
    return "";
  }

  let host = raw;
  if (host.startsWith("http://") || host.startsWith("https://")) {
    try {
      host = new URL(host).host;
    } catch {
      host = host.replace(/^https?:\/\//, "");
    }
  }

  if (host.includes("/")) {
    host = host.split("/", 1)[0];
  }

  return host.toLowerCase();
}

export function buildRuntimeConfig(env = process.env) {
  const explicitDomain = clean(env.EV_TEST_DESTINATION_DOMAIN);
  const caddyDomain = clean(env.CADDY_SITE_ADDRESS);

  const destinationDomain = normalizeHost(explicitDomain || caddyDomain || "www.bominal.com") || "www.bominal.com";

  return {
    port: Number(env.PORT || 8787),
    evervaultTeamId: clean(env.EVERVAULT_TEAM_ID || env.NEXT_PUBLIC_EVERVAULT_TEAM_ID),
    evervaultAppId: clean(env.EVERVAULT_APP_ID || env.NEXT_PUBLIC_EVERVAULT_APP_ID),
    evervaultApiKey: clean(env.EVERVAULT_API_KEY),
    evervaultApiBaseUrl: clean(env.EVERVAULT_API_BASE_URL || "https://api.evervault.com"),
    destinationDomain,
    listenerPath: clean(env.EV_TEST_LISTENER_PATH || "/evervault-test/relay-listener"),
    cardListenerPath: clean(env.EV_TEST_CARD_LISTENER_PATH || "/evervault-test/relay-listener-card"),
    srtListenerPath: clean(env.EV_TEST_SRT_LISTENER_PATH || "/evervault-test/srt-listener"),
    sharedSecret: clean(env.EV_TEST_SHARED_SECRET),
    resultTtlSeconds: Number(env.EV_TEST_RESULT_TTL_SECONDS || 120),
    pollTimeoutSeconds: Number(env.EV_TEST_POLL_TIMEOUT_SECONDS || 20),
  };
}
