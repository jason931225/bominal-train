(function () {
  let evInstance = null;

  function fromBase64Url(value) {
    const padded = value.replace(/-/g, "+").replace(/_/g, "/");
    const normalized = padded + "=".repeat((4 - (padded.length % 4)) % 4);
    return Uint8Array.from(atob(normalized), (char) => char.charCodeAt(0));
  }

  function toBase64Url(buffer) {
    return btoa(String.fromCharCode(...new Uint8Array(buffer)))
      .replace(/\+/g, "-")
      .replace(/\//g, "_")
      .replace(/=+$/, "");
  }

  function parseRegistrationOptions(optionsJson) {
    const options = JSON.parse(optionsJson);
    options.challenge = fromBase64Url(options.challenge);

    if (options.user && options.user.id) {
      options.user.id = fromBase64Url(options.user.id);
    }

    if (Array.isArray(options.excludeCredentials)) {
      options.excludeCredentials = options.excludeCredentials.map((credential) => ({
        ...credential,
        id: fromBase64Url(credential.id),
      }));
    }

    return options;
  }

  function parseLoginOptions(optionsJson) {
    const options = JSON.parse(optionsJson);
    options.challenge = fromBase64Url(options.challenge);

    if (Array.isArray(options.allowCredentials)) {
      options.allowCredentials = options.allowCredentials.map((credential) => ({
        ...credential,
        id: fromBase64Url(credential.id),
      }));
    }

    return options;
  }

  function serializeRegistration(credential) {
    const response = credential.response;

    return JSON.stringify({
      id: credential.id,
      rawId: toBase64Url(credential.rawId),
      type: credential.type,
      response: {
        attestationObject: toBase64Url(response.attestationObject),
        clientDataJSON: toBase64Url(response.clientDataJSON),
      },
    });
  }

  function serializeAssertion(assertion) {
    const response = assertion.response;

    return JSON.stringify({
      id: assertion.id,
      rawId: toBase64Url(assertion.rawId),
      type: assertion.type,
      response: {
        authenticatorData: toBase64Url(response.authenticatorData),
        clientDataJSON: toBase64Url(response.clientDataJSON),
        signature: toBase64Url(response.signature),
        userHandle: response.userHandle ? toBase64Url(response.userHandle) : null,
      },
    });
  }

  function getEvervault() {
    if (evInstance) return evInstance;

    const teamId = document
      .querySelector('meta[name="ev-team-id"]')
      ?.getAttribute("content");
    const appId = document
      .querySelector('meta[name="ev-app-id"]')
      ?.getAttribute("content");

    if (!teamId || !appId) {
      throw new Error("Evervault meta tags missing");
    }
    if (typeof Evervault === "undefined") {
      throw new Error("Evervault SDK is unavailable");
    }

    evInstance = new Evervault(teamId, appId);
    return evInstance;
  }

  window.__evEncrypt = async function (plaintext) {
    return await getEvervault().encrypt(plaintext);
  };

  window.__startPasskeyRegistration = async function (optionsJson) {
    const options = parseRegistrationOptions(optionsJson);
    const credential = await navigator.credentials.create({ publicKey: options });
    return serializeRegistration(credential);
  };

  window.__startPasskeyLogin = async function (optionsJson) {
    const options = parseLoginOptions(optionsJson);
    const assertion = await navigator.credentials.get({ publicKey: options });
    return serializeAssertion(assertion);
  };

  window.__startConditionalPasskeyLogin = async function (optionsJson) {
    const options = parseLoginOptions(optionsJson);
    const assertion = await navigator.credentials.get({
      publicKey: options,
      mediation: "conditional",
    });
    return serializeAssertion(assertion);
  };

  window.__submitCard = async function (
    label,
    cardNumber,
    cardPassword,
    birthday,
    expireMmyy,
    cardType
  ) {
    try {
      const ev = getEvervault();
      const lastFour = cardNumber.slice(-4);
      const [encNumber, encPassword, encBirthday, encExpiry, encExpiryYymm] =
        await Promise.all([
          ev.encrypt(cardNumber),
          ev.encrypt(cardPassword),
          ev.encrypt(birthday),
          ev.encrypt(expireMmyy),
          ev.encrypt(expireMmyy.slice(2) + expireMmyy.slice(0, 2)),
        ]);

      const response = await fetch("/api/cards", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({
          label: label || "My Card",
          card_number: encNumber,
          card_password: encPassword,
          birthday: encBirthday,
          expire_date: encExpiry,
          expire_date_yymm: encExpiryYymm,
          last_four: lastFour,
          card_type: cardType || "J",
        }),
      });

      if (!response.ok) {
        const body = await response
          .json()
          .catch(function () {
            return { message: "Card submission failed" };
          });
        return { ok: false, error: body.message || "Card submission failed" };
      }

      return { ok: true };
    } catch (error) {
      return {
        ok: false,
        error:
          error && typeof error.message === "string"
            ? error.message
            : "Card encryption failed",
      };
    }
  };
})();
