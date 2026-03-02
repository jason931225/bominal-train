import path from "node:path";
import { fileURLToPath } from "node:url";

import dotenv from "dotenv";
import express from "express";

import { dispatchViaRelay, ensureRelayDefinition, probeManagementAuth } from "./src/evervault-relay-client.mjs";
import { SessionStore } from "./src/session-store.mjs";
import { buildRuntimeConfig } from "./src/config.mjs";

dotenv.config();

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const app = express();

const config = buildRuntimeConfig(process.env);

const sessionStore = new SessionStore({ ttlSeconds: config.resultTtlSeconds });

setInterval(() => {
  sessionStore.cleanupExpired();
}, 10000).unref();

app.disable("x-powered-by");
app.use(express.json({ limit: "100kb" }));
app.use(express.urlencoded({ extended: false }));
app.use(express.static(path.join(__dirname, "public")));

function missingConfigKeys() {
  const keys = [];
  if (!config.evervaultTeamId) keys.push("EVERVAULT_TEAM_ID");
  if (!config.evervaultAppId) keys.push("EVERVAULT_APP_ID");
  if (!config.evervaultApiKey) keys.push("EVERVAULT_API_KEY");
  if (!config.sharedSecret) keys.push("EV_TEST_SHARED_SECRET");
  return keys;
}

function toErrorMessage(error, fallback) {
  const text = String(error?.message || "").trim();
  return text || fallback;
}

function sanitizeRelayBodySnippet(text) {
  const truncated = String(text || "").slice(0, 240);
  return truncated.replace(/\s+/g, " ").trim();
}

app.get("/api/test/config", (_req, res) => {
  res.json({
    evervault_team_id: config.evervaultTeamId,
    evervault_app_id: config.evervaultAppId,
    poll_timeout_seconds: config.pollTimeoutSeconds,
    listener_path: config.listenerPath,
    destination_domain: config.destinationDomain,
  });
});

app.post("/api/test/self-check", async (_req, res) => {
  const missingKeys = missingConfigKeys();
  if (missingKeys.length > 0) {
    return res.status(400).json({
      ok: false,
      detail: `Missing required config: ${missingKeys.join(", ")}`,
    });
  }

  try {
    const probe = await probeManagementAuth({
      appId: config.evervaultAppId,
      apiKey: config.evervaultApiKey,
      apiBaseUrl: config.evervaultApiBaseUrl,
    });

    return res.json({
      ok: true,
      detail: "Evervault management auth check passed",
      relay_count: probe.relayCount,
      destination_domain: config.destinationDomain,
      listener_path: config.listenerPath,
    });
  } catch (error) {
    return res.status(502).json({
      ok: false,
      detail: toErrorMessage(error, "Evervault management auth check failed"),
    });
  }
});

app.post("/api/test/run", async (req, res) => {
  const missingKeys = missingConfigKeys();
  if (missingKeys.length > 0) {
    return res.status(400).json({
      ok: false,
      detail: `Missing required config: ${missingKeys.join(", ")}`,
    });
  }

  const encryptedCardNumber = String(req.body?.encrypted_card_number || "").trim();
  const expectedLast4 = String(req.body?.expected_last4 || "").trim();
  const revealFullOnce = Boolean(req.body?.reveal_full_once);

  if (!encryptedCardNumber || !encryptedCardNumber.startsWith("ev:")) {
    return res.status(400).json({
      ok: false,
      detail: "encrypted_card_number must be an Evervault-encrypted token",
    });
  }

  if (!/^\d{4}$/.test(expectedLast4)) {
    return res.status(400).json({
      ok: false,
      detail: "expected_last4 must be exactly 4 digits",
    });
  }

  const created = sessionStore.createSession({
    expectedLast4,
    revealFullOnce,
  });

  try {
    const relayRuntime = await ensureRelayDefinition({
      appId: config.evervaultAppId,
      apiKey: config.evervaultApiKey,
      apiBaseUrl: config.evervaultApiBaseUrl,
      destinationDomain: config.destinationDomain,
      listenerPath: config.listenerPath,
    });

    const relayResponse = await dispatchViaRelay({
      appId: config.evervaultAppId,
      apiKey: config.evervaultApiKey,
      relayDomain: relayRuntime.relayDomain,
      listenerPath: relayRuntime.listenerPath,
      timeoutMs: config.pollTimeoutSeconds * 1000,
      formData: {
        encrypted_card_number: encryptedCardNumber,
        session_id: created.id,
        session_nonce: created.nonce,
        expected_last4: expectedLast4,
        shared_secret: config.sharedSecret,
      },
    });

    sessionStore.recordRelayDispatch(created.id, {
      relayId: relayRuntime.relayId,
      relayDomain: relayRuntime.relayDomain,
      destinationDomain: relayRuntime.destinationDomain,
      listenerPath: relayRuntime.listenerPath,
      dispatchStatusCode: relayResponse.statusCode,
      dispatchResponseSnippet: sanitizeRelayBodySnippet(relayResponse.text),
    });

    if (relayResponse.statusCode >= 400) {
      sessionStore.recordFailure(created.id, `Relay request failed with status ${relayResponse.statusCode}`);
    }

    return res.status(202).json({
      ok: true,
      session_id: created.id,
      status: "pending",
      relay: {
        relay_id: relayRuntime.relayId,
        relay_domain: relayRuntime.relayDomain,
        destination_domain: relayRuntime.destinationDomain,
        listener_path: relayRuntime.listenerPath,
        dispatch_status_code: relayResponse.statusCode,
      },
    });
  } catch (error) {
    sessionStore.recordFailure(created.id, toErrorMessage(error, "Relay flow setup failed"));

    return res.status(502).json({
      ok: false,
      session_id: created.id,
      detail: toErrorMessage(error, "Relay flow setup failed"),
    });
  }
});

app.get("/api/test/result/:sessionId", (req, res) => {
  const sessionId = String(req.params.sessionId || "").trim();
  if (!sessionId) {
    return res.status(400).json({
      ok: false,
      detail: "sessionId is required",
    });
  }

  const result = sessionStore.getResult(sessionId);
  if (result.status === "not_found") {
    return res.status(404).json(result);
  }
  return res.json(result);
});

app.post("/evervault-test/relay-listener", (req, res) => {
  const sharedSecret = String(req.body?.shared_secret || "").trim();
  if (sharedSecret !== config.sharedSecret) {
    return res.status(401).json({ ok: false, detail: "listener shared_secret mismatch" });
  }

  const sessionId = String(req.body?.session_id || "").trim();
  const nonce = String(req.body?.session_nonce || "").trim();
  const decryptedPan = String(req.body?.encrypted_card_number || "").trim();

  if (!sessionId || !nonce || !decryptedPan) {
    return res.status(400).json({ ok: false, detail: "missing session_id, session_nonce, or encrypted_card_number" });
  }

  const receipt = sessionStore.recordListenerReceipt({ sessionId, nonce, decryptedPan });
  if (!receipt.ok) {
    return res.status(400).json({ ok: false, detail: receipt.error });
  }

  const result = sessionStore.getResult(sessionId);
  return res.json({
    ok: true,
    status: result.status,
    masked_pan: result?.proof?.masked_pan || null,
    matched_expected_last4: result?.proof?.matched_expected_last4 || false,
  });
});

app.listen(config.port, "0.0.0.0", () => {
  // Intentionally avoid logging request payloads to protect decrypted values.
  console.log(`evervault-test listening on http://0.0.0.0:${config.port}`);
});
