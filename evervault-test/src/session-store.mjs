import crypto from "node:crypto";

function maskPan(value) {
  const digits = String(value || "").replace(/\D/g, "");
  if (digits.length < 4) {
    return "****";
  }
  return `**** **** **** ${digits.slice(-4)}`;
}

function last4(value) {
  const digits = String(value || "").replace(/\D/g, "");
  return digits.slice(-4);
}

function toIso(timestampMs) {
  return new Date(timestampMs).toISOString();
}

function normalizeError(message) {
  const text = String(message || "Unknown error").trim();
  return text || "Unknown error";
}

export class SessionStore {
  constructor({ ttlSeconds = 120, now = () => Date.now(), randomUUID, randomNonce } = {}) {
    this.ttlSeconds = Math.max(1, Number(ttlSeconds) || 120);
    this.now = now;
    this.randomUUID = randomUUID || (() => crypto.randomUUID());
    this.randomNonce = randomNonce || (() => crypto.randomBytes(16).toString("hex"));
    this.sessions = new Map();
  }

  createSession({ expectedLast4, revealFullOnce }) {
    const createdAtMs = this.now();
    const id = this.randomUUID();
    const nonce = this.randomNonce();

    const session = {
      id,
      nonce,
      created_at: toIso(createdAtMs),
      expires_at: toIso(createdAtMs + this.ttlSeconds * 1000),
      expires_at_ms: createdAtMs + this.ttlSeconds * 1000,
      expected_last4: String(expectedLast4 || ""),
      reveal_full_once: Boolean(revealFullOnce),
      full_pan_consumed: false,
      status: "pending",
      error: null,
      relay: null,
      proof: null,
    };

    this.sessions.set(id, session);
    return { id: session.id, nonce: session.nonce, expires_at: session.expires_at };
  }

  recordRelayDispatch(
    sessionId,
    {
      relayId,
      relayDomain,
      destinationDomain,
      listenerPath,
      dispatchStatusCode,
      dispatchResponseSnippet,
    } = {},
  ) {
    const session = this.sessions.get(sessionId);
    if (!session) {
      return false;
    }

    session.relay = {
      relay_id: String(relayId || ""),
      relay_domain: String(relayDomain || ""),
      destination_domain: String(destinationDomain || ""),
      listener_path: String(listenerPath || ""),
      dispatch_status_code: Number(dispatchStatusCode) || 0,
      dispatch_response_snippet: String(dispatchResponseSnippet || ""),
    };

    return true;
  }

  recordFailure(sessionId, message) {
    const session = this.sessions.get(sessionId);
    if (!session) {
      return false;
    }

    session.status = "failed";
    session.error = normalizeError(message);
    return true;
  }

  recordListenerReceipt({ sessionId, nonce, decryptedPan }) {
    const session = this.sessions.get(sessionId);
    if (!session) {
      return { ok: false, error: "Session not found" };
    }

    const expiredResult = this.#expireIfNeeded(session);
    if (expiredResult.status === "expired") {
      return { ok: false, error: "Session expired" };
    }

    if (String(nonce || "") !== session.nonce) {
      return { ok: false, error: "Session nonce mismatch" };
    }

    const digits = String(decryptedPan || "").replace(/\D/g, "");
    if (digits.length < 12 || digits.length > 19) {
      return { ok: false, error: "Decrypted card number format is invalid" };
    }

    const cardLast4 = last4(digits);
    session.status = "received";
    session.error = null;
    session.proof = {
      masked_pan: maskPan(digits),
      last4: cardLast4,
      matched_expected_last4: cardLast4 === session.expected_last4,
      received_at: toIso(this.now()),
      decrypted_pan: digits,
    };

    return { ok: true };
  }

  getResult(sessionId) {
    const session = this.sessions.get(sessionId);
    if (!session) {
      return {
        status: "not_found",
        session_id: sessionId,
        error: "Session not found",
      };
    }

    const expiredResult = this.#expireIfNeeded(session);
    if (expiredResult.status === "expired") {
      return this.#formatExpired(session);
    }

    if (session.status === "pending") {
      return this.#formatPending(session);
    }

    if (session.status === "failed") {
      return this.#formatFailed(session);
    }

    if (session.status === "received") {
      return this.#formatReceived(session);
    }

    return {
      status: "failed",
      session_id: session.id,
      error: "Unknown session state",
    };
  }

  cleanupExpired() {
    const nowMs = this.now();
    for (const [id, session] of this.sessions.entries()) {
      if (session.expires_at_ms <= nowMs) {
        this.sessions.delete(id);
      }
    }
  }

  #expireIfNeeded(session) {
    if (session.status === "expired") {
      return { status: "expired" };
    }

    if (this.now() > session.expires_at_ms) {
      session.status = "expired";
      if (!session.error) {
        session.error = "Session expired before listener confirmation";
      }
      return { status: "expired" };
    }

    return { status: session.status };
  }

  #formatPending(session) {
    return {
      status: "pending",
      session_id: session.id,
      created_at: session.created_at,
      expires_at: session.expires_at,
      relay: session.relay,
      error: null,
    };
  }

  #formatFailed(session) {
    return {
      status: "failed",
      session_id: session.id,
      created_at: session.created_at,
      expires_at: session.expires_at,
      relay: session.relay,
      error: session.error || "Unknown failure",
    };
  }

  #formatExpired(session) {
    return {
      status: "expired",
      session_id: session.id,
      created_at: session.created_at,
      expires_at: session.expires_at,
      relay: session.relay,
      error: session.error || "Session expired",
    };
  }

  #formatReceived(session) {
    const proof = {
      masked_pan: session.proof.masked_pan,
      last4: session.proof.last4,
      matched_expected_last4: session.proof.matched_expected_last4,
      received_at: session.proof.received_at,
    };

    if (session.reveal_full_once && !session.full_pan_consumed) {
      proof.full_pan_once = session.proof.decrypted_pan;
      session.full_pan_consumed = true;
    }

    return {
      status: "received",
      session_id: session.id,
      created_at: session.created_at,
      expires_at: session.expires_at,
      relay: session.relay,
      proof,
    };
  }
}
