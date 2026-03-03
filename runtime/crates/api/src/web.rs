use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <main class="mx-auto flex min-h-screen w-full max-w-6xl flex-col px-6 py-12 lg:px-10">
            <section class="rounded-3xl border border-slate-200 bg-white p-8 shadow-sm">
                <p class="text-xs uppercase tracking-[0.24em] text-slate-500">"Bominal Rust Cutover"</p>
                <h1 class="mt-3 text-4xl font-black leading-tight text-slate-900 lg:text-6xl">
                    "Leptos SSR + Tailwind"
                </h1>
                <p class="mt-4 max-w-3xl text-base leading-7 text-slate-600 lg:text-lg">
                    "Parallel runtime is active. This frontend is server-rendered by Leptos on axum with local auth, passkeys, and Redis-backed runtime services."
                </p>
                <div class="mt-8 grid grid-cols-1 gap-4 sm:grid-cols-3">
                    <article class="rounded-2xl bg-slate-900 p-5 text-slate-100">
                        <p class="text-xs uppercase tracking-wide text-slate-400">"API"</p>
                        <p class="mt-2 text-2xl font-semibold">"axum 0.8"</p>
                    </article>
                    <article class="rounded-2xl bg-cyan-600 p-5 text-cyan-50">
                        <p class="text-xs uppercase tracking-wide text-cyan-100">"Data"</p>
                        <p class="mt-2 text-2xl font-semibold">"postgres + redis"</p>
                    </article>
                    <article class="rounded-2xl bg-emerald-600 p-5 text-emerald-50">
                        <p class="text-xs uppercase tracking-wide text-emerald-100">"Auth"</p>
                        <p class="mt-2 text-2xl font-semibold">"server WebAuthn"</p>
                    </article>
                </div>
            </section>
        </main>
    }
}

pub fn render_home() -> String {
    view! { <HomePage /> }.to_html()
}

#[derive(Debug, Clone)]
pub struct AuthPreflight {
    pub database_configured: bool,
    pub redis_configured: bool,
    pub session_secret_configured: bool,
    pub invite_base_url_configured: bool,
    pub passkey_provider_server_only: bool,
    pub webauthn_rp_id_configured: bool,
    pub webauthn_rp_origin_configured: bool,
}

fn status_badge(ready: bool) -> &'static str {
    if ready {
        "<span class=\"rounded-full bg-emerald-100 px-2.5 py-1 text-xs font-semibold uppercase tracking-wide text-emerald-700\">Ready</span>"
    } else {
        "<span class=\"rounded-full bg-rose-100 px-2.5 py-1 text-xs font-semibold uppercase tracking-wide text-rose-700\">Missing</span>"
    }
}

fn status_row(name: &str, ready: bool, hint: &str) -> String {
    format!(
        "<li class=\"flex items-center justify-between rounded-xl border border-slate-200 bg-slate-50 px-4 py-3\"><div><p class=\"text-sm font-semibold text-slate-900\">{name}</p><p class=\"text-xs text-slate-500\">{hint}</p></div>{}</li>",
        status_badge(ready)
    )
}

pub fn render_auth(preflight: &AuthPreflight) -> String {
    let rows = [
        status_row(
            "DATABASE_URL",
            preflight.database_configured,
            "Persistent store for auth users, invites, and passkeys.",
        ),
        status_row(
            "REDIS_URL",
            preflight.redis_configured,
            "Session store + passkey challenge TTL state.",
        ),
        status_row(
            "SESSION_SECRET",
            preflight.session_secret_configured,
            "Server-side secret for token hashing and invite protection.",
        ),
        status_row(
            "INVITE_BASE_URL",
            preflight.invite_base_url_configured,
            "Base URL used when issuing invite links.",
        ),
        status_row(
            "PASSKEY_PROVIDER=server_webauthn",
            preflight.passkey_provider_server_only,
            "WebAuthn provider mode (single-provider runtime).",
        ),
        status_row(
            "WEBAUTHN_RP_ID",
            preflight.webauthn_rp_id_configured,
            "Relying-party ID for passkey ceremonies.",
        ),
        status_row(
            "WEBAUTHN_RP_ORIGIN",
            preflight.webauthn_rp_origin_configured,
            "Expected browser origin for passkey ceremonies.",
        ),
    ]
    .join("");

    let summary = if preflight.database_configured
        && preflight.redis_configured
        && preflight.session_secret_configured
        && preflight.passkey_provider_server_only
        && preflight.webauthn_rp_id_configured
        && preflight.webauthn_rp_origin_configured
    {
        "<p class=\"rounded-xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800\">Local auth runtime contract is ready.</p>"
    } else {
        "<p class=\"rounded-xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800\">Set missing keys in <code>env/local/runtime.env</code> before production-like testing.</p>"
    };

    let html = r#"
<main class="mx-auto flex min-h-screen w-full max-w-6xl flex-col gap-6 px-6 py-12 lg:px-10">
  <section class="rounded-3xl border border-slate-200 bg-white p-8 shadow-sm">
    <p class="text-xs uppercase tracking-[0.24em] text-slate-500">bominal auth</p>
    <h1 class="mt-3 text-4xl font-black leading-tight text-slate-900 lg:text-6xl">Auth Workspace</h1>
    <p class="mt-4 max-w-3xl text-base leading-7 text-slate-600 lg:text-lg">
      Server-only WebAuthn with invite-only onboarding and password fallback.
    </p>
  </section>

  <section class="grid grid-cols-1 gap-6 lg:grid-cols-2">
    <article class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm">
      <h2 class="text-xl font-semibold text-slate-900">Auth Preflight</h2>
      <p class="mt-2 text-sm text-slate-600">Runtime auth contract checks.</p>
      <ul class="mt-4 space-y-3">__ROWS__</ul>
      <div class="mt-4">__SUMMARY__</div>
    </article>

    <article class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm">
      <h2 class="text-xl font-semibold text-slate-900">Session</h2>
      <p class="mt-2 text-sm text-slate-600">Current app session cookie status.</p>
      <div class="mt-4 flex gap-3">
        <button id="refresh-session" type="button" class="inline-flex items-center rounded-xl bg-cyan-600 px-4 py-2 text-sm font-semibold text-cyan-50 hover:bg-cyan-500">Refresh</button>
        <button id="logout-session" type="button" class="inline-flex items-center rounded-xl border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-900 hover:bg-slate-100">Logout</button>
      </div>
      <pre id="session-output" class="mt-4 overflow-x-auto rounded-xl bg-slate-900 p-4 text-xs leading-6 text-slate-100">Loading session...</pre>
    </article>
  </section>

  <section class="grid grid-cols-1 gap-6 lg:grid-cols-2">
    <article class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm">
      <h2 class="text-xl font-semibold text-slate-900">Invite Accept</h2>
      <p class="mt-2 text-sm text-slate-600">Invite-only onboarding flow.</p>
      <form id="invite-form" class="mt-4 space-y-3">
        <div>
          <label for="invite-token" class="mb-2 block text-sm font-medium text-slate-700">Invite token</label>
          <input id="invite-token" type="text" class="w-full rounded-xl border border-slate-300 px-3 py-2 text-sm text-slate-900 focus:border-cyan-500 focus:outline-none focus:ring-2 focus:ring-cyan-200" />
        </div>
        <div>
          <label for="invite-email" class="mb-2 block text-sm font-medium text-slate-700">Email</label>
          <input id="invite-email" type="email" autocomplete="email" class="w-full rounded-xl border border-slate-300 px-3 py-2 text-sm text-slate-900 focus:border-cyan-500 focus:outline-none focus:ring-2 focus:ring-cyan-200" />
        </div>
        <div>
          <label for="invite-password" class="mb-2 block text-sm font-medium text-slate-700">Password</label>
          <input id="invite-password" type="password" autocomplete="new-password" class="w-full rounded-xl border border-slate-300 px-3 py-2 text-sm text-slate-900 focus:border-cyan-500 focus:outline-none focus:ring-2 focus:ring-cyan-200" />
        </div>
        <button type="submit" class="inline-flex items-center rounded-xl bg-slate-900 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800">Accept invite</button>
      </form>
      <pre id="invite-output" class="mt-4 overflow-x-auto rounded-xl bg-slate-900 p-4 text-xs leading-6 text-slate-100">Accept an invite to create an account.</pre>
    </article>

    <article class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm">
      <h2 class="text-xl font-semibold text-slate-900">Password Sign In</h2>
      <p class="mt-2 text-sm text-slate-600">Fallback sign in path.</p>
      <form id="password-signin-form" class="mt-4 space-y-3">
        <div>
          <label for="signin-email" class="mb-2 block text-sm font-medium text-slate-700">Email</label>
          <input id="signin-email" type="email" autocomplete="email" class="w-full rounded-xl border border-slate-300 px-3 py-2 text-sm text-slate-900 focus:border-cyan-500 focus:outline-none focus:ring-2 focus:ring-cyan-200" />
        </div>
        <div>
          <label for="signin-password" class="mb-2 block text-sm font-medium text-slate-700">Password</label>
          <input id="signin-password" type="password" autocomplete="current-password" class="w-full rounded-xl border border-slate-300 px-3 py-2 text-sm text-slate-900 focus:border-cyan-500 focus:outline-none focus:ring-2 focus:ring-cyan-200" />
        </div>
        <button type="submit" class="inline-flex items-center rounded-xl bg-slate-900 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800">Sign in</button>
      </form>
      <pre id="signin-output" class="mt-4 overflow-x-auto rounded-xl bg-slate-900 p-4 text-xs leading-6 text-slate-100">Sign in to establish a session cookie.</pre>
    </article>
  </section>

  <section class="grid grid-cols-1 gap-6 lg:grid-cols-2">
    <article class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm">
      <h2 class="text-xl font-semibold text-slate-900">Passkey Sign In</h2>
      <p class="mt-2 text-sm text-slate-600">Usernameless discoverable credential flow.</p>
      <button id="passkey-signin-button" type="button" class="mt-4 inline-flex items-center rounded-xl bg-cyan-600 px-4 py-2 text-sm font-semibold text-cyan-50 hover:bg-cyan-500">Sign in with passkey</button>
      <pre id="passkey-signin-output" class="mt-4 overflow-x-auto rounded-xl bg-slate-900 p-4 text-xs leading-6 text-slate-100">Use your passkey to sign in.</pre>
    </article>

    <article class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm">
      <h2 class="text-xl font-semibold text-slate-900">Passkey Register</h2>
      <p class="mt-2 text-sm text-slate-600">Requires an active session cookie.</p>
      <div class="mt-4 space-y-3">
        <div>
          <label for="passkey-friendly-name" class="mb-2 block text-sm font-medium text-slate-700">Friendly name</label>
          <input id="passkey-friendly-name" type="text" class="w-full rounded-xl border border-slate-300 px-3 py-2 text-sm text-slate-900 focus:border-cyan-500 focus:outline-none focus:ring-2 focus:ring-cyan-200" placeholder="My laptop" />
        </div>
        <button id="passkey-register-button" type="button" class="inline-flex items-center rounded-xl border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-900 hover:bg-slate-100">Register passkey</button>
      </div>
      <pre id="passkey-register-output" class="mt-4 overflow-x-auto rounded-xl bg-slate-900 p-4 text-xs leading-6 text-slate-100">Register a passkey after sign-in.</pre>
    </article>
  </section>
</main>

<script>
  (() => {
    const sessionOutput = document.getElementById('session-output');
    const refreshSessionButton = document.getElementById('refresh-session');
    const logoutSessionButton = document.getElementById('logout-session');

    const inviteForm = document.getElementById('invite-form');
    const inviteTokenInput = document.getElementById('invite-token');
    const inviteEmailInput = document.getElementById('invite-email');
    const invitePasswordInput = document.getElementById('invite-password');
    const inviteOutput = document.getElementById('invite-output');

    const passwordSigninForm = document.getElementById('password-signin-form');
    const signinEmailInput = document.getElementById('signin-email');
    const signinPasswordInput = document.getElementById('signin-password');
    const signinOutput = document.getElementById('signin-output');

    const passkeySigninButton = document.getElementById('passkey-signin-button');
    const passkeySigninOutput = document.getElementById('passkey-signin-output');

    const passkeyRegisterButton = document.getElementById('passkey-register-button');
    const passkeyFriendlyNameInput = document.getElementById('passkey-friendly-name');
    const passkeyRegisterOutput = document.getElementById('passkey-register-output');

    function setOutput(el, payload) {
      if (!el) return;
      el.textContent = typeof payload === 'string' ? payload : JSON.stringify(payload, null, 2);
    }

    function toErrorMessage(result, fallback) {
      if (!result) return fallback;
      if (typeof result === 'string') return result;
      if (result.body && typeof result.body.message === 'string') return result.body.message;
      if (typeof result.status === 'number') return fallback + ' (status ' + result.status + ')';
      return fallback;
    }

    async function requestJson(url, method, payload) {
      const response = await fetch(url, {
        method,
        headers: {
          'Content-Type': 'application/json',
          Accept: 'application/json',
        },
        body: payload ? JSON.stringify(payload) : undefined,
      });

      const text = await response.text();
      let body = text;
      try {
        body = text ? JSON.parse(text) : null;
      } catch (_err) {
        // keep raw text
      }

      return {
        ok: response.ok,
        status: response.status,
        body,
      };
    }

    function b64urlToBuffer(base64url) {
      const padded = (base64url + '==='.slice((base64url.length + 3) % 4)).replace(/-/g, '+').replace(/_/g, '/');
      const binary = atob(padded);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i += 1) {
        bytes[i] = binary.charCodeAt(i);
      }
      return bytes.buffer;
    }

    function bufferToB64url(buffer) {
      const bytes = new Uint8Array(buffer);
      let binary = '';
      for (let i = 0; i < bytes.length; i += 1) {
        binary += String.fromCharCode(bytes[i]);
      }
      return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    }

    function prepareRegistrationOptions(options) {
      const publicKey = structuredClone(options.publicKey);
      publicKey.challenge = b64urlToBuffer(publicKey.challenge);
      publicKey.user.id = b64urlToBuffer(publicKey.user.id);
      if (Array.isArray(publicKey.excludeCredentials)) {
        publicKey.excludeCredentials = publicKey.excludeCredentials.map((credential) => ({
          ...credential,
          id: b64urlToBuffer(credential.id),
        }));
      }
      return { publicKey };
    }

    function prepareAuthenticationOptions(options) {
      const publicKey = structuredClone(options.publicKey);
      publicKey.challenge = b64urlToBuffer(publicKey.challenge);
      if (Array.isArray(publicKey.allowCredentials)) {
        publicKey.allowCredentials = publicKey.allowCredentials.map((credential) => ({
          ...credential,
          id: b64urlToBuffer(credential.id),
        }));
      }
      return {
        publicKey,
        mediation: options.mediation,
      };
    }

    function serializeRegistrationCredential(credential) {
      return {
        id: credential.id,
        rawId: bufferToB64url(credential.rawId),
        type: credential.type,
        response: {
          attestationObject: bufferToB64url(credential.response.attestationObject),
          clientDataJSON: bufferToB64url(credential.response.clientDataJSON),
          transports: typeof credential.response.getTransports === 'function'
            ? credential.response.getTransports()
            : undefined,
        },
        clientExtensionResults: credential.getClientExtensionResults
          ? credential.getClientExtensionResults()
          : {},
      };
    }

    function serializeAuthenticationCredential(credential) {
      return {
        id: credential.id,
        rawId: bufferToB64url(credential.rawId),
        type: credential.type,
        response: {
          authenticatorData: bufferToB64url(credential.response.authenticatorData),
          clientDataJSON: bufferToB64url(credential.response.clientDataJSON),
          signature: bufferToB64url(credential.response.signature),
          userHandle: credential.response.userHandle
            ? bufferToB64url(credential.response.userHandle)
            : null,
        },
        clientExtensionResults: credential.getClientExtensionResults
          ? credential.getClientExtensionResults()
          : {},
      };
    }

    async function refreshSession() {
      const result = await requestJson('/api/auth/session/me', 'GET');
      setOutput(sessionOutput, result.body || result);
    }

    if (refreshSessionButton) {
      refreshSessionButton.addEventListener('click', async () => {
        await refreshSession();
      });
    }

    if (logoutSessionButton) {
      logoutSessionButton.addEventListener('click', async () => {
        const result = await requestJson('/api/auth/session/logout', 'POST');
        if (!result.ok) {
          setOutput(sessionOutput, toErrorMessage(result, 'Logout failed'));
          return;
        }
        await refreshSession();
      });
    }

    if (inviteTokenInput) {
      const params = new URLSearchParams(window.location.search);
      const token = params.get('invite_token');
      if (token) {
        inviteTokenInput.value = token;
      }
    }

    if (inviteForm) {
      inviteForm.addEventListener('submit', async (event) => {
        event.preventDefault();
        const payload = {
          invite_token: inviteTokenInput.value.trim(),
          email: inviteEmailInput.value.trim(),
          password: invitePasswordInput.value,
        };

        setOutput(inviteOutput, 'Accepting invite...');
        const result = await requestJson('/api/auth/invite/accept', 'POST', payload);
        if (!result.ok) {
          setOutput(inviteOutput, toErrorMessage(result, 'Invite accept failed'));
          return;
        }

        setOutput(inviteOutput, result.body || result);
        await refreshSession();
      });
    }

    if (passwordSigninForm) {
      passwordSigninForm.addEventListener('submit', async (event) => {
        event.preventDefault();
        const payload = {
          email: signinEmailInput.value.trim(),
          password: signinPasswordInput.value,
        };

        setOutput(signinOutput, 'Signing in...');
        const result = await requestJson('/api/auth/password/signin', 'POST', payload);
        if (!result.ok) {
          setOutput(signinOutput, toErrorMessage(result, 'Sign in failed'));
          return;
        }

        setOutput(signinOutput, result.body || result);
        await refreshSession();
      });
    }

    if (passkeySigninButton) {
      passkeySigninButton.addEventListener('click', async () => {
        if (!window.PublicKeyCredential || !navigator.credentials) {
          setOutput(passkeySigninOutput, 'WebAuthn is not supported in this browser.');
          return;
        }

        setOutput(passkeySigninOutput, 'Starting passkey sign-in...');

        const start = await requestJson('/api/auth/passkeys/auth/start', 'POST', {});
        if (!start.ok || !start.body || !start.body.options || !start.body.flow_id) {
          setOutput(passkeySigninOutput, toErrorMessage(start, 'Failed to start passkey sign-in'));
          return;
        }

        try {
          const credential = await navigator.credentials.get(prepareAuthenticationOptions(start.body.options));
          if (!credential) {
            setOutput(passkeySigninOutput, 'Passkey sign-in was cancelled.');
            return;
          }

          const finish = await requestJson('/api/auth/passkeys/auth/finish', 'POST', {
            flow_id: start.body.flow_id,
            credential: serializeAuthenticationCredential(credential),
          });

          if (!finish.ok) {
            setOutput(passkeySigninOutput, toErrorMessage(finish, 'Passkey sign-in failed'));
            return;
          }

          setOutput(passkeySigninOutput, finish.body || finish);
          await refreshSession();
        } catch (err) {
          setOutput(passkeySigninOutput, String(err));
        }
      });
    }

    if (passkeyRegisterButton) {
      passkeyRegisterButton.addEventListener('click', async () => {
        if (!window.PublicKeyCredential || !navigator.credentials) {
          setOutput(passkeyRegisterOutput, 'WebAuthn is not supported in this browser.');
          return;
        }

        const payload = {
          friendly_name: passkeyFriendlyNameInput.value.trim() || null,
        };

        setOutput(passkeyRegisterOutput, 'Starting passkey registration...');
        const start = await requestJson('/api/auth/passkeys/register/start', 'POST', payload);
        if (!start.ok || !start.body || !start.body.options || !start.body.flow_id) {
          setOutput(passkeyRegisterOutput, toErrorMessage(start, 'Failed to start passkey registration'));
          return;
        }

        try {
          const credential = await navigator.credentials.create(prepareRegistrationOptions(start.body.options));
          if (!credential) {
            setOutput(passkeyRegisterOutput, 'Passkey registration was cancelled.');
            return;
          }

          const finish = await requestJson('/api/auth/passkeys/register/finish', 'POST', {
            flow_id: start.body.flow_id,
            credential: serializeRegistrationCredential(credential),
          });

          if (!finish.ok) {
            setOutput(passkeyRegisterOutput, toErrorMessage(finish, 'Passkey registration failed'));
            return;
          }

          setOutput(passkeyRegisterOutput, finish.body || finish);
        } catch (err) {
          setOutput(passkeyRegisterOutput, String(err));
        }
      });
    }

    refreshSession().catch((err) => {
      setOutput(sessionOutput, String(err));
    });
  })();
</script>
"#;

    html.replace("__ROWS__", &rows)
        .replace("__SUMMARY__", summary)
}

#[derive(Debug, Clone)]
pub struct AdminMaintenanceView {
    pub admin_email: String,
    pub db_ok: bool,
    pub redis_ok: bool,
    pub ready_ok: bool,
    pub health_path: &'static str,
    pub ready_path: &'static str,
    pub metrics_path: &'static str,
    pub metrics_snapshot: String,
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn readiness_badge(ok: bool) -> &'static str {
    if ok {
        "<span class=\"rounded-full bg-emerald-100 px-2.5 py-1 text-xs font-semibold uppercase tracking-wide text-emerald-700\">Healthy</span>"
    } else {
        "<span class=\"rounded-full bg-rose-100 px-2.5 py-1 text-xs font-semibold uppercase tracking-wide text-rose-700\">Degraded</span>"
    }
}

pub fn render_admin_maintenance(view: &AdminMaintenanceView) -> String {
    let admin_email = html_escape(&view.admin_email);
    let metrics_snapshot = html_escape(&view.metrics_snapshot);
    let db_badge = readiness_badge(view.db_ok);
    let redis_badge = readiness_badge(view.redis_ok);
    let ready_badge = readiness_badge(view.ready_ok);

    let html = r#"
<main class="mx-auto flex min-h-screen w-full max-w-7xl flex-col gap-6 px-6 py-12 lg:px-10">
  <section class="rounded-3xl border border-slate-200 bg-white p-8 shadow-sm">
    <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <p class="text-xs uppercase tracking-[0.24em] text-slate-500">bominal admin</p>
        <h1 class="mt-2 text-3xl font-black leading-tight text-slate-900 lg:text-5xl">Maintenance Dashboard</h1>
        <p class="mt-3 text-sm text-slate-600">Signed in as <code class="rounded bg-slate-100 px-2 py-1 text-xs text-slate-800">__ADMIN_EMAIL__</code></p>
      </div>
      __READY_BADGE__
    </div>
  </section>

  <section class="grid grid-cols-1 gap-6 lg:grid-cols-3">
    <article class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
      <div class="flex items-center justify-between">
        <h2 class="text-base font-semibold text-slate-900">Database</h2>
        __DB_BADGE__
      </div>
      <p class="mt-3 text-sm text-slate-600">Readiness probe dependency status.</p>
    </article>
    <article class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
      <div class="flex items-center justify-between">
        <h2 class="text-base font-semibold text-slate-900">Redis</h2>
        __REDIS_BADGE__
      </div>
      <p class="mt-3 text-sm text-slate-600">Session and queue cache dependency status.</p>
    </article>
    <article class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
      <h2 class="text-base font-semibold text-slate-900">Probe Endpoints</h2>
      <div class="mt-3 space-y-2 text-sm">
        <a class="block rounded-lg border border-slate-200 px-3 py-2 text-cyan-700 hover:bg-cyan-50" href="__HEALTH_PATH__" target="_blank" rel="noreferrer">GET __HEALTH_PATH__</a>
        <a class="block rounded-lg border border-slate-200 px-3 py-2 text-cyan-700 hover:bg-cyan-50" href="__READY_PATH__" target="_blank" rel="noreferrer">GET __READY_PATH__</a>
      </div>
    </article>
  </section>

  <section class="rounded-3xl border border-slate-200 bg-white p-6 shadow-sm">
    <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <h2 class="text-xl font-semibold text-slate-900">Prometheus Metrics</h2>
        <p class="mt-1 text-sm text-slate-600">Admin-only metrics view for runtime observability.</p>
      </div>
      <button id="refresh-metrics" type="button" class="inline-flex items-center rounded-xl bg-slate-900 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800">Refresh metrics</button>
    </div>
    <pre id="metrics-output" class="mt-4 max-h-[28rem] overflow-auto rounded-2xl bg-slate-950 p-4 text-xs leading-5 text-slate-100">__METRICS_SNAPSHOT__</pre>
  </section>
</main>

<script>
  (() => {
    const refreshButton = document.getElementById('refresh-metrics');
    const metricsOutput = document.getElementById('metrics-output');
    const metricsPath = '__METRICS_PATH__';

    if (!refreshButton || !metricsOutput) {
      return;
    }

    refreshButton.addEventListener('click', async () => {
      metricsOutput.textContent = 'Refreshing metrics...';
      try {
        const response = await fetch(metricsPath, {
          method: 'GET',
          headers: { Accept: 'text/plain' },
        });
        const text = await response.text();
        if (!response.ok) {
          metricsOutput.textContent = text || ('Request failed with status ' + response.status);
          return;
        }
        metricsOutput.textContent = text;
      } catch (error) {
        metricsOutput.textContent = String(error);
      }
    });
  })();
</script>
"#;

    html.replace("__ADMIN_EMAIL__", &admin_email)
        .replace("__READY_BADGE__", ready_badge)
        .replace("__DB_BADGE__", db_badge)
        .replace("__REDIS_BADGE__", redis_badge)
        .replace("__HEALTH_PATH__", view.health_path)
        .replace("__READY_PATH__", view.ready_path)
        .replace("__METRICS_PATH__", view.metrics_path)
        .replace("__METRICS_SNAPSHOT__", &metrics_snapshot)
}
