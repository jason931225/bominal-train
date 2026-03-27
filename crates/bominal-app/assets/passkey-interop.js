(function () {
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
})();
