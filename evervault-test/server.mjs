import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import dotenv from "dotenv";
import express from "express";

import { dispatchViaRelay, ensureRelayDefinition, probeManagementAuth } from "./src/evervault-relay-client.mjs";
import { SessionStore } from "./src/session-store.mjs";
import { buildRuntimeConfig } from "./src/config.mjs";
import { dispatchViaGoOutboundRelay } from "./src/go-relay-runner.mjs";

dotenv.config();

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const app = express();
const publicDir = path.join(__dirname, "public");
const goRelaySenderBin = path.resolve(__dirname, process.env.EV_TEST_GO_RELAY_SENDER_BIN || "bin/relay-sender");

const config = buildRuntimeConfig(process.env);

const sessionStore = new SessionStore({ ttlSeconds: config.resultTtlSeconds });

setInterval(() => {
  sessionStore.cleanupExpired();
}, 10000).unref();

app.disable("x-powered-by");
app.use(express.json({ limit: "100kb" }));
app.use(express.urlencoded({ extended: false }));
app.use(express.static(publicDir));
app.use("/evervault-test", express.static(publicDir));

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

function getConfigHandler(_req, res) {
  res.json({
    evervault_team_id: config.evervaultTeamId,
    evervault_app_id: config.evervaultAppId,
    poll_timeout_seconds: config.pollTimeoutSeconds,
    listener_path: config.listenerPath,
    card_listener_path: config.cardListenerPath,
    srt_listener_path: config.srtListenerPath,
    go_relay_sender_bin: goRelaySenderBin,
    destination_domain: config.destinationDomain,
  });
}

async function selfCheckHandler(_req, res) {
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
      card_listener_path: config.cardListenerPath,
      srt_listener_path: config.srtListenerPath,
      go_relay_sender_bin: goRelaySenderBin,
      go_relay_sender_bin_exists: fs.existsSync(goRelaySenderBin),
    });
  } catch (error) {
    return res.status(502).json({
      ok: false,
      detail: toErrorMessage(error, "Evervault management auth check failed"),
    });
  }
}

async function runHandler(req, res) {
  const missingKeys = missingConfigKeys();
  if (missingKeys.length > 0) {
    return res.status(400).json({
      ok: false,
      detail: `Missing required config: ${missingKeys.join(", ")}`,
    });
  }

  const encryptedCardNumber = String(req.body?.encrypted_card_number || "").trim();
  const expectedLast4 = String(req.body?.expected_last4 || "").trim();

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
    browserEncryptedPan: encryptedCardNumber,
  });

  try {
    if (!fs.existsSync(goRelaySenderBin)) {
      return res.status(500).json({
        ok: false,
        detail: `Go relay sender binary not found: ${goRelaySenderBin}`,
      });
    }

    const relayRuntime = await ensureRelayDefinition({
      appId: config.evervaultAppId,
      apiKey: config.evervaultApiKey,
      apiBaseUrl: config.evervaultApiBaseUrl,
      destinationDomain: config.destinationDomain,
      listenerPath: config.listenerPath,
      cardListenerPath: config.cardListenerPath,
      srtListenerPath: config.srtListenerPath,
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
      input: {
        browser_encrypted_pan: encryptedCardNumber,
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
}

async function runCardHandler(req, res) {
  const missingKeys = missingConfigKeys();
  if (missingKeys.length > 0) {
    return res.status(400).json({
      ok: false,
      detail: `Missing required config: ${missingKeys.join(", ")}`,
    });
  }

  const encryptedCardNumber = String(req.body?.encrypted_card_number || "").trim();
  const encryptedExpiryMonth = String(req.body?.encrypted_card_expiry_month || "").trim();
  const encryptedExpiryYear = String(req.body?.encrypted_card_expiry_year || "").trim();
  const encryptedCvc = String(req.body?.encrypted_card_cvc || "").trim();
  const expectedLast4 = String(req.body?.expected_last4 || "").trim();

  if (!encryptedCardNumber || !encryptedCardNumber.startsWith("ev:")) {
    return res.status(400).json({
      ok: false,
      detail: "encrypted_card_number must be an Evervault-encrypted token",
    });
  }

  if (encryptedCvc && !encryptedCvc.startsWith("ev:")) {
    return res.status(400).json({
      ok: false,
      detail: "encrypted_card_cvc must be Evervault-encrypted when provided",
    });
  }

  if (encryptedExpiryMonth && !/^\d{1,2}$/.test(encryptedExpiryMonth)) {
    return res.status(400).json({
      ok: false,
      detail: "encrypted_card_expiry_month must be plain month digits from UI Card",
    });
  }

  if (encryptedExpiryYear && !/^\d{2,4}$/.test(encryptedExpiryYear)) {
    return res.status(400).json({
      ok: false,
      detail: "encrypted_card_expiry_year must be plain year digits from UI Card",
    });
  }

  const created = sessionStore.createSession({
    expectedLast4,
    browserEncryptedPan: encryptedCardNumber,
    browserEncryptedCard: {
      number: encryptedCardNumber,
      expiry_month: encryptedExpiryMonth,
      expiry_year: encryptedExpiryYear,
      cvc: encryptedCvc,
    },
  });

  try {
    const relayRuntime = await ensureRelayDefinition({
      appId: config.evervaultAppId,
      apiKey: config.evervaultApiKey,
      apiBaseUrl: config.evervaultApiBaseUrl,
      destinationDomain: config.destinationDomain,
      listenerPath: config.listenerPath,
      cardListenerPath: config.cardListenerPath,
      srtListenerPath: config.srtListenerPath,
    });

    const relayResponse = await dispatchViaRelay({
      appId: config.evervaultAppId,
      apiKey: config.evervaultApiKey,
      relayDomain: relayRuntime.relayDomain,
      listenerPath: relayRuntime.cardListenerPath,
      timeoutMs: config.pollTimeoutSeconds * 1000,
      formData: {
        encrypted_card_number: encryptedCardNumber,
        encrypted_card_expiry_month: encryptedExpiryMonth,
        encrypted_card_expiry_year: encryptedExpiryYear,
        encrypted_card_cvc: encryptedCvc,
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
      listenerPath: relayRuntime.cardListenerPath,
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
        listener_path: relayRuntime.cardListenerPath,
        dispatch_status_code: relayResponse.statusCode,
      },
      input: {
        browser_encrypted_card: {
          number: encryptedCardNumber,
          expiry_month: encryptedExpiryMonth,
          expiry_year: encryptedExpiryYear,
          cvc: encryptedCvc,
        },
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
}

async function runSrtGoHandler(req, res) {
  const missingKeys = missingConfigKeys();
  if (missingKeys.length > 0) {
    return res.status(400).json({
      ok: false,
      detail: `Missing required config: ${missingKeys.join(", ")}`,
    });
  }

  const encryptedCardNumber = String(req.body?.encrypted_card_number || "").trim();
  const encryptedExpiryMonth = String(req.body?.encrypted_card_expiry_month || "").trim();
  const encryptedExpiryYear = String(req.body?.encrypted_card_expiry_year || "").trim();
  const encryptedCvc = String(req.body?.encrypted_card_cvc || "").trim();
  const amount = String(req.body?.amount || "1300").trim();
  const currency = String(req.body?.currency || "TWD")
    .trim()
    .toUpperCase();
  const bookingReference = String(req.body?.booking_reference || `SRT-MOCK-${Date.now()}`).trim();

  for (const [field, value] of Object.entries({
    encrypted_card_number: encryptedCardNumber,
    encrypted_card_expiry_month: encryptedExpiryMonth,
    encrypted_card_expiry_year: encryptedExpiryYear,
    encrypted_card_cvc: encryptedCvc,
  })) {
    if (!value || !value.startsWith("ev:")) {
      return res.status(400).json({
        ok: false,
        detail: `${field} must be an Evervault-encrypted token`,
      });
    }
  }

  if (!/^\d+(\.\d{1,2})?$/.test(amount)) {
    return res.status(400).json({ ok: false, detail: "amount must be numeric (string or number)" });
  }

  if (!/^[A-Z]{3}$/.test(currency)) {
    return res.status(400).json({ ok: false, detail: "currency must be a 3-letter code" });
  }

  try {
    const relayRuntime = await ensureRelayDefinition({
      appId: config.evervaultAppId,
      apiKey: config.evervaultApiKey,
      apiBaseUrl: config.evervaultApiBaseUrl,
      destinationDomain: config.destinationDomain,
      listenerPath: config.listenerPath,
      cardListenerPath: config.cardListenerPath,
      srtListenerPath: config.srtListenerPath,
    });

    const outboundHeaders = {
      "Content-Type": "application/json",
      Authorization: `Bearer ${encryptedCardNumber}`,
      "X-SRT-Encrypted-PAN": encryptedCardNumber,
      "X-SRT-Merchant": "bominal-mock-merchant",
      "X-SRT-Trace-Id": crypto.randomUUID(),
      "X-EV-Test-Shared-Secret": config.sharedSecret,
    };

    const outboundBody = {
      provider: "mock_srt",
      payment: {
        amount,
        currency,
        card: {
          number: encryptedCardNumber,
          expiry_month: encryptedExpiryMonth,
          expiry_year: encryptedExpiryYear,
          cvc: encryptedCvc,
        },
      },
      booking: {
        reference: bookingReference,
        route_code: String(req.body?.route_code || "0999"),
        departure_date: String(req.body?.departure_date || "2026-03-15"),
      },
      meta: {
        source: "evervault-test",
        sent_at: new Date().toISOString(),
      },
    };

    const destinationUrl = `https://${config.destinationDomain}${relayRuntime.srtListenerPath}`;
    const relayResult = await dispatchViaGoOutboundRelay({
      binaryPath: goRelaySenderBin,
      appId: config.evervaultAppId,
      apiKey: config.evervaultApiKey,
      destinationUrl,
      method: "POST",
      headers: outboundHeaders,
      body: outboundBody,
      timeoutMs: config.pollTimeoutSeconds * 1000,
    });

    let parsedResponseBody = relayResult.response_body;
    try {
      parsedResponseBody = JSON.parse(String(relayResult.response_body || ""));
    } catch {
      parsedResponseBody = relayResult.response_body;
    }

    const responsePayload = {
      ok: Boolean(relayResult.ok),
      flow: "go_sdk_outbound_relay_mock_srt",
      relay: {
        relay_id: relayRuntime.relayId,
        relay_domain: relayRuntime.relayDomain,
        destination_domain: relayRuntime.destinationDomain,
        listener_path: relayRuntime.srtListenerPath,
        go_outbound_sender_bin: goRelaySenderBin,
      },
      outbound_request: {
        url: destinationUrl,
        method: "POST",
        headers: outboundHeaders,
        body: outboundBody,
      },
      outbound_response: {
        status_code: Number(relayResult.status_code || 0),
        headers: relayResult.response_headers || {},
        body: parsedResponseBody,
      },
    };

    if (!relayResult.ok || Number(relayResult.status_code || 0) >= 400) {
      return res.status(502).json(responsePayload);
    }

    return res.json(responsePayload);
  } catch (error) {
    return res.status(502).json({
      ok: false,
      detail: toErrorMessage(error, "go sdk outbound relay flow failed"),
    });
  }
}

function resultHandler(req, res) {
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
}

function relayListenerHandler(req, res) {
  const sharedSecret = String(req.body?.shared_secret || "").trim();
  if (sharedSecret !== config.sharedSecret) {
    return res.status(401).json({ ok: false, detail: "listener shared_secret mismatch" });
  }

  const sessionId = String(req.body?.session_id || "").trim();
  const nonce = String(req.body?.session_nonce || "").trim();
  const decryptedPan = String(req.body?.encrypted_card_number || "").trim();
  const decryptedExpiryMonth = String(req.body?.encrypted_card_expiry_month || "").trim();
  const decryptedExpiryYear = String(req.body?.encrypted_card_expiry_year || "").trim();
  const decryptedCvc = String(req.body?.encrypted_card_cvc || "").trim();

  if (!sessionId || !nonce || !decryptedPan) {
    return res.status(400).json({ ok: false, detail: "missing session_id, session_nonce, or encrypted_card_number" });
  }

  const receipt = sessionStore.recordListenerReceipt({
    sessionId,
    nonce,
    decryptedPan,
    decryptedExpiryMonth,
    decryptedExpiryYear,
    decryptedCvc,
  });
  if (!receipt.ok) {
    return res.status(400).json({ ok: false, detail: receipt.error });
  }

  const result = sessionStore.getResult(sessionId);
  return res.json({
    ok: true,
    status: result.status,
    proof: result?.proof || null,
  });
}

function srtListenerHandler(req, res) {
  const sharedSecret = String(req.get("x-ev-test-shared-secret") || "").trim();
  if (sharedSecret !== config.sharedSecret) {
    return res.status(401).json({ ok: false, detail: "listener shared secret header mismatch" });
  }

  const seenHeaders = {
    authorization: String(req.get("authorization") || ""),
    x_srt_encrypted_pan: String(req.get("x-srt-encrypted-pan") || ""),
    x_srt_merchant: String(req.get("x-srt-merchant") || ""),
    x_srt_trace_id: String(req.get("x-srt-trace-id") || ""),
    content_type: String(req.get("content-type") || ""),
  };

  return res.json({
    ok: true,
    listener: "bominal_mock_srt_listener",
    received_at: new Date().toISOString(),
    received_headers: seenHeaders,
    received_body: req.body || {},
  });
}

app.get("/api/test/config", getConfigHandler);
app.get("/evervault-test/api/test/config", getConfigHandler);

app.post("/api/test/self-check", selfCheckHandler);
app.post("/evervault-test/api/test/self-check", selfCheckHandler);

app.post("/api/test/run", runHandler);
app.post("/evervault-test/api/test/run", runHandler);
app.post("/api/test/run-card", runCardHandler);
app.post("/evervault-test/api/test/run-card", runCardHandler);
app.post("/api/test/run-srt-go", runSrtGoHandler);
app.post("/evervault-test/api/test/run-srt-go", runSrtGoHandler);

app.get("/api/test/result/:sessionId", resultHandler);
app.get("/evervault-test/api/test/result/:sessionId", resultHandler);

// Support both prefix-preserving and prefix-stripping reverse proxies.
app.post("/evervault-test/relay-listener", relayListenerHandler);
app.post("/relay-listener", relayListenerHandler);
app.post("/evervault-test/relay-listener-card", relayListenerHandler);
app.post("/relay-listener-card", relayListenerHandler);
app.post("/evervault-test/srt-listener", srtListenerHandler);
app.post("/srt-listener", srtListenerHandler);

app.listen(config.port, "0.0.0.0", () => {
  // Intentionally avoid logging request payloads to protect decrypted values.
  console.log(`evervault-test listening on http://0.0.0.0:${config.port}`);
});
