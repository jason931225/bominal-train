// Bominal — Evervault + WebAuthn interop
// Compiled: npx esbuild ts/interop.ts --bundle --outfile=ts/interop.js --format=iife --global-name=BominalInterop

declare const Evervault: any;

let evInstance: any = null;

function getEvervault(): any {
  if (evInstance) return evInstance;
  const teamId = document.querySelector('meta[name="ev-team-id"]')?.getAttribute('content');
  const appId = document.querySelector('meta[name="ev-app-id"]')?.getAttribute('content');
  if (!teamId || !appId) throw new Error('Evervault meta tags missing');
  evInstance = new Evervault(teamId, appId);
  return evInstance;
}

// Called from card form submit handler
(window as any).__evEncrypt = async function(plaintext: string): Promise<string> {
  const ev = getEvervault();
  return await ev.encrypt(plaintext);
};

// Called from card form — encrypts all card fields and POSTs to /api/cards
(window as any).__submitCard = async function(
  label: string,
  cardNumber: string,
  cardPassword: string,
  birthday: string,
  expireMmyy: string,
  cardType: string,
): Promise<{ ok: boolean; error?: string; card?: any }> {
  try {
    const ev = getEvervault();
    const lastFour = cardNumber.slice(-4);

    // Encrypt all sensitive fields
    const [encNumber, encPassword, encBirthday, encExpiry, encExpiryYymm] = await Promise.all([
      ev.encrypt(cardNumber),
      ev.encrypt(cardPassword),
      ev.encrypt(birthday),
      ev.encrypt(expireMmyy),
      ev.encrypt(expireMmyy.slice(2) + expireMmyy.slice(0, 2)), // MMYY -> YYMM
    ]);

    const resp = await fetch('/api/cards', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'include',
      body: JSON.stringify({
        label: label || 'My Card',
        card_number: encNumber,
        card_password: encPassword,
        birthday: encBirthday,
        expire_date: encExpiry,
        expire_date_yymm: encExpiryYymm,
        last_four: lastFour,
        card_type: cardType || 'J',
      }),
    });

    if (!resp.ok) {
      const body = await resp.json().catch(() => ({ message: 'Request failed' }));
      return { ok: false, error: body.message || `HTTP ${resp.status}` };
    }

    const card = await resp.json();
    return { ok: true, card };
  } catch (e: any) {
    return { ok: false, error: e.message || 'Encryption failed' };
  }
};

// ── Helpers ──────────────────────────────────────────────────────

function toBase64url(buffer: ArrayBuffer): string {
  return btoa(String.fromCharCode(...new Uint8Array(buffer)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
}

// ── View Transitions ─────────────────────────────────────────────
// Wraps a navigation callback in document.startViewTransition() when available.

(window as any).__startViewTransition = function(cb: () => void): void {
  if ('startViewTransition' in document) {
    (document as any).startViewTransition(cb);
  } else {
    cb();
  }
};

// ── WebAuthn — Passkey Registration ──────────────────────────────

(window as any).__startPasskeyRegistration = async function(optionsJson: string): Promise<string> {
  const options = JSON.parse(optionsJson);
  // Convert challenge from base64url
  options.challenge = Uint8Array.from(
    atob(options.challenge.replace(/-/g, '+').replace(/_/g, '/')),
    c => c.charCodeAt(0),
  );
  if (options.user?.id) {
    // Decode base64url user.id back to raw UUID bytes (16 bytes)
    const b64 = options.user.id.replace(/-/g, '+').replace(/_/g, '/');
    options.user.id = Uint8Array.from(atob(b64), c => c.charCodeAt(0));
  }
  if (options.excludeCredentials) {
    options.excludeCredentials = options.excludeCredentials.map((c: any) => ({
      ...c,
      id: Uint8Array.from(
        atob(c.id.replace(/-/g, '+').replace(/_/g, '/')),
        ch => ch.charCodeAt(0),
      ),
    }));
  }
  const credential = await navigator.credentials.create({ publicKey: options }) as PublicKeyCredential;
  const response = credential.response as AuthenticatorAttestationResponse;
  return JSON.stringify({
    id: credential.id,
    rawId: toBase64url(credential.rawId),
    type: credential.type,
    response: {
      attestationObject: toBase64url(response.attestationObject),
      clientDataJSON: toBase64url(response.clientDataJSON),
    },
  });
};

// ── WebAuthn — Passkey Login ─────────────────────────────────────

(window as any).__startPasskeyLogin = async function(optionsJson: string): Promise<string> {
  const options = JSON.parse(optionsJson);
  options.challenge = Uint8Array.from(
    atob(options.challenge.replace(/-/g, '+').replace(/_/g, '/')),
    c => c.charCodeAt(0),
  );
  if (options.allowCredentials) {
    options.allowCredentials = options.allowCredentials.map((c: any) => ({
      ...c,
      id: Uint8Array.from(
        atob(c.id.replace(/-/g, '+').replace(/_/g, '/')),
        ch => ch.charCodeAt(0),
      ),
    }));
  }
  const assertion = await navigator.credentials.get({ publicKey: options }) as PublicKeyCredential;
  const response = assertion.response as AuthenticatorAssertionResponse;
  return JSON.stringify({
    id: assertion.id,
    rawId: toBase64url(assertion.rawId),
    type: assertion.type,
    response: {
      authenticatorData: toBase64url(response.authenticatorData),
      clientDataJSON: toBase64url(response.clientDataJSON),
      signature: toBase64url(response.signature),
      userHandle: response.userHandle
        ? toBase64url(response.userHandle)
        : null,
    },
  });
};
