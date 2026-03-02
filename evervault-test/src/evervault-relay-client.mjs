function normalizePath(pathname) {
  const input = String(pathname || "").trim();
  if (!input) {
    throw new Error("Listener path is required");
  }
  if (!input.startsWith("/")) {
    return `/${input}`;
  }
  return input;
}

function basicAuthHeader(appId, apiKey) {
  const token = Buffer.from(`${appId}:${apiKey}`, "utf8").toString("base64");
  return `Basic ${token}`;
}

function assertRequiredEnv(value, name) {
  if (!String(value || "").trim()) {
    throw new Error(`${name} is required`);
  }
}

function relaySignature(relay) {
  const routes = Array.isArray(relay?.routes) ? relay.routes : [];
  return routes.map((route) => {
    const request = Array.isArray(route?.request) ? route.request[0] : null;
    const selections = Array.isArray(request?.selections) ? request.selections : [];
    return {
      method: String(route?.method || "").toUpperCase(),
      path: String(route?.path || ""),
      action: String(request?.action || "").toLowerCase(),
      selections: selections.map((selection) => ({
        type: String(selection?.type || "").toLowerCase(),
        selector: String(selection?.selector || ""),
      })),
    };
  });
}

function relayMatchesDefinition(relay, expectedDefinition) {
  if (String(relay?.destinationDomain || "").toLowerCase() !== String(expectedDefinition.destinationDomain).toLowerCase()) {
    return false;
  }

  if (String(relay?.authentication || "").toLowerCase() !== "api-key") {
    return false;
  }

  return JSON.stringify(relaySignature(relay)) === JSON.stringify(relaySignature(expectedDefinition));
}

async function evervaultManagementRequest({
  appId,
  apiKey,
  apiBaseUrl,
  method,
  path,
  body,
  fetchImpl,
}) {
  const response = await fetchImpl(`${apiBaseUrl.replace(/\/$/, "")}${path}`, {
    method,
    headers: {
      Authorization: basicAuthHeader(appId, apiKey),
      Accept: "application/json",
      ...(body ? { "Content-Type": "application/json" } : {}),
    },
    body: body ? JSON.stringify(body) : undefined,
  });

  let payload = null;
  try {
    payload = await response.json();
  } catch {
    payload = null;
  }

  if (!response.ok) {
    throw new Error(`Evervault management request failed (${response.status})`);
  }

  return payload;
}

function extractRelayRuntime(relay) {
  const relayId = String(relay?.id || "").trim();
  const relayDomain = String(relay?.evervaultDomain || "").trim().toLowerCase();
  if (!relayId || !relayDomain.endsWith(".relay.evervault.app")) {
    throw new Error("Relay runtime response is invalid");
  }
  return { relayId, relayDomain };
}

export function buildExpectedRelayDefinition({ destinationDomain, listenerPath, cardListenerPath, srtListenerPath }) {
  const panPath = normalizePath(listenerPath);
  const cardPath = normalizePath(cardListenerPath || `${listenerPath}-card`);
  const srtPath = normalizePath(srtListenerPath || "/evervault-test/srt-listener");
  return {
    destinationDomain: String(destinationDomain || "").trim().toLowerCase(),
    authentication: "api-key",
    encryptEmptyStrings: true,
    routes: [
      {
        method: "POST",
        path: panPath,
        request: [
          {
            action: "decrypt",
            selections: [{ type: "form", selector: "encrypted_card_number" }],
          },
        ],
        response: [],
      },
      {
        method: "POST",
        path: cardPath,
        request: [
          {
            action: "decrypt",
            selections: [
              { type: "form", selector: "encrypted_card_number" },
              { type: "form", selector: "encrypted_card_cvc" },
            ],
          },
        ],
        response: [],
      },
      {
        method: "POST",
        path: srtPath,
        request: [
          {
            action: "decrypt",
            selections: [
              { type: "json", selector: "$.payment.card.number" },
              { type: "json", selector: "$.payment.card.expiry_month" },
              { type: "json", selector: "$.payment.card.expiry_year" },
              { type: "json", selector: "$.payment.card.cvc" },
              { type: "header", selector: "authorization" },
              { type: "header", selector: "x-srt-encrypted-pan" },
            ],
          },
        ],
        response: [],
      },
    ],
  };
}

export async function probeManagementAuth({ appId, apiKey, apiBaseUrl = "https://api.evervault.com", fetchImpl = fetch }) {
  assertRequiredEnv(appId, "EVERVAULT_APP_ID");
  assertRequiredEnv(apiKey, "EVERVAULT_API_KEY");

  const payload = await evervaultManagementRequest({
    appId,
    apiKey,
    apiBaseUrl,
    method: "GET",
    path: "/relays",
    fetchImpl,
  });

  const rows = Array.isArray(payload?.data) ? payload.data : [];
  return { relayCount: rows.length };
}

export async function ensureRelayDefinition({
  appId,
  apiKey,
  apiBaseUrl = "https://api.evervault.com",
  destinationDomain,
  listenerPath,
  cardListenerPath,
  srtListenerPath,
  fetchImpl = fetch,
}) {
  assertRequiredEnv(appId, "EVERVAULT_APP_ID");
  assertRequiredEnv(apiKey, "EVERVAULT_API_KEY");
  assertRequiredEnv(destinationDomain, "EV_TEST_DESTINATION_DOMAIN");

  const definition = buildExpectedRelayDefinition({ destinationDomain, listenerPath, cardListenerPath, srtListenerPath });

  const listPayload = await evervaultManagementRequest({
    appId,
    apiKey,
    apiBaseUrl,
    method: "GET",
    path: "/relays",
    fetchImpl,
  });

  const relays = Array.isArray(listPayload?.data) ? listPayload.data.filter((row) => row && typeof row === "object") : [];
  let selected = relays.find(
    (relay) => String(relay.destinationDomain || "").toLowerCase() === String(definition.destinationDomain).toLowerCase(),
  );

  if (!selected) {
    selected = await evervaultManagementRequest({
      appId,
      apiKey,
      apiBaseUrl,
      method: "POST",
      path: "/relays",
      body: definition,
      fetchImpl,
    });
    const runtime = extractRelayRuntime(selected);
    return {
      ...runtime,
      listenerPath: normalizePath(listenerPath),
      cardListenerPath: normalizePath(cardListenerPath || `${listenerPath}-card`),
      srtListenerPath: normalizePath(srtListenerPath || "/evervault-test/srt-listener"),
      destinationDomain: definition.destinationDomain,
    };
  }

  if (!relayMatchesDefinition(selected, definition)) {
    const relayId = String(selected.id || "").trim();
    if (!relayId) {
      throw new Error("Relay is missing ID");
    }

    selected = await evervaultManagementRequest({
      appId,
      apiKey,
      apiBaseUrl,
      method: "PATCH",
      path: `/relays/${relayId}`,
      body: {
        authentication: definition.authentication,
        encryptEmptyStrings: definition.encryptEmptyStrings,
        routes: definition.routes,
      },
      fetchImpl,
    });
  }

  const runtime = extractRelayRuntime(selected);
  return {
    ...runtime,
    listenerPath: normalizePath(listenerPath),
    cardListenerPath: normalizePath(cardListenerPath || `${listenerPath}-card`),
    srtListenerPath: normalizePath(srtListenerPath || "/evervault-test/srt-listener"),
    destinationDomain: definition.destinationDomain,
  };
}

export async function dispatchViaRelay({
  appId,
  apiKey,
  relayDomain,
  listenerPath,
  formData,
  timeoutMs = 20000,
  fetchImpl = fetch,
}) {
  assertRequiredEnv(appId, "EVERVAULT_APP_ID");
  assertRequiredEnv(apiKey, "EVERVAULT_API_KEY");
  assertRequiredEnv(relayDomain, "relayDomain");

  const normalizedPath = normalizePath(listenerPath);
  const relayUrl = `https://${String(relayDomain).trim().toLowerCase()}${normalizedPath}`;
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), Math.max(1000, Number(timeoutMs) || 20000));

  let response;
  try {
    response = await fetchImpl(relayUrl, {
      method: "POST",
      headers: {
        "X-Evervault-App-Id": appId,
        "X-Evervault-Api-Key": apiKey,
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: new URLSearchParams(formData).toString(),
      signal: controller.signal,
    });
  } finally {
    clearTimeout(timeout);
  }

  const text = await response.text();
  return {
    relayUrl,
    statusCode: response.status,
    text,
  };
}
