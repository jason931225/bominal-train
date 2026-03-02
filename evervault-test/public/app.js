const statusLine = document.getElementById("status-line");
const resultJson = document.getElementById("result-json");
const mockPanInput = document.getElementById("mock-pan");
const selfCheckBtn = document.getElementById("self-check-btn");
const runTestBtn = document.getElementById("run-test-btn");

let cachedConfig = null;
let evervaultScriptPromise = null;
const API_BASE = "/evervault-test/api/test";

function setStatus(message) {
  statusLine.textContent = message;
}

function setResult(value) {
  resultJson.textContent = JSON.stringify(value, null, 2);
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

async function encryptMockPan({ cardNumber, teamId, appId }) {
  await loadEvervaultScript();
  if (!window.Evervault) {
    throw new Error("Evervault SDK unavailable");
  }

  const client = new window.Evervault(teamId, appId);
  return client.encrypt(cardNumber);
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
  selfCheckBtn.disabled = true;
  runTestBtn.disabled = true;
  setStatus("Running self-check...");

  try {
    const payload = await fetchJson(`${API_BASE}/self-check`, { method: "POST", body: JSON.stringify({}) });
    setResult(payload);
    setStatus("Self-check passed");
  } catch (error) {
    setResult({ ok: false, detail: String(error.message || error) });
    setStatus("Self-check failed");
  } finally {
    selfCheckBtn.disabled = false;
    runTestBtn.disabled = false;
  }
});

runTestBtn.addEventListener("click", async () => {
  runTestBtn.disabled = true;
  selfCheckBtn.disabled = true;
  setStatus("Preparing encryption test...");

  try {
    const config = await getConfig();
    const cardNumber = digitsOnly(mockPanInput.value);

    if (cardNumber.length < 12 || cardNumber.length > 19) {
      throw new Error("Enter a mock card number between 12 and 19 digits");
    }

    if (!config.evervault_team_id || !config.evervault_app_id) {
      throw new Error("Browser Evervault credentials are missing on server config endpoint");
    }

    setStatus("Encrypting card number in browser...");
    const encrypted = await encryptMockPan({
      cardNumber,
      teamId: config.evervault_team_id,
      appId: config.evervault_app_id,
    });

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
    runTestBtn.disabled = false;
    selfCheckBtn.disabled = false;
  }
});
