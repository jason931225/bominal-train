pub fn render_auth_landing() -> String {
    let html = r#"
<main class="mx-auto flex min-h-[100dvh] w-full px-4 py-6 2xl:px-8">
  <div class="my-auto mx-auto w-full 2xl:grid 2xl:max-w-[1600px] 2xl:grid-cols-[3fr_2fr] 2xl:items-center 2xl:gap-12">
    <section class="glass-card hidden rounded-[22px] p-6 md:p-8 2xl:block">
      <p class="eyebrow">dashboard preview</p>
      <h2 class="auth-title mt-2 text-2xl font-semibold">Operational clarity at a glance</h2>
      <p class="auth-copy mt-2 text-sm">Track runtime health, active sessions, and high-priority jobs in one place.</p>
      <div class="mt-5 space-y-2">
        <div class="summary-row"><span>Queued jobs</span><span class="badge">24</span></div>
        <div class="summary-row"><span>Running jobs</span><span class="badge">6</span></div>
        <div class="summary-row"><span>Error rate (5m)</span><span>0.3%</span></div>
        <div class="summary-row"><span>P95 latency</span><span>182ms</span></div>
      </div>
    </section>

    <div class="relative mb-8 mx-auto w-[90%] max-w-[420px] 2xl:mb-0 2xl:w-full 2xl:max-w-[420px] 2xl:justify-self-center">
      <section class="glass-card rounded-[22px] p-6 md:p-8">
        <p class="eyebrow">bominal authentication</p>
        <h1 class="sr-only" aria-label="Sign in securely"></h1>
        <div class="mt-3 flex justify-center">
          <div class="auth-hero-icon" role="img" aria-label="Secure account sign-in">
            <img id="auth-hero-passkey-icon" class="auth-hero-icon-main auth-hero-icon-visible" src="/assets/icons/runtime-ui/auth-hero-passkey-light.svgz" data-svgz-light="/assets/icons/runtime-ui/auth-hero-passkey-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/auth-hero-passkey-dark.svgz" alt="" aria-hidden="true" />
            <img id="auth-hero-password-icon" class="auth-hero-icon-main auth-hero-icon-hidden" src="/assets/icons/runtime-ui/auth-hero-password-light.svgz" data-svgz-light="/assets/icons/runtime-ui/auth-hero-password-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/auth-hero-password-dark.svgz" alt="" aria-hidden="true" />
          </div>
        </div>

        <div class="auth-action-region mt-6">
          <div id="auth-passkey-view" class="auth-pane" aria-hidden="false">
            <div class="action-group" data-action-group="pair">
              <button
                id="passkey-primary"
                class="btn-primary h-12 w-full"
                data-action-role="primary"
              >
                Authenticate with passkey
              </button>
              <button
                id="toggle-email"
                class="btn-ghost h-12 w-full"
                data-action-role="secondary"
              >
                Sign in with email
              </button>
            </div>
          </div>

          <div id="auth-email-view" class="auth-pane hidden" aria-hidden="true">
            <form id="email-signin-form" class="space-y-3">
              <label class="field-label" for="signin-email">Email</label>
              <input id="signin-email" type="email" autocomplete="email" class="field-input h-12 w-full" />
              <label class="field-label" for="signin-password">Password</label>
              <input id="signin-password" type="password" autocomplete="current-password" class="field-input h-12 w-full" />
              <div class="action-group" data-action-group="pair">
                <button
                  id="back-passkey"
                  type="button"
                  class="btn-ghost h-12 w-full"
                  data-action-role="secondary"
                >
                  Back to passkey
                </button>
                <button
                  id="email-continue"
                  type="submit"
                  class="btn-primary h-12 w-full"
                  data-action-role="primary"
                >
                  Continue
                </button>
              </div>
            </form>
          </div>
        </div>

        <div id="auth-error" class="mt-3 hidden error-card"></div>
      </section>

      <button
        type="button"
        class="theme-mini-switch"
        data-theme-toggle
        data-theme-toggle-compact
        aria-label="Theme toggle"
      >
        <img class="theme-mini-icon theme-mini-icon-sun" src="/assets/icons/runtime-ui/theme-mini-sun-active.svgz" data-svgz-light="/assets/icons/runtime-ui/theme-mini-sun-active.svgz" data-svgz-dark="/assets/icons/runtime-ui/theme-mini-sun-default.svgz" alt="" aria-hidden="true" />
        <img class="theme-mini-icon theme-mini-icon-moon" src="/assets/icons/runtime-ui/theme-mini-moon-default.svgz" data-svgz-light="/assets/icons/runtime-ui/theme-mini-moon-default.svgz" data-svgz-dark="/assets/icons/runtime-ui/theme-mini-moon-active.svgz" alt="" aria-hidden="true" />
        <span class="theme-mini-thumb" aria-hidden="true"></span>
      </button>
    </div>
  </div>
</main>

<script>
(() => {
  const passkeyBtn = document.getElementById('passkey-primary');
  const toggleEmailBtn = document.getElementById('toggle-email');
  const backPasskeyBtn = document.getElementById('back-passkey');
  const passkeyView = document.getElementById('auth-passkey-view');
  const emailView = document.getElementById('auth-email-view');
  const passkeyHeroIcon = document.getElementById('auth-hero-passkey-icon');
  const passwordHeroIcon = document.getElementById('auth-hero-password-icon');
  const emailForm = document.getElementById('email-signin-form');
  const authError = document.getElementById('auth-error');
  const reduceMotionQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
  const fadeDurationMs = reduceMotionQuery.matches ? 0 : 120;
  const PASSKEY_PROMPT_TIMEOUT_MS = 12000;
  const passkeyDefaultLabel = passkeyBtn ? passkeyBtn.textContent : 'Authenticate with passkey';
  let swapInProgress = false;
  const themedSvgzIcons = Array.from(document.querySelectorAll('img[data-svgz-light][data-svgz-dark]'));

  const currentThemeMode = () => document.body?.dataset?.themeMode === 'dark' ? 'dark' : 'light';

  const syncThemedSvgzIcons = (mode) => {
    const normalized = mode === 'dark' ? 'dark' : 'light';
    themedSvgzIcons.forEach((icon) => {
      const nextSrc = normalized === 'dark' ? icon.dataset.svgzDark : icon.dataset.svgzLight;
      if (!nextSrc || icon.getAttribute('src') === nextSrc) return;
      icon.setAttribute('src', nextSrc);
    });
  };

  const showError = (message) => {
    authError.textContent = message;
    authError.classList.remove('hidden');
  };

  const clearError = () => {
    authError.textContent = '';
    authError.classList.add('hidden');
  };

  const setHeroIcon = (mode) => {
    if (!passkeyHeroIcon || !passwordHeroIcon) return;
    const showPassword = mode === 'email';
    const fromIcon = showPassword ? passkeyHeroIcon : passwordHeroIcon;
    const toIcon = showPassword ? passwordHeroIcon : passkeyHeroIcon;
    if (toIcon.classList.contains('auth-hero-icon-visible')) return;
    fromIcon.classList.remove('auth-hero-icon-visible');
    fromIcon.classList.add('auth-hero-icon-hidden');
    toIcon.classList.remove('auth-hero-icon-hidden');
    toIcon.classList.add('auth-hero-icon-visible');
  };

  const switchView = (target) => {
    const showingEmail = target === 'email';
    const fromView = showingEmail ? passkeyView : emailView;
    const toView = showingEmail ? emailView : passkeyView;

    if (!fromView || !toView || swapInProgress || fromView.classList.contains('hidden')) {
      return;
    }
    swapInProgress = true;
    fromView.classList.add('auth-pane-fade-out');
    const commitSwitch = () => {
      fromView.classList.add('hidden');
      fromView.setAttribute('aria-hidden', 'true');
      fromView.classList.remove('auth-pane-fade-out');

      toView.classList.remove('hidden');
      toView.setAttribute('aria-hidden', 'false');
      toView.classList.add('auth-pane-fade-in');
      requestAnimationFrame(() => {
        toView.classList.remove('auth-pane-fade-in');
      });
      swapInProgress = false;
    };

    if (fadeDurationMs === 0) {
      commitSwitch();
    } else {
      window.setTimeout(commitSwitch, fadeDurationMs);
    }
  };

  const showPasskeyView = () => {
    clearError();
    setHeroIcon('passkey');
    if (!passkeyView.classList.contains('hidden')) return;
    switchView('passkey');
  };

  const showEmailView = () => {
    clearError();
    setHeroIcon('email');
    if (!emailView.classList.contains('hidden')) return;
    switchView('email');
  };

  const requestJson = async (url, method, payload) => {
    const response = await fetch(url, {
      method,
      headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
      body: payload ? JSON.stringify(payload) : undefined,
    });
    const bodyText = await response.text();
    let body = null;
    try { body = bodyText ? JSON.parse(bodyText) : null; } catch (_err) {}
    return { ok: response.ok, status: response.status, body, bodyText };
  };

  const b64urlToBuffer = (value) => {
    const padded = (value + '==='.slice((value.length + 3) % 4)).replace(/-/g, '+').replace(/_/g, '/');
    const binary = atob(padded);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
    return bytes.buffer;
  };

  const bufferToB64url = (buffer) => {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.length; i += 1) binary += String.fromCharCode(bytes[i]);
    return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  };

  const setPasskeyBusy = (busy, label) => {
    if (!passkeyBtn) return;
    passkeyBtn.disabled = busy;
    passkeyBtn.setAttribute('aria-busy', busy ? 'true' : 'false');
    passkeyBtn.textContent = busy ? label : passkeyDefaultLabel;
  };

  const serializeAuthCredential = (credential) => ({
    id: credential.id,
    rawId: bufferToB64url(credential.rawId),
    type: credential.type,
    response: {
      authenticatorData: bufferToB64url(credential.response.authenticatorData),
      clientDataJSON: bufferToB64url(credential.response.clientDataJSON),
      signature: bufferToB64url(credential.response.signature),
      userHandle: credential.response.userHandle ? bufferToB64url(credential.response.userHandle) : null,
    },
    clientExtensionResults: credential.getClientExtensionResults ? credential.getClientExtensionResults() : {},
  });

  const getPasskeyCredential = async (options) => {
    if (typeof AbortController !== 'function') {
      return navigator.credentials.get(options);
    }
    const controller = new AbortController();
    const timeoutId = window.setTimeout(() => controller.abort(), PASSKEY_PROMPT_TIMEOUT_MS);
    try {
      return await navigator.credentials.get({ ...options, signal: controller.signal });
    } finally {
      window.clearTimeout(timeoutId);
    }
  };

  const passkeyAuth = async () => {
    if (!passkeyBtn || passkeyBtn.disabled) return;
    clearError();
    setPasskeyBusy(true, 'Preparing passkey...');
    try {
      const start = await requestJson('/api/auth/passkeys/auth/start', 'POST', {});
      if (!start.ok || !start.body || !start.body.options || !start.body.flow_id) {
        const requestId = start.body && start.body.request_id ? ` (request_id: ${start.body.request_id})` : '';
        showError(`Passkey start failed${requestId}`);
        return;
      }
      if (!window.PublicKeyCredential || !navigator.credentials) {
        showError('WebAuthn is not supported in this browser.');
        return;
      }
      const options = typeof structuredClone === 'function'
        ? structuredClone(start.body.options)
        : JSON.parse(JSON.stringify(start.body.options));
      if (options.mediation === 'conditional') {
        options.mediation = 'required';
      }
      options.publicKey.challenge = b64urlToBuffer(options.publicKey.challenge);
      if (Array.isArray(options.publicKey.allowCredentials)) {
        options.publicKey.allowCredentials = options.publicKey.allowCredentials.map((item) => ({ ...item, id: b64urlToBuffer(item.id) }));
      }
      setPasskeyBusy(true, 'Waiting for passkey...');
      const credential = await getPasskeyCredential(options);
      if (!credential) {
        showError('Passkey authentication was cancelled.');
        return;
      }
      const finish = await requestJson('/api/auth/passkeys/auth/finish', 'POST', {
        flow_id: start.body.flow_id,
        credential: serializeAuthCredential(credential),
      });
      if (!finish.ok) {
        const requestId = finish.body && finish.body.request_id ? ` (request_id: ${finish.body.request_id})` : '';
        showError(`Passkey sign-in failed${requestId}`);
        return;
      }
      window.location.href = '/dashboard';
    } catch (err) {
      if (err && typeof err === 'object' && err.name === 'SecurityError') {
        showEmailView();
        showError('Passkey is unavailable on this host. Use email/password sign-in.');
        return;
      }
      if (err && typeof err === 'object' && err.name === 'NotAllowedError') {
        showEmailView();
        showError('Passkey sign-in was cancelled. Continue with email sign-in.');
        return;
      }
      if (err && typeof err === 'object' && err.name === 'AbortError') {
        showEmailView();
        showError('Passkey sign-in timed out. Continue with email sign-in.');
        return;
      }
      showError('Passkey authentication failed. Use email/password if needed.');
    } finally {
      setPasskeyBusy(false);
    }
  };

  passkeyBtn.addEventListener('click', passkeyAuth);
  toggleEmailBtn.addEventListener('click', showEmailView);
  backPasskeyBtn.addEventListener('click', showPasskeyView);

  emailForm.addEventListener('submit', async (event) => {
    event.preventDefault();
    clearError();
    const email = document.getElementById('signin-email').value.trim();
    const password = document.getElementById('signin-password').value;
    const result = await requestJson('/api/auth/password/signin', 'POST', { email, password });
    if (!result.ok) {
      const requestId = result.body && result.body.request_id ? ` (request_id: ${result.body.request_id})` : '';
      showError(`Sign-in failed${requestId}`);
      return;
    }
    window.location.href = '/dashboard';
  });

  syncThemedSvgzIcons(currentThemeMode());
  if (document.body && typeof MutationObserver === 'function') {
    const themeObserver = new MutationObserver(() => {
      syncThemedSvgzIcons(currentThemeMode());
    });
    themeObserver.observe(document.body, { attributes: true, attributeFilter: ['data-theme-mode'] });
  }

})();
</script>
"#;
    html.to_string()
}
