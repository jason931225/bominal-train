export type PasskeyRegistrationOptionsResponse = {
  challenge_id: string;
  public_key: Record<string, unknown>;
};

export type PasskeyAuthenticationOptionsResponse = {
  challenge_id: string;
  public_key: Record<string, unknown>;
};

export type PasskeyOperationResult = {
  ok: boolean;
  error?: string;
};

function parseClientWebAuthnError(error: unknown, fallback: string): string {
  const domExceptionMessages: Record<string, string> = {
    NotAllowedError: "Passkey operation was cancelled or timed out.",
    InvalidStateError: "This passkey is already registered on this device for this account.",
    NotSupportedError: "Passkeys are not supported on this device/browser.",
    AbortError: "Passkey operation was interrupted.",
  };
  if (error instanceof DOMException) {
    if (error.name === "SecurityError") {
      const detail = error.message.trim();
      if (detail) {
        return `Passkey security check failed: ${detail}. Use http://localhost:3000 for local development.`;
      }
      return "Passkey security check failed. Use http://localhost:3000 for local development.";
    }
    return domExceptionMessages[error.name] ?? (error.message || fallback);
  }
  if (error instanceof Error) {
    return error.message || fallback;
  }
  return fallback;
}

function normalizeBase64Url(value: string): string {
  const padded = value.replace(/-/g, "+").replace(/_/g, "/");
  return padded + "=".repeat((4 - (padded.length % 4 || 4)) % 4);
}

function base64UrlToUint8Array(value: string): Uint8Array {
  const decoded = atob(normalizeBase64Url(value));
  const bytes = new Uint8Array(decoded.length);
  for (let i = 0; i < decoded.length; i += 1) {
    bytes[i] = decoded.charCodeAt(i);
  }
  return bytes;
}

function arrayBufferToBase64Url(value: ArrayBuffer | ArrayBufferView): string {
  const bytes = value instanceof ArrayBuffer ? new Uint8Array(value) : new Uint8Array(value.buffer);
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

function normalizeCreationOptions(publicKey: Record<string, unknown>): PublicKeyCredentialCreationOptions {
  const user = (publicKey.user ?? {}) as Record<string, unknown>;
  const excludeCredentials = Array.isArray(publicKey.excludeCredentials) ? publicKey.excludeCredentials : [];
  const normalized = {
    ...(publicKey as unknown as PublicKeyCredentialCreationOptions),
    challenge: base64UrlToUint8Array(String(publicKey.challenge ?? "")) as unknown as BufferSource,
    user: {
      ...(user as unknown as PublicKeyCredentialUserEntity),
      id: base64UrlToUint8Array(String(user.id ?? "")) as unknown as BufferSource,
    },
    excludeCredentials: excludeCredentials.map((item) => {
      const parsed = item as Record<string, unknown>;
      return {
        ...(parsed as unknown as PublicKeyCredentialDescriptor),
        id: base64UrlToUint8Array(String(parsed.id ?? "")) as unknown as BufferSource,
      } satisfies PublicKeyCredentialDescriptor;
    }),
  };
  return normalized as PublicKeyCredentialCreationOptions;
}

function normalizeRequestOptions(publicKey: Record<string, unknown>): PublicKeyCredentialRequestOptions {
  const allowCredentials = Array.isArray(publicKey.allowCredentials) ? publicKey.allowCredentials : [];
  const normalized = {
    ...(publicKey as unknown as PublicKeyCredentialRequestOptions),
    challenge: base64UrlToUint8Array(String(publicKey.challenge ?? "")) as unknown as BufferSource,
    allowCredentials: allowCredentials.map((item) => {
      const parsed = item as Record<string, unknown>;
      return {
        ...(parsed as unknown as PublicKeyCredentialDescriptor),
        id: base64UrlToUint8Array(String(parsed.id ?? "")) as unknown as BufferSource,
      } satisfies PublicKeyCredentialDescriptor;
    }),
  };
  return normalized as PublicKeyCredentialRequestOptions;
}

function serializeRegistrationCredential(credential: PublicKeyCredential): Record<string, unknown> {
  const response = credential.response as AuthenticatorAttestationResponse;
  const getTransports = (response as AuthenticatorAttestationResponse & { getTransports?: () => string[] }).getTransports;
  return {
    id: credential.id,
    rawId: arrayBufferToBase64Url(credential.rawId),
    type: credential.type,
    response: {
      attestationObject: arrayBufferToBase64Url(response.attestationObject),
      clientDataJSON: arrayBufferToBase64Url(response.clientDataJSON),
      transports: typeof getTransports === "function" ? getTransports.call(response) : [],
    },
    clientExtensionResults: credential.getClientExtensionResults(),
  };
}

function serializeAuthenticationCredential(credential: PublicKeyCredential): Record<string, unknown> {
  const response = credential.response as AuthenticatorAssertionResponse;
  return {
    id: credential.id,
    rawId: arrayBufferToBase64Url(credential.rawId),
    type: credential.type,
    response: {
      authenticatorData: arrayBufferToBase64Url(response.authenticatorData),
      clientDataJSON: arrayBufferToBase64Url(response.clientDataJSON),
      signature: arrayBufferToBase64Url(response.signature),
      userHandle: response.userHandle ? arrayBufferToBase64Url(response.userHandle) : null,
    },
    clientExtensionResults: credential.getClientExtensionResults(),
  };
}

async function parseApiError(response: Response, fallback: string): Promise<string> {
  const contentType = response.headers.get("content-type") ?? "";
  if (contentType.includes("application/json")) {
    const payload = (await response.json().catch(() => null)) as { detail?: string } | null;
    if (payload?.detail) return payload.detail;
  }
  const text = await response.text().catch(() => "");
  return text.trim() || fallback;
}

export function isPasskeySupported(): boolean {
  return typeof window !== "undefined" && typeof PublicKeyCredential !== "undefined" && !!navigator.credentials;
}

export async function registerPasskeyFromSession(apiBaseUrl: string): Promise<PasskeyOperationResult> {
  if (!isPasskeySupported()) {
    return { ok: false, error: "Passkeys are not supported on this device." };
  }

  const optionsResponse = await fetch(`${apiBaseUrl}/api/auth/passkeys/register/options`, {
    method: "POST",
    credentials: "include",
  });
  if (!optionsResponse.ok) {
    return { ok: false, error: await parseApiError(optionsResponse, "Could not start passkey setup.") };
  }

  const optionsBody = (await optionsResponse.json()) as PasskeyRegistrationOptionsResponse;
  let creation: Credential | null = null;
  try {
    creation = await navigator.credentials.create({
      publicKey: normalizeCreationOptions(optionsBody.public_key),
    });
  } catch (error: unknown) {
    return {
      ok: false,
      error: parseClientWebAuthnError(error, "Could not start passkey setup."),
    };
  }
  if (!creation) {
    return { ok: false, error: "Passkey setup was cancelled." };
  }

  const verifyResponse = await fetch(`${apiBaseUrl}/api/auth/passkeys/register/verify`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      challenge_id: optionsBody.challenge_id,
      credential: serializeRegistrationCredential(creation as PublicKeyCredential),
    }),
  });
  if (!verifyResponse.ok) {
    return { ok: false, error: await parseApiError(verifyResponse, "Could not finish passkey setup.") };
  }
  return { ok: true };
}

export async function signInWithPasskey(
  apiBaseUrl: string,
  params: { email: string; rememberMe: boolean },
): Promise<PasskeyOperationResult> {
  const { email, rememberMe } = params;
  if (!isPasskeySupported()) {
    return { ok: false, error: "Passkeys are not supported on this device." };
  }

  const optionsResponse = await fetch(`${apiBaseUrl}/api/auth/passkeys/auth/options`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    credentials: "include",
    body: JSON.stringify({ email }),
  });
  if (!optionsResponse.ok) {
    return { ok: false, error: await parseApiError(optionsResponse, "Could not start passkey sign in.") };
  }

  const optionsBody = (await optionsResponse.json()) as PasskeyAuthenticationOptionsResponse;
  let assertion: Credential | null = null;
  try {
    assertion = await navigator.credentials.get({
      publicKey: normalizeRequestOptions(optionsBody.public_key),
    });
  } catch (error: unknown) {
    return {
      ok: false,
      error: parseClientWebAuthnError(error, "Could not start passkey sign in."),
    };
  }
  if (!assertion) {
    return { ok: false, error: "Passkey sign in was cancelled." };
  }

  const verifyResponse = await fetch(`${apiBaseUrl}/api/auth/passkeys/auth/verify`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    credentials: "include",
    body: JSON.stringify({
      email,
      remember_me: rememberMe,
      challenge_id: optionsBody.challenge_id,
      credential: serializeAuthenticationCredential(assertion as PublicKeyCredential),
    }),
  });
  if (!verifyResponse.ok) {
    return { ok: false, error: await parseApiError(verifyResponse, "Passkey sign in failed.") };
  }

  return { ok: true };
}

export async function listPasskeysFromSession(
  apiBaseUrl: string,
): Promise<{ credentials: Array<{ id: string; created_at: string; last_used_at: string | null }> }> {
  const response = await fetch(`${apiBaseUrl}/api/auth/passkeys`, {
    method: "GET",
    credentials: "include",
  });
  if (!response.ok) {
    throw new Error(await parseApiError(response, "Could not load passkeys."));
  }
  return (await response.json()) as { credentials: Array<{ id: string; created_at: string; last_used_at: string | null }> };
}

export async function removePasskeyFromSession(apiBaseUrl: string, passkeyId: string): Promise<void> {
  const response = await fetch(`${apiBaseUrl}/api/auth/passkeys/${encodeURIComponent(passkeyId)}`, {
    method: "DELETE",
    credentials: "include",
  });
  if (!response.ok) {
    throw new Error(await parseApiError(response, "Could not remove passkey."));
  }
}
