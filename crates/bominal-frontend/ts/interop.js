var BominalInterop = (() => {
  var __getOwnPropNames = Object.getOwnPropertyNames;
  var __commonJS = (cb, mod) => function __require() {
    return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
  };

  // crates/bominal-frontend/ts/interop.ts
  var require_interop = __commonJS({
    "crates/bominal-frontend/ts/interop.ts"() {
      var evInstance = null;
      function getEvervault() {
        if (evInstance) return evInstance;
        const teamId = document.querySelector('meta[name="ev-team-id"]')?.getAttribute("content");
        const appId = document.querySelector('meta[name="ev-app-id"]')?.getAttribute("content");
        if (!teamId || !appId) throw new Error("Evervault meta tags missing");
        evInstance = new Evervault(teamId, appId);
        return evInstance;
      }
      window.__evEncrypt = async function(plaintext) {
        const ev = getEvervault();
        return await ev.encrypt(plaintext);
      };
      window.__submitCard = async function(label, cardNumber, cardPassword, birthday, expireMmyy, cardType) {
        try {
          const ev = getEvervault();
          const lastFour = cardNumber.slice(-4);
          const [encNumber, encPassword, encBirthday, encExpiry, encExpiryYymm] = await Promise.all([
            ev.encrypt(cardNumber),
            ev.encrypt(cardPassword),
            ev.encrypt(birthday),
            ev.encrypt(expireMmyy),
            ev.encrypt(expireMmyy.slice(2) + expireMmyy.slice(0, 2))
            // MMYY -> YYMM
          ]);
          const resp = await fetch("/api/cards", {
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
              card_type: cardType || "J"
            })
          });
          if (!resp.ok) {
            const body = await resp.json().catch(() => ({ message: "Request failed" }));
            return { ok: false, error: body.message || `HTTP ${resp.status}` };
          }
          const card = await resp.json();
          return { ok: true, card };
        } catch (e) {
          return { ok: false, error: e.message || "Encryption failed" };
        }
      };
      function toBase64url(buffer) {
        return btoa(String.fromCharCode(...new Uint8Array(buffer))).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
      }
      window.__startViewTransition = function(cb) {
        if ("startViewTransition" in document) {
          document.startViewTransition(cb);
        } else {
          cb();
        }
      };
      window.__startPasskeyRegistration = async function(optionsJson) {
        const options = JSON.parse(optionsJson);
        options.challenge = Uint8Array.from(
          atob(options.challenge.replace(/-/g, "+").replace(/_/g, "/")),
          (c) => c.charCodeAt(0)
        );
        if (options.user?.id) {
          const b64 = options.user.id.replace(/-/g, "+").replace(/_/g, "/");
          options.user.id = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
        }
        if (options.excludeCredentials) {
          options.excludeCredentials = options.excludeCredentials.map((c) => ({
            ...c,
            id: Uint8Array.from(
              atob(c.id.replace(/-/g, "+").replace(/_/g, "/")),
              (ch) => ch.charCodeAt(0)
            )
          }));
        }
        const credential = await navigator.credentials.create({ publicKey: options });
        const response = credential.response;
        return JSON.stringify({
          id: credential.id,
          rawId: toBase64url(credential.rawId),
          type: credential.type,
          response: {
            attestationObject: toBase64url(response.attestationObject),
            clientDataJSON: toBase64url(response.clientDataJSON)
          }
        });
      };
      window.__startPasskeyLogin = async function(optionsJson) {
        const options = JSON.parse(optionsJson);
        options.challenge = Uint8Array.from(
          atob(options.challenge.replace(/-/g, "+").replace(/_/g, "/")),
          (c) => c.charCodeAt(0)
        );
        if (options.allowCredentials) {
          options.allowCredentials = options.allowCredentials.map((c) => ({
            ...c,
            id: Uint8Array.from(
              atob(c.id.replace(/-/g, "+").replace(/_/g, "/")),
              (ch) => ch.charCodeAt(0)
            )
          }));
        }
        const assertion = await navigator.credentials.get({ publicKey: options });
        const response = assertion.response;
        return JSON.stringify({
          id: assertion.id,
          rawId: toBase64url(assertion.rawId),
          type: assertion.type,
          response: {
            authenticatorData: toBase64url(response.authenticatorData),
            clientDataJSON: toBase64url(response.clientDataJSON),
            signature: toBase64url(response.signature),
            userHandle: response.userHandle ? toBase64url(response.userHandle) : null
          }
        });
      };
      window.__startConditionalPasskeyLogin = async function(optionsJson) {
        const options = JSON.parse(optionsJson);
        options.challenge = Uint8Array.from(
          atob(options.challenge.replace(/-/g, "+").replace(/_/g, "/")),
          (c) => c.charCodeAt(0)
        );
        if (options.allowCredentials) {
          options.allowCredentials = options.allowCredentials.map((c) => ({
            ...c,
            id: Uint8Array.from(
              atob(c.id.replace(/-/g, "+").replace(/_/g, "/")),
              (ch) => ch.charCodeAt(0)
            )
          }));
        }
        const assertion = await navigator.credentials.get({
          publicKey: options,
          mediation: "conditional"
        });
        const response = assertion.response;
        return JSON.stringify({
          id: assertion.id,
          rawId: toBase64url(assertion.rawId),
          type: assertion.type,
          response: {
            authenticatorData: toBase64url(response.authenticatorData),
            clientDataJSON: toBase64url(response.clientDataJSON),
            signature: toBase64url(response.signature),
            userHandle: response.userHandle ? toBase64url(response.userHandle) : null
          }
        });
      };
    }
  });
  return require_interop();
})();
