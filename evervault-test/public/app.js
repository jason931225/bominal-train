const statusLine = document.getElementById("status-line");
const resultJson = document.getElementById("result-json");
const mockPanInput = document.getElementById("mock-pan");
const cardStateLine = document.getElementById("ev-card-state");
const selfCheckBtn = document.getElementById("self-check-btn");
const runTestBtn = document.getElementById("run-test-btn");
const runCardTestBtn = document.getElementById("run-card-test-btn");

let cachedConfig = null;
let evervaultScriptPromise = null;
let evervaultClient = null;
let uiCard = null;
let uiCardLatest = null;
const API_BASE = "/evervault-test/api/test";

function setStatus(message) {
  statusLine.textContent = message;
}

function setResult(value) {
  resultJson.textContent = JSON.stringify(value, null, 2);
}

function setCardState(message) {
  if (cardStateLine) {
    cardStateLine.textContent = message;
  }
}

function digitsOnly(value) {
  return String(value || "").replace(/\D/g, "");
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function fetchJson(url, options = {}) {
  const response = await fetch(url, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...(options.headers || {}),
    },
  });
  const payload = await response.json().catch(() => ({ detail: "Invalid JSON response" }));
  if (!response.ok) {
    const detail = String(payload?.detail || `Request failed with status ${response.status}`);
    throw new Error(detail);
  }
  return payload;
}

function setBusy(isBusy) {
  selfCheckBtn.disabled = isBusy;
  runTestBtn.disabled = isBusy;
  runCardTestBtn.disabled = isBusy;
}

async function getConfig() {
  if (cachedConfig) {
    return cachedConfig;
  }
  cachedConfig = await fetchJson(`${API_BASE}/config`, { method: "GET", headers: {} });
  return cachedConfig;
}

function loadEvervaultScript() {
  if (window.Evervault) {
    return Promise.resolve();
  }

  if (evervaultScriptPromise) {
    return evervaultScriptPromise;
  }

  evervaultScriptPromise = new Promise((resolve, reject) => {
    const src = "https://js.evervault.com/v2";
    const existing = document.querySelector(`script[src=\"${src}\"]`);

    if (existing) {
      existing.addEventListener("load", () => resolve(), { once: true });
      existing.addEventListener("error", () => reject(new Error("Failed to load Evervault SDK")), { once: true });
      return;
    }

    const script = document.createElement("script");
    script.src = src;
    script.async = true;
    script.onload = () => resolve();
    script.onerror = () => reject(new Error("Failed to load Evervault SDK"));
    document.head.appendChild(script);
  });

  return evervaultScriptPromise;
}

async function getEvervaultClient() {
  if (evervaultClient) return evervaultClient;

  const config = await getConfig();
  await loadEvervaultScript();
  if (!window.Evervault) throw new Error("Evervault SDK unavailable");
  if (!config.evervault_team_id || !config.evervault_app_id) {
    throw new Error("Browser Evervault credentials are missing on server config endpoint");
  }

  evervaultClient = new window.Evervault(config.evervault_team_id, config.evervault_app_id);
  return evervaultClient;
}

async function encryptMockPan(cardNumber) {
  const client = await getEvervaultClient();
  return client.encrypt(cardNumber);
}

function readUiCardEncryptedValues() {
  const sourceCard = uiCard?.values?.card || uiCardLatest?.card || {};
  const expiry = sourceCard.expiry || {};
  const last4 = digitsOnly(sourceCard.last4 || sourceCard.lastFour || sourceCard.last_four).slice(-4);
  return {
    encrypted_card_number: String(sourceCard.number || "").trim(),
    encrypted_card_expiry_month: String(expiry.month || "").trim(),
    encrypted_card_expiry_year: String(expiry.year || "").trim(),
    encrypted_card_cvc: String(sourceCard.cvc || "").trim(),
    expected_last4: last4,
  };
}

async function initUiCard() {
  if (uiCard) return uiCard;

  const client = await getEvervaultClient();
  const createCard =
    client?.ui && typeof client.ui.card === "function"
      ? (options) => client.ui.card(options)
      : typeof client.card === "function"
        ? (options) => client.card(options)
        : null;

  if (!createCard) {
    throw new Error("Evervault UI Card API unavailable in loaded SDK");
  }

  const theme = client?.ui?.themes && typeof client.ui.themes.clean === "function" ? client.ui.themes.clean() : undefined;
  uiCard = createCard(theme ? { theme } : {});

  if (typeof uiCard.on === "function") {
    uiCard.on("ready", () => {
      setCardState("UI Card ready");
    });
    uiCard.on("change", (payload) => {
      uiCardLatest = payload;
      const last4 = digitsOnly(payload?.card?.last4 || payload?.card?.lastFour || "").slice(-4);
      setCardState(last4 ? `UI Card updated (last4: ${last4})` : "UI Card updated");
    });
    uiCard.on("error", (error) => {
      setCardState(`UI Card error: ${String(error?.message || error)}`);
    });
  }

  if (typeof uiCard.mount !== "function") {
    throw new Error("Evervault UI Card mount API is unavailable");
  }

  uiCard.mount("#ev-card-fields");
  setCardState("UI Card mounted");
  return uiCard;
}

async function pollResult(sessionId, timeoutSeconds) {
  const timeoutAt = Date.now() + timeoutSeconds * 1000;

  while (Date.now() < timeoutAt) {
    const result = await fetchJson(`${API_BASE}/result/${encodeURIComponent(sessionId)}`, { method: "GET", headers: {} });

    if (["received", "failed", "expired"].includes(result.status)) {
      return result;
    }

    await sleep(1000);
  }

  throw new Error("Timed out while waiting for relay listener confirmation");
}

selfCheckBtn.addEventListener("click", async () => {
  setBusy(true);
  setStatus("Running self-check...");

  try {
    const payload = await fetchJson(`${API_BASE}/self-check`, { method: "POST", body: JSON.stringify({}) });
    setResult(payload);
    setStatus("Self-check passed");
  } catch (error) {
    setResult({ ok: false, detail: String(error.message || error) });
    setStatus("Self-check failed");
  } finally {
    setBusy(false);
  }
});

runTestBtn.addEventListener("click", async () => {
  setBusy(true);
  setStatus("Preparing encryption test...");

  try {
    const config = await getConfig();
    const cardNumber = digitsOnly(mockPanInput.value);

    if (cardNumber.length < 12 || cardNumber.length > 19) {
      throw new Error("Enter a mock card number between 12 and 19 digits");
    }

    setStatus("Encrypting card number in browser...");
    const encrypted = await encryptMockPan(cardNumber);

    setResult({
      stage: "browser_encrypted",
      encrypted_card_number: encrypted,
    });

    setStatus("Dispatching to backend and relay...");
    const runPayload = await fetchJson(`${API_BASE}/run`, {
      method: "POST",
      body: JSON.stringify({
        encrypted_card_number: encrypted,
        expected_last4: cardNumber.slice(-4),
      }),
    });

    setResult(runPayload);
    setStatus("Polling result...");

    const final = await pollResult(runPayload.session_id, Number(config.poll_timeout_seconds) || 20);
    setResult(final);

    if (final.status === "received") {
      setStatus("Relay decryption confirmed by listener");
    } else {
      setStatus(`Test ended with status: ${final.status}`);
    }
  } catch (error) {
    setResult({ ok: false, detail: String(error.message || error) });
    setStatus("Test failed");
  } finally {
    setBusy(false);
  }
});

runCardTestBtn.addEventListener("click", async () => {
  setBusy(true);
  setStatus("Preparing UI Card relay test...");

  try {
    const config = await getConfig();
    await initUiCard();

    if (typeof uiCard?.validate === "function") {
      try {
        uiCard.validate();
      } catch {
        // no-op: some SDK versions throw when called before completion
      }
    }

    const encryptedCard = readUiCardEncryptedValues();
    if (!encryptedCard.encrypted_card_number || !encryptedCard.encrypted_card_number.startsWith("ev:")) {
      throw new Error("Fill UI Card with valid card details so encrypted values are available");
    }

    setResult({
      stage: "browser_ui_card_encrypted",
      encrypted_card: encryptedCard,
    });

    setStatus("Dispatching UI Card payload to backend and relay...");
    const runPayload = await fetchJson(`${API_BASE}/run-card`, {
      method: "POST",
      body: JSON.stringify(encryptedCard),
    });

    setResult(runPayload);
    setStatus("Polling UI Card result...");

    const final = await pollResult(runPayload.session_id, Number(config.poll_timeout_seconds) || 20);
    setResult(final);

    if (final.status === "received") {
      setStatus("UI Card relay decryption confirmed by listener");
    } else {
      setStatus(`UI Card test ended with status: ${final.status}`);
    }
  } catch (error) {
    setResult({ ok: false, detail: String(error.message || error) });
    setStatus("UI Card test failed");
  } finally {
    setBusy(false);
  }
});

initUiCard().catch((error) => {
  setCardState(`UI Card init failed: ${String(error.message || error)}`);
});
