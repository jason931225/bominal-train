const EVERVAULT_SCRIPT_URL = "https://js.evervault.com/v2";
const EVERVAULT_CIPHERTEXT_PREFIX = "ev:";

type EvervaultClient = {
  encrypt: (value: string) => Promise<string>;
};

type EvervaultClientConstructor = new (teamId: string, appId: string) => EvervaultClient;

export type EvervaultPaymentPlaintext = {
  card_number: string;
  pin2: string;
  birth_date: string;
  expiry: string;
  last4: string;
};

export type EvervaultPaymentEncrypted = {
  encrypted_card_number: string;
  encrypted_pin2: string;
  encrypted_birth_date: string;
  encrypted_expiry: string;
  last4: string;
  brand: string | null;
};

export type EvervaultBrowserCredentials = {
  teamId: string;
  appId: string;
};

type EvervaultBrowserEnv = {
  [key: string]: string | undefined;
  NEXT_PUBLIC_EVERVAULT_TEAM_ID?: string;
  NEXT_PUBLIC_EVERVAULT_APP_ID?: string;
};

declare global {
  interface Window {
    Evervault?: EvervaultClientConstructor;
  }
}

let evervaultScriptPromise: Promise<void> | null = null;

function inferCardBrand(cardNumber: string): string | null {
  const digits = cardNumber.replace(/\D/g, "");
  if (digits.startsWith("4")) return "visa";
  if (/^5[1-5]/.test(digits) || /^2[2-7]/.test(digits)) return "mastercard";
  if (/^3[47]/.test(digits)) return "amex";
  if (/^6(?:011|5|4[4-9])/.test(digits)) return "discover";
  return null;
}

function loadEvervaultScript(): Promise<void> {
  if (typeof window === "undefined") {
    return Promise.reject(new Error("Evervault encryption requires a browser runtime"));
  }
  if (window.Evervault) {
    return Promise.resolve();
  }
  if (evervaultScriptPromise) {
    return evervaultScriptPromise;
  }

  evervaultScriptPromise = new Promise<void>((resolve, reject) => {
    const existing = document.querySelector<HTMLScriptElement>(`script[src="${EVERVAULT_SCRIPT_URL}"]`);
    if (existing) {
      existing.addEventListener("load", () => resolve(), { once: true });
      existing.addEventListener("error", () => reject(new Error("Failed to load Evervault script")), { once: true });
      return;
    }

    const script = document.createElement("script");
    script.src = EVERVAULT_SCRIPT_URL;
    script.async = true;
    script.onload = () => resolve();
    script.onerror = () => reject(new Error("Failed to load Evervault script"));
    document.head.appendChild(script);
  }).finally(() => {
    if (!window.Evervault) {
      evervaultScriptPromise = null;
    }
  });

  return evervaultScriptPromise;
}

export function getEvervaultBrowserCredentialsFromEnv(env: EvervaultBrowserEnv = process.env): EvervaultBrowserCredentials | null {
  const teamId = String(env.NEXT_PUBLIC_EVERVAULT_TEAM_ID ?? "").trim();
  const appId = String(env.NEXT_PUBLIC_EVERVAULT_APP_ID ?? "").trim();
  if (!teamId || !appId) {
    return null;
  }
  return { teamId, appId };
}

export function isEvervaultCiphertext(value: string): boolean {
  return String(value).trim().startsWith(EVERVAULT_CIPHERTEXT_PREFIX);
}

export function assertEvervaultCiphertext(value: string, fieldName: string): string {
  const normalized = String(value ?? "").trim();
  if (!isEvervaultCiphertext(normalized)) {
    throw new Error(`Expected Evervault ciphertext for ${fieldName}`);
  }
  return normalized;
}

export function assertEvervaultEncryptedPaymentPayload(payload: EvervaultPaymentEncrypted): EvervaultPaymentEncrypted {
  if (!/^\d{4}$/.test(payload.last4)) {
    throw new Error("Expected 4-digit last4 in encrypted payment payload");
  }

  return {
    ...payload,
    encrypted_card_number: assertEvervaultCiphertext(payload.encrypted_card_number, "encrypted_card_number"),
    encrypted_pin2: assertEvervaultCiphertext(payload.encrypted_pin2, "encrypted_pin2"),
    encrypted_birth_date: assertEvervaultCiphertext(payload.encrypted_birth_date, "encrypted_birth_date"),
    encrypted_expiry: assertEvervaultCiphertext(payload.encrypted_expiry, "encrypted_expiry"),
  };
}

export async function encryptPaymentFields(
  payload: EvervaultPaymentPlaintext,
  creds: EvervaultBrowserCredentials,
): Promise<EvervaultPaymentEncrypted> {
  const teamId = String(creds.teamId || "").trim();
  const appId = String(creds.appId || "").trim();
  if (!teamId || !appId) {
    throw new Error("Evervault browser credentials are missing");
  }

  await loadEvervaultScript();
  if (!window.Evervault) {
    throw new Error("Evervault SDK is unavailable");
  }

  const client = new window.Evervault(teamId, appId);
  const [encryptedCardNumber, encryptedPin2, encryptedBirthDate, encryptedExpiry] = await Promise.all([
    client.encrypt(payload.card_number),
    client.encrypt(payload.pin2),
    client.encrypt(payload.birth_date),
    client.encrypt(payload.expiry),
  ]);

  const encryptedPayload: EvervaultPaymentEncrypted = {
    encrypted_card_number: encryptedCardNumber,
    encrypted_pin2: encryptedPin2,
    encrypted_birth_date: encryptedBirthDate,
    encrypted_expiry: encryptedExpiry,
    last4: payload.last4,
    brand: inferCardBrand(payload.card_number),
  };

  return assertEvervaultEncryptedPaymentPayload(encryptedPayload);
}
