use bominal_ui::{
    AdminSection, DashboardSection, SettingsTab, render_admin_bottom_nav, render_admin_sidebar,
    render_app_topbar, render_dashboard_bottom_nav, render_dashboard_sidebar,
    render_dev_ui_showcase, render_settings_tabs, html_escape as primitive_html_escape,
};

fn html_escape(value: &str) -> String {
    primitive_html_escape(value)
}

fn app_shell_topbar(title: &str, subtitle: &str) -> String {
    render_app_topbar(title, subtitle)
}

fn dashboard_desktop_sidebar(active: &str) -> String {
    let section = match active {
        "train" => DashboardSection::Train,
        "jobs" => DashboardSection::Jobs,
        "security" | "settings" => DashboardSection::Settings,
        _ => DashboardSection::Home,
    };
    render_dashboard_sidebar(section)
}

fn dashboard_settings_tabs(active: &str) -> String {
    let tab = match active {
        "provider" => SettingsTab::Providers,
        "payment" => SettingsTab::Payment,
        _ => SettingsTab::Account,
    };
    render_settings_tabs(tab)
}

pub fn render_dev_ui() -> String {
    render_dev_ui_showcase()
}

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

pub fn render_dashboard_overview(email: &str) -> String {
    let topbar = app_shell_topbar("Dashboard", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("home");
    let bottom_nav = render_dashboard_bottom_nav(DashboardSection::Home);
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">
    {sidebar}
    <section class="glass-card rounded-[22px] p-5">
      <h2 class="text-lg font-semibold txt-strong">Operational summary</h2>
      <div id="dashboard-summary" class="mt-4 space-y-2">
        <div class="summary-row"><span>Total jobs</span><span>--</span></div>
        <div class="summary-row"><span>Queued</span><span>--</span></div>
        <div class="summary-row"><span>Running</span><span>--</span></div>
        <div class="summary-row"><span>Failed</span><span>--</span></div>
      </div>
      <div class="action-group" data-action-group="single">
        <a
          href="/dashboard/jobs"
          class="btn-primary inline-flex h-12 w-full items-center justify-center"
          data-action-role="primary"
        >
          View Jobs
        </a>
      </div>
    </section>
  </div>
</main>
{bottom_nav}

<script>
(() => {{
  const summary = document.getElementById('dashboard-summary');
  fetch('/api/dashboard/summary', {{ headers: {{ Accept: 'application/json' }} }})
    .then((res) => res.json().then((json) => [res.ok, json]))
    .then(([ok, data]) => {{
      if (!ok) {{
        summary.innerHTML = `<div class="error-card">Failed to load summary. request_id: ${{data.request_id || 'n/a'}}</div>`;
        return;
      }}
      summary.innerHTML = `
        <div class="summary-row"><span>Total jobs</span><span>${{data.total_jobs}}</span></div>
        <div class="summary-row"><span>Queued</span><span>${{data.queued_jobs}}</span></div>
        <div class="summary-row"><span>Running</span><span>${{data.running_jobs}}</span></div>
        <div class="summary-row"><span>Failed</span><span>${{data.failed_jobs}}</span></div>
        <div class="support-row">Support code: ${{data.support_request_id}}</div>`;
    }})
    .catch((err) => {{
      summary.innerHTML = `<div class="error-card">${{String(err)}}</div>`;
    }});
}})();
</script>"#
    )
}

pub fn render_dashboard_jobs(email: &str) -> String {
    let topbar = app_shell_topbar("Jobs", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("jobs");
    let bottom_nav = render_dashboard_bottom_nav(DashboardSection::Jobs);
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">
    {sidebar}
    <section class="glass-card rounded-[22px] p-5">
      <h2 class="text-lg font-semibold txt-strong">My runtime jobs</h2>
      <div id="jobs-list" class="mt-4 space-y-2"><div class="loading-card">Loading jobs...</div></div>
    </section>
  </div>
</main>
{bottom_nav}
<script>
(() => {{
  const list = document.getElementById('jobs-list');
  const renderJobs = (jobs) => {{
    if (!jobs.length) {{
      list.innerHTML = '<div class="empty-card">No jobs available.</div>';
      return;
    }}
    list.innerHTML = jobs.map((job) => `
      <a class="summary-card" href="/dashboard/jobs/${{job.job_id}}">
        <div class="summary-row"><span>Job</span><span>${{job.job_id}}</span></div>
        <div class="summary-row"><span>Status</span><span class="badge">${{job.status}}</span></div>
      </a>
    `).join('');
  }};
  fetch('/api/dashboard/jobs', {{ headers: {{ Accept: 'application/json' }} }})
    .then((res) => res.json().then((json) => [res.ok, json]))
    .then(([ok, data]) => {{
      if (!ok) {{
        list.innerHTML = `<div class="error-card">Failed to load jobs. request_id: ${{data.request_id || 'n/a'}}</div>`;
        return;
      }}
      renderJobs(data.jobs || []);
    }})
    .catch((err) => {{
      list.innerHTML = `<div class="error-card">${{String(err)}}</div>`;
    }});
}})();
</script>"#
    )
}

pub fn render_dashboard_job_detail(email: &str, job_id: &str) -> String {
    let topbar = app_shell_topbar(
        "Job detail",
        &format!("{} · {}", html_escape(email), html_escape(job_id)),
    );
    let sidebar = dashboard_desktop_sidebar("jobs");
    let escaped_job_id = html_escape(job_id);
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-28 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">
    {sidebar}
    <section class="glass-card rounded-[22px] p-5">
      <div class="summary-row"><span>Job ID</span><span id="job-id">{escaped_job_id}</span></div>
      <div class="summary-row"><span>Status</span><span id="job-status">--</span></div>
      <div id="events" class="mt-4 space-y-2"></div>
    </section>
  </div>
</main>
<div class="action-sticky" data-action-group="sticky">
  <a
    href="/dashboard/jobs"
    class="btn-ghost h-12 w-full text-center"
    data-action-role="secondary"
  >
    Back
  </a>
  <button
    id="manual-refresh"
    class="btn-primary h-12 w-full"
    data-action-role="primary"
  >
    Refresh
  </button>
</div>
<script>
(() => {{
  const jobId = document.getElementById('job-id').textContent.trim();
  const statusEl = document.getElementById('job-status');
  const eventsEl = document.getElementById('events');
  const refreshBtn = document.getElementById('manual-refresh');
  const EVENT_PAGE_LIMIT = 100;
  const MAX_RENDERED_EVENTS = 200;
  let eventsCursor = null;
  let fallbackInterval = null;

  const encodeCursor = (afterId) => {{
    try {{
      const payload = JSON.stringify({{ v: 1, job_id: jobId, after_id: Number(afterId) }});
      return btoa(payload).replace(/\\+/g, '-').replace(/\\//g, '_').replace(/=+$/g, '');
    }} catch (_err) {{
      return null;
    }}
  }};

  const trimRenderedEvents = () => {{
    while (eventsEl.children.length > MAX_RENDERED_EVENTS) {{
      eventsEl.removeChild(eventsEl.lastElementChild);
    }}
  }};

  const renderEvents = (events) => {{
    if (!events.length && !eventsEl.innerHTML.trim()) {{
      eventsEl.innerHTML = '<div class="empty-card">No events yet.</div>';
      return;
    }}
    if (events.length && eventsEl.querySelector('.empty-card')) {{
      eventsEl.innerHTML = '';
    }}
    events.forEach((event) => {{
      const eventId = Number(event.id || 0);
      if (eventId > 0) {{
        const encoded = encodeCursor(eventId);
        if (encoded) eventsCursor = encoded;
      }}
      const node = document.createElement('div');
      node.className = 'summary-card';
      node.innerHTML = `<div class="summary-row"><span>${{event.event_type}}</span><span>${{event.id}}</span></div>`;
      eventsEl.prepend(node);
    }});
    trimRenderedEvents();
  }};

  const loadJob = () => fetch(`/api/dashboard/jobs/${{jobId}}`, {{ headers: {{ Accept: 'application/json' }} }})
    .then((res) => res.json().then((json) => [res.ok, json]))
    .then(([ok, data]) => {{
      if (!ok) {{
        statusEl.textContent = `error (request_id: ${{data.request_id || 'n/a'}})`;
        return;
      }}
      statusEl.textContent = data.status;
    }});

  const fetchEventsPage = () => {{
    const params = new URLSearchParams();
    params.set('limit', String(EVENT_PAGE_LIMIT));
    if (eventsCursor) {{
      params.set('cursor', eventsCursor);
    }}
    return fetch(`/api/dashboard/jobs/${{jobId}}/events?${{params.toString()}}`, {{ headers: {{ Accept: 'application/json' }} }})
      .then((res) => res.json().then((json) => [res.ok, json]))
      .then(([ok, data]) => {{
        if (!ok) throw new Error('failed to load events');
        return data;
      }});
  }};

  const applyEventPage = (data) => {{
    renderEvents(Array.isArray(data.items) ? data.items : []);
    const page = data.page || {{}};
    if (typeof page.next_cursor === 'string' && page.next_cursor.trim()) {{
      eventsCursor = page.next_cursor;
    }}
    return Boolean(page.has_more);
  }};

  const drainEvents = async () => {{
    try {{
      for (;;) {{
        const page = await fetchEventsPage();
        if (!applyEventPage(page)) break;
      }}
    }} catch (_err) {{}}
  }};

  const startFallback = () => {{
    if (fallbackInterval) return;
    fallbackInterval = setInterval(() => {{
      void drainEvents();
    }}, 10000);
  }};

  const startSse = () => {{
    if (!window.EventSource) {{
      startFallback();
      return;
    }}
    const params = new URLSearchParams();
    params.set('limit', String(EVENT_PAGE_LIMIT));
    if (eventsCursor) {{
      params.set('cursor', eventsCursor);
    }}
    const source = new EventSource(`/api/dashboard/jobs/${{jobId}}/events/stream?${{params.toString()}}`);
    source.addEventListener('job_event', (event) => {{
      try {{
        const payload = JSON.parse(event.data);
        renderEvents([payload]);
      }} catch (_err) {{}}
    }});
    source.onerror = () => {{
      source.close();
      startFallback();
    }};
  }};

  refreshBtn.addEventListener('click', () => {{
    loadJob();
    void drainEvents();
  }});

  loadJob();
  void drainEvents();
  startSse();
}})();
</script>"#
    )
}

pub fn render_dashboard_settings(email: &str) -> String {
    let topbar = app_shell_topbar("Settings", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("security");
    let settings_tabs = dashboard_settings_tabs("account");
    let bottom_nav = render_dashboard_bottom_nav(DashboardSection::Settings);
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">
    {sidebar}
    <section class="space-y-4">
      {settings_tabs}
      <div class="space-y-4">
        <section class="summary-card p-4">
          <div class="flex items-center justify-between gap-2">
            <h3 class="text-sm font-semibold txt-strong">Passkeys</h3>
            <button
              id="passkey-create"
              type="button"
              class="btn-chip inline-flex h-9 w-9 items-center justify-center rounded-xl p-0"
              aria-label="Create passkey"
              title="Create passkey"
            >
              <img class="h-4 w-4 icon-muted" src="/assets/icons/runtime-ui/icon-plus-btn-chip-light.svgz" data-svgz-light="/assets/icons/runtime-ui/icon-plus-btn-chip-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/icon-plus-btn-chip-dark.svgz" alt="" aria-hidden="true" />
            </button>
          </div>
          <div id="passkey-status" class="mt-3 hidden"></div>
          <div id="passkeys-list" class="mt-4 space-y-2"><div class="loading-card">Loading passkeys...</div></div>
        </section>

        <section class="summary-card p-4">
          <h3 class="text-sm font-semibold txt-strong">Change password</h3>
          <p class="mt-1 text-sm txt-supporting">Use upper/lowercase letters, numbers, and symbols.</p>
          <form id="password-change-form" class="mt-4 space-y-3">
            <label class="field-label" for="new-password">New password</label>
            <input id="new-password" type="password" autocomplete="new-password" class="field-input h-12 w-full" />
            <label class="field-label" for="confirm-password">Confirm new password</label>
            <input id="confirm-password" type="password" autocomplete="new-password" class="field-input h-12 w-full" />
            <div class="space-y-1">
              <div class="password-strength-track">
                <div id="password-strength-fill" class="password-strength-fill"></div>
              </div>
              <p id="password-strength-text" class="text-xs txt-supporting">Strength: weak</p>
              <p id="password-match-hint" class="text-xs txt-supporting">Passwords must match.</p>
            </div>
            <div class="action-group" data-action-group="single">
              <button
                type="submit"
                class="btn-primary h-12 w-full"
                data-action-role="primary"
              >
                Update password
              </button>
            </div>
          </form>
          <div id="password-change-status" class="mt-3 hidden"></div>
        </section>
      </div>

      <div id="security-modal" class="app-modal-backdrop hidden" aria-hidden="true">
        <div class="app-modal-card" role="dialog" aria-modal="true" aria-labelledby="security-modal-title">
          <h4 id="security-modal-title" class="text-base font-semibold txt-strong"></h4>
          <p id="security-modal-message" class="mt-2 text-sm txt-supporting"></p>
          <div id="security-modal-input-wrap" class="mt-3 hidden">
            <label id="security-modal-input-label" class="field-label" for="security-modal-input">Current password</label>
            <input id="security-modal-input" type="password" autocomplete="current-password" class="field-input h-12 w-full" />
          </div>
          <div class="action-pair mt-4" data-action-group="pair">
            <button
              id="security-modal-cancel"
              type="button"
              class="btn-ghost h-11 w-full"
              data-action-role="secondary"
            >
              Cancel
            </button>
            <button
              id="security-modal-confirm"
              type="button"
              class="btn-primary h-11 w-full"
              data-action-role="primary"
            >
              Confirm
            </button>
          </div>
        </div>
      </div>

      <div id="passkey-modal" class="app-modal-backdrop hidden" aria-hidden="true">
        <div class="app-modal-card" role="dialog" aria-modal="true" aria-labelledby="passkey-modal-title">
          <div class="flex items-center justify-between gap-2">
            <h4 id="passkey-modal-title" class="text-base font-semibold txt-strong">Edit passkey</h4>
            <button id="passkey-modal-delete" type="button" class="btn-destructive h-9 w-9 p-0" aria-label="Delete passkey">
              <img class="h-4 w-4" src="/assets/icons/runtime-ui/icon-trash-destructive-light.svgz" data-svgz-light="/assets/icons/runtime-ui/icon-trash-destructive-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/icon-trash-destructive-dark.svgz" alt="" aria-hidden="true" />
            </button>
          </div>
          <div class="mt-3 space-y-3">
            <div>
              <label class="field-label" for="passkey-modal-label-input">Label</label>
              <input id="passkey-modal-label-input" type="text" maxlength="80" class="field-input h-11 w-full" />
            </div>
            <div>
              <label class="field-label" for="passkey-modal-credential-id">Credential ID</label>
              <input id="passkey-modal-credential-id" type="text" readonly class="field-input h-11 w-full" />
            </div>
            <div class="summary-row">
              <span>Last used</span>
              <span id="passkey-modal-last-used" class="txt-supporting">Never</span>
            </div>
            <div class="summary-row">
              <span>Created</span>
              <span id="passkey-modal-created-at" class="txt-supporting">-</span>
            </div>
            <div class="summary-row">
              <span>AAGUID</span>
              <span id="passkey-modal-aaguid" class="txt-supporting">Unknown</span>
            </div>
            <div class="summary-row">
              <span>Backup eligible (BE)</span>
              <span id="passkey-modal-be" class="txt-supporting">Unknown</span>
            </div>
            <div class="summary-row">
              <span>Backup state (BS)</span>
              <span id="passkey-modal-bs" class="txt-supporting">Unknown</span>
            </div>
          </div>
          <div class="action-pair mt-4" data-action-group="pair">
            <button
              id="passkey-modal-close"
              type="button"
              class="btn-ghost h-11 w-full"
              data-action-role="secondary"
            >
              Close
            </button>
            <button
              id="passkey-modal-save"
              type="button"
              class="btn-primary h-11 w-full"
              data-action-role="primary"
            >
              Save label
            </button>
          </div>
        </div>
      </div>
    </section>
  </div>
</main>
{bottom_nav}
<script>
(() => {{
  const list = document.getElementById('passkeys-list');
  const passkeyStatus = document.getElementById('passkey-status');
  const passkeyCreateButton = document.getElementById('passkey-create');
  const passkeyModal = document.getElementById('passkey-modal');
  const passkeyModalLabelInput = document.getElementById('passkey-modal-label-input');
  const passkeyModalCredentialInput = document.getElementById('passkey-modal-credential-id');
  const passkeyModalLastUsed = document.getElementById('passkey-modal-last-used');
  const passkeyModalCreatedAt = document.getElementById('passkey-modal-created-at');
  const passkeyModalAaguid = document.getElementById('passkey-modal-aaguid');
  const passkeyModalBe = document.getElementById('passkey-modal-be');
  const passkeyModalBs = document.getElementById('passkey-modal-bs');
  const passkeyModalCloseButton = document.getElementById('passkey-modal-close');
  const passkeyModalSaveButton = document.getElementById('passkey-modal-save');
  const passkeyModalDeleteButton = document.getElementById('passkey-modal-delete');
  const passwordForm = document.getElementById('password-change-form');
  const newPasswordInput = document.getElementById('new-password');
  const confirmPasswordInput = document.getElementById('confirm-password');
  const strengthFill = document.getElementById('password-strength-fill');
  const strengthText = document.getElementById('password-strength-text');
  const matchHint = document.getElementById('password-match-hint');
  const passwordStatus = document.getElementById('password-change-status');
  const securityModal = document.getElementById('security-modal');
  const securityModalTitle = document.getElementById('security-modal-title');
  const securityModalMessage = document.getElementById('security-modal-message');
  const securityModalInputWrap = document.getElementById('security-modal-input-wrap');
  const securityModalInput = document.getElementById('security-modal-input');
  const securityModalCancel = document.getElementById('security-modal-cancel');
  const securityModalConfirm = document.getElementById('security-modal-confirm');
  let selectedCredentialId = null;
  let passkeysById = new Map();
  let modalResolver = null;
  const PASSKEY_LABEL_OVERRIDES_KEY = 'bominal.passkey.label_overrides.v1';
  const passkeyLabelOverrides = (() => {{
    try {{
      const raw = window.localStorage.getItem(PASSKEY_LABEL_OVERRIDES_KEY);
      if (!raw) {{
        return new Map();
      }}
      const parsed = JSON.parse(raw);
      if (!parsed || typeof parsed !== 'object') {{
        return new Map();
      }}
      return new Map(Object.entries(parsed).filter(([_, value]) => typeof value === 'string'));
    }} catch (_err) {{
      return new Map();
    }}
  }})();
  const currentThemeMode = () => document.body?.dataset?.themeMode === 'dark' ? 'dark' : 'light';
  const syncThemedSvgzIcons = () => {{
    const mode = currentThemeMode();
    const icons = document.querySelectorAll('img[data-svgz-light][data-svgz-dark]');
    icons.forEach((icon) => {{
      const nextSrc = mode === 'dark' ? icon.dataset.svgzDark : icon.dataset.svgzLight;
      if (!nextSrc || icon.getAttribute('src') === nextSrc) return;
      icon.setAttribute('src', nextSrc);
    }});
  }};

  const requestJson = (url, method, payload) => fetch(url, {{
    method,
    headers: {{ 'Content-Type': 'application/json', 'Accept': 'application/json' }},
    body: payload ? JSON.stringify(payload) : undefined,
  }}).then(async (res) => {{
    const text = await res.text();
    let body = null;
    try {{ body = text ? JSON.parse(text) : null; }} catch (_err) {{}}
    return {{ ok: res.ok, status: res.status, body }};
  }});

  const showStatus = (node, message, kind) => {{
    node.classList.remove('hidden');
    if (kind === 'error') {{
      node.className = 'mt-3 error-card';
    }} else if (kind === 'success') {{
      node.className = 'mt-3 summary-card';
    }} else {{
      node.className = 'mt-3 loading-card';
    }}
    node.textContent = message;
  }};

  const clearStatus = (node) => {{
    node.className = 'mt-3 hidden';
    node.textContent = '';
  }};

  const b64urlToBuffer = (value) => {{
    const padded = (value + '==='.slice((value.length + 3) % 4)).replace(/-/g, '+').replace(/_/g, '/');
    const binary = atob(padded);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i);
    return bytes.buffer;
  }};

  const bufferToB64url = (buffer) => {{
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.length; i += 1) binary += String.fromCharCode(bytes[i]);
    return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  }};

  const escapeHtml = (value) => String(value || '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');

  const formatTimestamp = (value) => {{
    if (!value) return 'Never';
    const parsed = new Date(value);
    if (Number.isNaN(parsed.getTime())) return String(value);
    return parsed.toLocaleString();
  }};

  const formatBool = (value) => {{
    if (value === true) return 'Yes';
    if (value === false) return 'No';
    return 'Unknown';
  }};

  const detectDeviceLabel = () => {{
    const ua = navigator.userAgent || '';
    const platformRaw = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || '';

    const platform = (() => {{
      const source = `${{platformRaw}} ${{ua}}`;
      if (/iPhone|iPad|iPod/i.test(source)) return 'iOS';
      if (/Android/i.test(source)) return 'Android';
      if (/Mac/i.test(source)) return 'Mac';
      if (/Win/i.test(source)) return 'Windows';
      if (/Linux/i.test(source)) return 'Linux';
      return 'Device';
    }})();

    const browser = (() => {{
      if (/Edg\\//i.test(ua)) return 'Edge';
      if (/Firefox\\//i.test(ua)) return 'Firefox';
      if (/Chrome\\//i.test(ua) && !/Edg\\//i.test(ua)) return 'Chrome';
      if (/Safari\\//i.test(ua) && !/Chrome\\//i.test(ua)) return 'Safari';
      return 'Browser';
    }})();

    return `${{platform}} ${{browser}}`;
  }};

  const safeDetectDeviceLabel = () => {{
    try {{
      return detectDeviceLabel();
    }} catch (_err) {{
      return 'This device';
    }}
  }};

  const persistPasskeyLabelOverrides = () => {{
    try {{
      const serializable = Object.fromEntries(passkeyLabelOverrides.entries());
      window.localStorage.setItem(PASSKEY_LABEL_OVERRIDES_KEY, JSON.stringify(serializable));
    }} catch (_err) {{}}
  }};

  const effectivePasskeyLabel = (passkey) => {{
    if (!passkey || !passkey.credential_id) {{
      return 'Unnamed passkey';
    }}
    const override = passkeyLabelOverrides.get(passkey.credential_id);
    if (override && override.trim()) {{
      return override.trim();
    }}
    if (passkey.friendly_name && String(passkey.friendly_name).trim()) {{
      return String(passkey.friendly_name).trim();
    }}
    return 'Unnamed passkey';
  }};

  const serializeRegisterCredential = (credential) => {{
    const response = credential.response;
    const transports = typeof response.getTransports === 'function' ? response.getTransports() : null;
    const serializedResponse = {{
      attestationObject: bufferToB64url(response.attestationObject),
      clientDataJSON: bufferToB64url(response.clientDataJSON),
    }};
    if (Array.isArray(transports) && transports.length) {{
      serializedResponse.transports = transports;
    }}
    return {{
      id: credential.id,
      rawId: bufferToB64url(credential.rawId),
      type: credential.type,
      response: serializedResponse,
      clientExtensionResults: credential.getClientExtensionResults ? credential.getClientExtensionResults() : {{}},
    }};
  }};

  const scorePassword = (value) => {{
    let score = 0;
    if (value.length >= 10) score += 1;
    if (value.length >= 14) score += 1;
    if (/[a-z]/.test(value)) score += 1;
    if (/[A-Z]/.test(value)) score += 1;
    if (/[0-9]/.test(value)) score += 1;
    if (/[^A-Za-z0-9]/.test(value)) score += 1;
    return score;
  }};

  const renderStrength = () => {{
    const score = scorePassword(newPasswordInput.value);
    const percent = Math.round((score / 6) * 100);
    strengthFill.style.width = `${{percent}}%`;
    strengthFill.classList.remove('bg-rose-400', 'bg-amber-400', 'bg-emerald-500');
    if (score <= 2) {{
      strengthFill.classList.add('bg-rose-400');
      strengthText.textContent = 'Strength: weak';
    }} else if (score <= 4) {{
      strengthFill.classList.add('bg-amber-400');
      strengthText.textContent = 'Strength: medium';
    }} else {{
      strengthFill.classList.add('bg-emerald-500');
      strengthText.textContent = 'Strength: strong';
    }}
    return score;
  }};

  const renderMatch = () => {{
    const confirmValue = confirmPasswordInput.value;
    const matches = newPasswordInput.value === confirmValue && confirmValue.length > 0;
    matchHint.classList.remove('txt-critical', 'txt-positive');
    if (!confirmValue) {{
      matchHint.textContent = 'Passwords must match.';
      return false;
    }}
    if (matches) {{
      matchHint.textContent = 'Passwords match.';
      matchHint.classList.add('txt-positive');
      return true;
    }}
    matchHint.textContent = 'Passwords do not match.';
    matchHint.classList.add('txt-critical');
    return false;
  }};

  const MODAL_LAYER_BASE = 70;
  let modalLayerCounter = 0;

  const bringModalToBody = (modalNode) => {{
    if (!modalNode || !document.body) {{
      return;
    }}
    if (modalNode.parentElement !== document.body) {{
      document.body.appendChild(modalNode);
    }}
  }};

  const openModalLayer = (modalNode) => {{
    if (!modalNode) {{
      return;
    }}
    bringModalToBody(modalNode);
    modalLayerCounter += 1;
    modalNode.style.zIndex = String(MODAL_LAYER_BASE + modalLayerCounter);
    modalNode.classList.remove('hidden');
    modalNode.setAttribute('aria-hidden', 'false');
  }};

  const closeModalLayer = (modalNode) => {{
    if (!modalNode) {{
      return;
    }}
    modalNode.classList.add('hidden');
    modalNode.setAttribute('aria-hidden', 'true');
    modalNode.style.removeProperty('z-index');
    if (!document.querySelector('.app-modal-backdrop:not(.hidden)')) {{
      modalLayerCounter = 0;
    }}
  }};

  const topVisibleModal = () => {{
    const visible = Array.from(document.querySelectorAll('.app-modal-backdrop:not(.hidden)'));
    if (!visible.length) {{
      return null;
    }}
    return visible.reduce((currentTop, candidate) => {{
      const currentZ = Number(currentTop.style.zIndex || MODAL_LAYER_BASE);
      const candidateZ = Number(candidate.style.zIndex || MODAL_LAYER_BASE);
      return candidateZ >= currentZ ? candidate : currentTop;
    }});
  }};

  const closeSecurityModal = (result) => {{
    if (!modalResolver || !securityModal) {{
      return;
    }}
    const resolve = modalResolver;
    modalResolver = null;
    closeModalLayer(securityModal);
    if (securityModalInput) {{
      securityModalInput.value = '';
    }}
    resolve(result);
  }};

  const openSecurityModal = (options) => new Promise((resolve) => {{
    if (!securityModal || !securityModalTitle || !securityModalMessage || !securityModalInputWrap || !securityModalInput || !securityModalCancel || !securityModalConfirm) {{
      resolve({{ confirmed: false, value: '' }});
      return;
    }}

    securityModalTitle.textContent = options.title || 'Confirm action';
    securityModalMessage.textContent = options.message || '';
    securityModalConfirm.textContent = options.confirmText || 'Confirm';
    securityModalInputWrap.classList.toggle('hidden', !options.withPassword);
    securityModalInput.value = '';
    openModalLayer(securityModal);
    modalResolver = resolve;

    requestAnimationFrame(() => {{
      if (options.withPassword) {{
        securityModalInput.focus();
      }} else {{
        securityModalConfirm.focus();
      }}
    }});
  }});

  const closePasskeyModal = () => {{
    if (!passkeyModal) {{
      return;
    }}
    closeModalLayer(passkeyModal);
  }};

  const openPasskeyModal = () => {{
    const selected = selectedCredentialId ? passkeysById.get(selectedCredentialId) : null;
    if (!selected) {{
      showStatus(passkeyStatus, 'Select a passkey to edit.', 'error');
      return;
    }}
    if (!passkeyModal || !passkeyModalLabelInput || !passkeyModalCredentialInput || !passkeyModalLastUsed || !passkeyModalCreatedAt || !passkeyModalAaguid || !passkeyModalBe || !passkeyModalBs) {{
      showStatus(passkeyStatus, 'Passkey editor is unavailable.', 'error');
      return;
    }}

    passkeyModalLabelInput.value = effectivePasskeyLabel(selected);
    passkeyModalCredentialInput.value = selected.credential_id || '';
    passkeyModalLastUsed.textContent = selected.last_used_at ? formatTimestamp(selected.last_used_at) : 'Never';
    passkeyModalCreatedAt.textContent = formatTimestamp(selected.created_at);
    passkeyModalAaguid.textContent = selected.aaguid || 'Unknown';
    passkeyModalBe.textContent = formatBool(selected.backup_eligible);
    passkeyModalBs.textContent = formatBool(selected.backup_state);
    openModalLayer(passkeyModal);
    requestAnimationFrame(() => {{
      passkeyModalLabelInput.focus();
      passkeyModalLabelInput.select();
    }});
  }};

  const syncPasskeySelection = () => {{
    let selectedExists = false;
    list.querySelectorAll('[data-credential-id]').forEach((card) => {{
      const selected = card.dataset.credentialId === selectedCredentialId;
      card.classList.toggle('passkey-card-selected', selected);
      card.setAttribute('aria-pressed', selected ? 'true' : 'false');
      if (selected) {{
        selectedExists = true;
      }}
    }});
    if (!selectedExists) {{
      selectedCredentialId = null;
    }}
  }};

  const load = () => fetch('/api/auth/passkeys', {{ headers: {{ Accept: 'application/json' }} }})
    .then((res) => res.json().then((json) => [res.ok, json]))
    .then(([ok, data]) => {{
      if (!ok) {{
        passkeysById = new Map();
        selectedCredentialId = null;
        syncPasskeySelection();
        list.innerHTML = `<div class="error-card">Failed to load passkeys. request_id: ${{data.request_id || 'n/a'}}</div>`;
        return;
      }}
      const passkeys = data.passkeys || [];
      passkeysById = new Map(passkeys.map((item) => [item.credential_id, item]));
      if (!passkeys.length) {{
        selectedCredentialId = null;
        syncPasskeySelection();
        list.innerHTML = '<div class="empty-card">No passkeys registered.</div>';
        return;
      }}
      if (selectedCredentialId && !passkeys.some((item) => item.credential_id === selectedCredentialId)) {{
        selectedCredentialId = null;
      }}
      list.innerHTML = passkeys.map((item) => `
        <button type="button" class="summary-card passkey-card w-full text-left" data-credential-id="${{item.credential_id}}" aria-pressed="false">
          <div class="flex items-center justify-between gap-2 text-sm">
            <span>${{escapeHtml(effectivePasskeyLabel(item))}}</span>
            <span class="txt-supporting">${{item.last_used_at ? `Used ${{formatTimestamp(item.last_used_at)}}` : 'Never used'}}</span>
          </div>
        </button>
      `).join('');
      list.querySelectorAll('[data-credential-id]').forEach((card) => {{
        card.addEventListener('click', () => {{
          selectedCredentialId = card.dataset.credentialId || null;
          syncPasskeySelection();
          openPasskeyModal();
        }});
      }});
      syncPasskeySelection();
    }});

  const deleteSelectedPasskey = async () => {{
    if (!selectedCredentialId) {{
      return;
    }}
    const modalResult = await openSecurityModal({{
      title: 'Delete passkey',
      message: `Delete selected passkey (${{selectedCredentialId}})? This action cannot be undone.`,
      confirmText: 'Delete',
      withPassword: false,
    }});
    if (!modalResult.confirmed) {{
      return;
    }}
    clearStatus(passkeyStatus);
    if (passkeyModalDeleteButton) {{
      passkeyModalDeleteButton.disabled = true;
    }}
    const result = await requestJson(`/api/auth/passkeys/${{selectedCredentialId}}`, 'DELETE');
    if (!result.ok) {{
      const requestId = result.body && result.body.request_id ? ` (request_id: ${{result.body.request_id}})` : '';
      showStatus(passkeyStatus, `Passkey deletion failed${{requestId}}`, 'error');
      syncPasskeySelection();
      if (passkeyModalDeleteButton) {{
        passkeyModalDeleteButton.disabled = false;
      }}
      return;
    }}
    passkeyLabelOverrides.delete(selectedCredentialId);
    persistPasskeyLabelOverrides();
    selectedCredentialId = null;
    closePasskeyModal();
    showStatus(passkeyStatus, 'Passkey deleted successfully.', 'success');
    await load();
    if (passkeyModalDeleteButton) {{
      passkeyModalDeleteButton.disabled = false;
    }}
  }};

  const saveSelectedPasskeyLabel = async () => {{
    if (!selectedCredentialId || !passkeyModalLabelInput) {{
      return;
    }}
    const friendlyName = passkeyModalLabelInput.value.trim();
    if (!friendlyName) {{
      showStatus(passkeyStatus, 'Passkey label is required.', 'error');
      passkeyModalLabelInput.focus();
      return;
    }}
    clearStatus(passkeyStatus);
    if (passkeyModalSaveButton) {{
      passkeyModalSaveButton.disabled = true;
    }}
    passkeyLabelOverrides.set(selectedCredentialId, friendlyName);
    persistPasskeyLabelOverrides();
    showStatus(passkeyStatus, 'Passkey label updated for this browser.', 'success');
    syncPasskeySelection();
    await load();
    if (passkeyModalSaveButton) {{
      passkeyModalSaveButton.disabled = false;
    }}
  }};

  const createPasskey = async () => {{
    clearStatus(passkeyStatus);
    if (!window.PublicKeyCredential || !navigator.credentials) {{
      showStatus(passkeyStatus, 'Passkeys are unavailable in this browser.', 'error');
      return;
    }}

    if (passkeyCreateButton) {{
      passkeyCreateButton.disabled = true;
    }}
    try {{
      const friendlyName = safeDetectDeviceLabel();
      const start = await requestJson('/api/auth/passkeys/register/start', 'POST', {{
        friendly_name: friendlyName,
      }});
      if (!start.ok || !start.body || !start.body.options || !start.body.flow_id) {{
        const requestId = start.body && start.body.request_id ? ` (request_id: ${{start.body.request_id}})` : '';
        showStatus(passkeyStatus, `Passkey setup failed${{requestId}}`, 'error');
        return;
      }}

      const options = typeof structuredClone === 'function'
        ? structuredClone(start.body.options)
        : JSON.parse(JSON.stringify(start.body.options));
      options.publicKey.challenge = b64urlToBuffer(options.publicKey.challenge);
      if (options.publicKey.user && options.publicKey.user.id) {{
        options.publicKey.user.id = b64urlToBuffer(options.publicKey.user.id);
      }}
      if (Array.isArray(options.publicKey.excludeCredentials)) {{
        options.publicKey.excludeCredentials = options.publicKey.excludeCredentials.map((item) => ({{ ...item, id: b64urlToBuffer(item.id) }}));
      }}

      const credential = await navigator.credentials.create(options);
      if (!credential) {{
        showStatus(passkeyStatus, 'Passkey creation was cancelled.', 'error');
        return;
      }}

      const finish = await requestJson('/api/auth/passkeys/register/finish', 'POST', {{
        flow_id: start.body.flow_id,
        credential: serializeRegisterCredential(credential),
      }});
      if (!finish.ok) {{
        const requestId = finish.body && finish.body.request_id ? ` (request_id: ${{finish.body.request_id}})` : '';
        showStatus(passkeyStatus, `Passkey setup failed${{requestId}}`, 'error');
        return;
      }}

      showStatus(passkeyStatus, 'Passkey created successfully.', 'success');
      load();
    }} catch (err) {{
      const errName = err && typeof err === 'object' && err.name ? String(err.name) : '';
      if (err && typeof err === 'object' && err.name === 'SecurityError') {{
        showStatus(passkeyStatus, 'Passkeys are unavailable on this host. Use localhost for WebAuthn.', 'error');
      }} else if (err && typeof err === 'object' && err.name === 'NotAllowedError') {{
        showStatus(passkeyStatus, 'Passkey creation was cancelled or blocked by the browser.', 'error');
      }} else if (err && typeof err === 'object' && err.name === 'InvalidStateError') {{
        showStatus(passkeyStatus, 'This authenticator is already registered for your account.', 'error');
      }} else {{
        const suffix = errName ? ` (${{errName}})` : '';
        showStatus(passkeyStatus, `Passkey creation failed${{suffix}}.`, 'error');
      }}
    }} finally {{
      if (passkeyCreateButton) {{
        passkeyCreateButton.disabled = false;
      }}
    }}
  }};

  if (passkeyCreateButton) {{
    passkeyCreateButton.addEventListener('click', async (event) => {{
      event.preventDefault();
      const modalResult = await openSecurityModal({{
        title: 'Create passkey',
        message: 'Passkeys are faster to use and resistant to phishing because they do not require typing your password. Create a passkey on this device now?',
        confirmText: 'Continue',
        withPassword: false,
      }});
      if (!modalResult.confirmed) {{
        return;
      }}
      createPasskey();
    }});
  }}
  if (passkeyModalCloseButton) {{
    passkeyModalCloseButton.addEventListener('click', closePasskeyModal);
  }}
  if (passkeyModalSaveButton) {{
    passkeyModalSaveButton.addEventListener('click', saveSelectedPasskeyLabel);
  }}
  if (passkeyModalDeleteButton) {{
    passkeyModalDeleteButton.addEventListener('click', deleteSelectedPasskey);
  }}
  if (passkeyModal) {{
    passkeyModal.addEventListener('click', (event) => {{
      if (event.target === passkeyModal) {{
        closePasskeyModal();
      }}
    }});
  }}

  if (securityModalCancel) {{
    securityModalCancel.addEventListener('click', () => {{
      closeSecurityModal({{ confirmed: false, value: '' }});
    }});
  }}

  if (securityModalConfirm) {{
    securityModalConfirm.addEventListener('click', () => {{
      const requiresPassword = securityModalInputWrap && !securityModalInputWrap.classList.contains('hidden');
      if (requiresPassword) {{
        const value = securityModalInput.value;
        if (!value) {{
          securityModalInput.focus();
          return;
        }}
        closeSecurityModal({{ confirmed: true, value }});
        return;
      }}
      closeSecurityModal({{ confirmed: true, value: '' }});
    }});
  }}

  if (securityModal) {{
    securityModal.addEventListener('click', (event) => {{
      if (event.target === securityModal) {{
        closeSecurityModal({{ confirmed: false, value: '' }});
      }}
    }});
  }}

  document.addEventListener('keydown', (event) => {{
    if (event.key !== 'Escape') {{
      return;
    }}
    const modal = topVisibleModal();
    if (!modal) {{
      return;
    }}
    event.preventDefault();
    if (modal === securityModal) {{
      closeSecurityModal({{ confirmed: false, value: '' }});
      return;
    }}
    if (modal === passkeyModal) {{
      closePasskeyModal();
    }}
  }});

  newPasswordInput.addEventListener('input', () => {{
    renderStrength();
    renderMatch();
  }});
  confirmPasswordInput.addEventListener('input', renderMatch);

  passwordForm.addEventListener('submit', async (event) => {{
    event.preventDefault();
    clearStatus(passwordStatus);

    const score = renderStrength();
    const matches = renderMatch();
    if (!matches) {{
      showStatus(passwordStatus, 'New password and confirmation must match.', 'error');
      return;
    }}
    if (score < 4) {{
      showStatus(passwordStatus, 'Password strength is too weak. Use a longer mixed password.', 'error');
      return;
    }}

    const modalResult = await openSecurityModal({{
      title: 'Confirm password update',
      message: 'Enter your current password to apply this change.',
      confirmText: 'Update',
      withPassword: true,
    }});
    if (!modalResult.confirmed) {{
      return;
    }}
    const currentPassword = modalResult.value;
    if (!currentPassword) {{
      showStatus(passwordStatus, 'Current password is required to update password.', 'error');
      return;
    }}

    const result = await requestJson('/api/auth/password/change', 'POST', {{
      current_password: currentPassword,
      new_password: newPasswordInput.value,
    }});
    if (!result.ok) {{
      const requestId = result.body && result.body.request_id ? ` (request_id: ${{result.body.request_id}})` : '';
      showStatus(passwordStatus, `Password update failed${{requestId}}`, 'error');
      return;
    }}

    showStatus(passwordStatus, 'Password updated successfully.', 'success');
    newPasswordInput.value = '';
    confirmPasswordInput.value = '';
    renderStrength();
    renderMatch();
  }});

  renderStrength();
  renderMatch();
  syncThemedSvgzIcons();
  if (document.body && typeof MutationObserver === 'function') {{
    const themeObserver = new MutationObserver(() => syncThemedSvgzIcons());
    themeObserver.observe(document.body, {{ attributes: true, attributeFilter: ['data-theme-mode'] }});
  }}
  load();
}})();
</script>"#
    )
}

pub fn render_dashboard_train(email: &str) -> String {
    let topbar = app_shell_topbar("Train", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("train");
    let bottom_nav = render_dashboard_bottom_nav(DashboardSection::Train);

    let mut html = String::new();
    html.push_str(&topbar);
    html.push_str(
        r#"<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">"#,
    );
    html.push_str(&sidebar);
    html.push_str(
        r#"<section class="space-y-4">
      <section class="glass-card rounded-[22px] p-5">
        <h2 class="text-lg font-semibold txt-strong" data-i18n="workspace.title">Train workspace</h2>
        <p class="mt-1 text-sm txt-supporting" data-i18n="workspace.subtitle">Korail-inspired hybrid flow with modal selectors and station catalog safety checks.</p>
        <div id="train-preflight" class="mt-4 space-y-2">
          <div class="loading-card" data-i18n="preflight.loading">Loading provider readiness...</div>
        </div>
      </section>

      <section class="glass-card rounded-[22px] p-5">
        <h3 class="text-base font-semibold txt-strong" data-i18n="search.title">Search trains</h3>
        <form id="train-search-form" class="mt-3 space-y-3">
          <div class="rounded-[24px] border border-slate-200/80 bg-white/65 p-3">
            <div class="grid grid-cols-1 gap-2 md:grid-cols-[minmax(0,1fr)_44px_minmax(0,1fr)]">
              <button id="dep-station-open" type="button" class="summary-card h-14 text-left">
                <span class="text-[11px] uppercase tracking-[0.08em] txt-supporting" data-i18n="search.departure">Departure</span>
                <div id="dep-station-display" class="mt-1 text-sm font-semibold txt-strong" data-i18n="search.select_station">Select station</div>
              </button>
              <button id="station-swap" type="button" class="btn-ghost h-14 w-11 p-0" aria-label="Swap stations" data-i18n-aria-label="search.swap_stations">↔</button>
              <button id="arr-station-open" type="button" class="summary-card h-14 text-left">
                <span class="text-[11px] uppercase tracking-[0.08em] txt-supporting" data-i18n="search.arrival">Arrival</span>
                <div id="arr-station-display" class="mt-1 text-sm font-semibold txt-strong" data-i18n="search.select_station">Select station</div>
              </button>
            </div>
            <div class="mt-2 grid grid-cols-1 gap-2 md:grid-cols-3">
              <button id="dep-date-open" type="button" class="summary-card h-14 text-left">
                <span class="text-[11px] uppercase tracking-[0.08em] txt-supporting" data-i18n="search.departure_date">Departure date</span>
                <div id="dep-date-display" class="mt-1 text-sm font-semibold txt-strong" data-i18n="search.select_date">Select date</div>
              </button>
              <button id="dep-time-open" type="button" class="summary-card h-14 text-left">
                <span class="text-[11px] uppercase tracking-[0.08em] txt-supporting" data-i18n="search.departure_time">Departure time</span>
                <div id="dep-time-display" class="mt-1 text-sm font-semibold txt-strong" data-i18n="search.select_time">Select time</div>
              </button>
              <button id="passenger-open" type="button" class="summary-card h-14 text-left">
                <span class="text-[11px] uppercase tracking-[0.08em] txt-supporting" data-i18n="search.passengers">Passengers</span>
                <div id="passenger-display" class="mt-1 text-sm font-semibold txt-strong">1</div>
              </button>
            </div>
          </div>
          <div class="action-group" data-action-group="single">
            <button id="train-search-submit" type="submit" class="btn-primary h-11 w-full" data-action-role="primary" data-i18n="search.start">Start search</button>
          </div>
        </form>
        <div id="train-search-status" class="mt-3 hidden"></div>
      </section>

      <section class="glass-card rounded-[22px] p-5">
        <div class="summary-row">
          <h3 class="text-base font-semibold txt-strong" data-i18n="search.latest_result">Latest search result</h3>
          <span id="active-search-id" class="text-xs txt-supporting" data-i18n="search.none">none</span>
        </div>
        <div id="train-provider-jobs" class="mt-3 space-y-2"></div>
        <div id="train-results" class="mt-3 space-y-2"></div>
      </section>

      <section class="glass-card rounded-[22px] p-5">
        <h3 class="text-base font-semibold txt-strong" data-i18n="search.recent">Recent searches</h3>
        <div id="train-search-history" class="mt-3 space-y-2"><div class="loading-card" data-i18n="search.loading_history">Loading history...</div></div>
      </section>

      <div id="station-picker-modal" class="app-modal-backdrop hidden" aria-hidden="true">
        <div class="app-modal-card max-w-[820px]" role="dialog" aria-modal="true" aria-labelledby="station-picker-title">
          <div class="flex items-center justify-between">
            <h4 id="station-picker-title" class="text-base font-semibold txt-strong" data-i18n="station.modal_title">Station picker</h4>
            <button id="station-picker-close" type="button" class="btn-ghost h-9 w-9 p-0" aria-label="Close" data-i18n-aria-label="common.close">✕</button>
          </div>
          <div class="mt-3">
            <label class="sr-only" for="station-picker-query" data-i18n="station.search_label">Search station</label>
            <div class="relative">
              <input id="station-picker-query" type="text" autocomplete="off" class="field-input h-11 w-full pr-11 leading-none" placeholder="Search station name or initials (Seoul, ㅅㅇ)" data-i18n-placeholder="station.search_placeholder" />
              <span class="pointer-events-none absolute inset-y-0 right-3 inline-flex items-center txt-supporting" aria-hidden="true">
                <img class="h-5 w-5 icon-muted" src="/assets/icons/runtime-ui/icon-search-supporting-light.svgz" data-svgz-light="/assets/icons/runtime-ui/icon-search-supporting-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/icon-search-supporting-dark.svgz" alt="" aria-hidden="true" />
              </span>
            </div>
          </div>
          <div id="station-picker-correction" class="mt-2 hidden"></div>
          <div id="station-picker-suggestions" class="mt-3 max-h-[220px] space-y-1 overflow-y-auto"></div>
          <div class="mt-3 flex flex-wrap gap-2">
            <button type="button" id="station-tab-major" class="btn-primary h-9 px-3" data-i18n="station.tab_major">Major stations</button>
            <button type="button" id="station-tab-region" class="btn-ghost h-9 px-3" data-i18n="station.tab_region">By region</button>
          </div>
          <div id="station-picker-regions" class="mt-3 flex flex-wrap gap-2"></div>
          <div id="station-picker-list" class="mt-3 max-h-[240px] space-y-1 overflow-y-auto"></div>
        </div>
      </div>

      <div id="date-picker-modal" class="app-modal-backdrop hidden" aria-hidden="true">
        <div class="app-modal-card max-w-[540px]" role="dialog" aria-modal="true" aria-labelledby="date-picker-title">
          <div class="flex items-center justify-between">
            <h4 id="date-picker-title" class="text-base font-semibold txt-strong" data-i18n="date.modal_title">Departure date</h4>
            <button id="date-picker-close" type="button" class="btn-ghost h-9 w-9 p-0" aria-label="Close" data-i18n-aria-label="common.close">✕</button>
          </div>
          <div class="mt-3">
            <label class="field-label" for="date-picker-input" data-i18n="date.label">Date</label>
            <input id="date-picker-input" type="date" class="field-input h-11 w-full" />
          </div>
          <div class="mt-4 grid grid-cols-2 gap-2">
            <button id="date-picker-cancel" type="button" class="btn-ghost h-11 w-full" data-i18n="common.cancel">Cancel</button>
            <button id="date-picker-apply" type="button" class="btn-primary h-11 w-full" data-i18n="common.apply">Apply</button>
          </div>
        </div>
      </div>

      <div id="time-picker-modal" class="app-modal-backdrop hidden" aria-hidden="true">
        <div class="app-modal-card max-w-[540px]" role="dialog" aria-modal="true" aria-labelledby="time-picker-title">
          <div class="flex items-center justify-between">
            <h4 id="time-picker-title" class="text-base font-semibold txt-strong" data-i18n="time.modal_title">Departure time</h4>
            <button id="time-picker-close" type="button" class="btn-ghost h-9 w-9 p-0" aria-label="Close" data-i18n-aria-label="common.close">✕</button>
          </div>
          <div class="mt-3">
            <label class="field-label" data-i18n="time.hour_label">Hour</label>
            <div id="time-picker-hour-list" class="mt-2 grid grid-cols-4 gap-2 md:grid-cols-6"></div>
          </div>
          <div class="mt-3 hidden md:block">
            <label class="field-label" for="time-picker-desktop-input" data-i18n="time.desktop_label">Desktop free time input</label>
            <input id="time-picker-desktop-input" type="time" class="field-input h-11 w-full" />
          </div>
          <div class="mt-4 grid grid-cols-2 gap-2">
            <button id="time-picker-cancel" type="button" class="btn-ghost h-11 w-full" data-i18n="common.cancel">Cancel</button>
            <button id="time-picker-apply" type="button" class="btn-primary h-11 w-full" data-i18n="common.apply">Apply</button>
          </div>
        </div>
      </div>

      <div id="passenger-picker-modal" class="app-modal-backdrop hidden" aria-hidden="true">
        <div class="app-modal-card max-w-[540px]" role="dialog" aria-modal="true" aria-labelledby="passenger-picker-title">
          <div class="flex items-center justify-between">
            <h4 id="passenger-picker-title" class="text-base font-semibold txt-strong" data-i18n="passenger.modal_title">Passengers</h4>
            <button id="passenger-picker-close" type="button" class="btn-ghost h-9 w-9 p-0" aria-label="Close" data-i18n-aria-label="common.close">✕</button>
          </div>
          <div id="passenger-picker-rows" class="mt-3 space-y-2"></div>
          <div class="mt-4 grid grid-cols-2 gap-2">
            <button id="passenger-picker-cancel" type="button" class="btn-ghost h-11 w-full" data-i18n="common.cancel">Cancel</button>
            <button id="passenger-picker-apply" type="button" class="btn-primary h-11 w-full" data-i18n="common.apply">Apply</button>
          </div>
        </div>
      </div>
    </section>
  </div>
</main>
<script>
(() => {
  const preflightNode = document.getElementById('train-preflight');
  const form = document.getElementById('train-search-form');
  const statusNode = document.getElementById('train-search-status');
  const depStationOpen = document.getElementById('dep-station-open');
  const arrStationOpen = document.getElementById('arr-station-open');
  const swapButton = document.getElementById('station-swap');
  const depStationDisplay = document.getElementById('dep-station-display');
  const arrStationDisplay = document.getElementById('arr-station-display');
  const dateOpen = document.getElementById('dep-date-open');
  const dateDisplay = document.getElementById('dep-date-display');
  const timeOpen = document.getElementById('dep-time-open');
  const timeDisplay = document.getElementById('dep-time-display');
  const passengerOpen = document.getElementById('passenger-open');
  const passengerDisplay = document.getElementById('passenger-display');
  const resultsNode = document.getElementById('train-results');
  const jobsNode = document.getElementById('train-provider-jobs');
  const historyNode = document.getElementById('train-search-history');
  const activeSearchIdNode = document.getElementById('active-search-id');
  const stationModal = document.getElementById('station-picker-modal');
  const stationModalClose = document.getElementById('station-picker-close');
  const stationQuery = document.getElementById('station-picker-query');
  const stationCorrection = document.getElementById('station-picker-correction');
  const stationSuggestions = document.getElementById('station-picker-suggestions');
  const stationTabMajor = document.getElementById('station-tab-major');
  const stationTabRegion = document.getElementById('station-tab-region');
  const stationRegionsNode = document.getElementById('station-picker-regions');
  const stationListNode = document.getElementById('station-picker-list');
  const dateModal = document.getElementById('date-picker-modal');
  const dateClose = document.getElementById('date-picker-close');
  const dateInput = document.getElementById('date-picker-input');
  const dateCancel = document.getElementById('date-picker-cancel');
  const dateApply = document.getElementById('date-picker-apply');
  const timeModal = document.getElementById('time-picker-modal');
  const timeClose = document.getElementById('time-picker-close');
  const timeHourList = document.getElementById('time-picker-hour-list');
  const timeDesktopInput = document.getElementById('time-picker-desktop-input');
  const timeCancel = document.getElementById('time-picker-cancel');
  const timeApply = document.getElementById('time-picker-apply');
  const passengerModal = document.getElementById('passenger-picker-modal');
  const passengerClose = document.getElementById('passenger-picker-close');
  const passengerRows = document.getElementById('passenger-picker-rows');
  const passengerCancel = document.getElementById('passenger-picker-cancel');
  const passengerApply = document.getElementById('passenger-picker-apply');
  const currentThemeMode = () => document.body?.dataset?.themeMode === 'dark' ? 'dark' : 'light';
  const syncThemedSvgzIcons = (rootNode) => {
    const mode = currentThemeMode();
    const root = rootNode && typeof rootNode.querySelectorAll === 'function' ? rootNode : document;
    const icons = root.querySelectorAll('img[data-svgz-light][data-svgz-dark]');
    icons.forEach((icon) => {
      const nextSrc = mode === 'dark' ? icon.dataset.svgzDark : icon.dataset.svgzLight;
      if (!nextSrc || icon.getAttribute('src') === nextSrc) return;
      icon.setAttribute('src', nextSrc);
    });
  };

  const now = new Date();
  let depDate = now.toISOString().slice(0, 10);
  let depTime = String(now.getHours()).padStart(2, '0') + ':00';
  let depSelection = null;
  let arrSelection = null;
  let pollTimer = null;
  let modalLayerCounter = 0;
  const MODAL_BASE = 70;
  let stationPickerTarget = 'dep';
  let stationTab = 'major';
  let stationRegionsData = null;
  let stationQueryCounter = 0;
  let stationSuggestDebounceTimer = null;
  let activeRegionKey = 'seoul';
  let passengerDraft = { adult: 1, child: 0, senior: 0, disability_1_to_3: 0, disability_4_to_6: 0 };
  let passengerCommitted = { ...passengerDraft };
  const TRAIN_I18N = {
    en: {
      'workspace.title': 'Train workspace',
      'workspace.subtitle': 'Korail-inspired hybrid flow with modal selectors and station catalog safety checks.',
      'preflight.loading': 'Loading provider readiness...',
      'search.title': 'Search trains',
      'search.departure': 'Departure',
      'search.arrival': 'Arrival',
      'search.departure_date': 'Departure date',
      'search.departure_time': 'Departure time',
      'search.passengers': 'Passengers',
      'search.start': 'Start search',
      'search.latest_result': 'Latest search result',
      'search.recent': 'Recent searches',
      'search.loading_history': 'Loading history...',
      'search.select_station': 'Select station',
      'search.select_date': 'Select date',
      'search.select_time': 'Select time',
      'search.swap_stations': 'Swap stations',
      'search.none': 'none',
      'station.modal_title': 'Station picker',
      'station.search_label': 'Search station',
      'station.search_placeholder': 'Search station name or initials (Seoul, ㅅㅇ)',
      'station.tab_major': 'Major stations',
      'station.tab_region': 'By region',
      'station.correction_prompt': 'Did you mean {query}?',
      'date.modal_title': 'Departure date',
      'date.label': 'Date',
      'time.modal_title': 'Departure time',
      'time.hour_label': 'Hour',
      'time.desktop_label': 'Desktop free time input',
      'passenger.modal_title': 'Passengers',
      'passenger.adult': 'Adult (13+)',
      'passenger.child': 'Child (6-12)',
      'passenger.senior': 'Senior (65+)',
      'passenger.disability_1_to_3': 'Disability (level 1-3)',
      'passenger.disability_4_to_6': 'Disability (level 4-6)',
      'passenger.count.one': '{count} passenger',
      'passenger.count.other': '{count} passengers',
      'common.close': 'Close',
      'common.cancel': 'Cancel',
      'common.apply': 'Apply',
      'empty.provider_jobs': 'No provider jobs for this search.',
      'empty.results': 'No trains returned yet.',
      'empty.history': 'No searches yet.',
      'empty.stations': 'No stations in this view.',
      'status.general_available': 'General ✓',
      'status.general_unavailable': 'General ✕',
      'status.special_available': 'Special ✓',
      'status.special_unavailable': 'Special ✕',
      'history.providers': 'providers',
      'error.load_history': 'Failed to load search history.',
      'error.poll_search': 'Search polling failed.',
      'error.load_snapshot': 'Could not load search snapshot.',
      'error.load_preflight': 'Failed to load preflight.',
      'error.load_station_catalog': 'Could not load station catalog.',
      'error.station_lookup': 'Station lookup failed.',
      'error.date_required': 'Departure date is required.',
      'error.passenger_required': 'At least one passenger is required.',
      'error.station_required': 'Choose departure and arrival stations.',
      'error.search_failed': 'Search request failed.',
      'success.search_accepted': 'Search {searchId} accepted.',
      'provider.payment': 'Payment',
      'provider.credentials': 'Credentials',
      'provider.ready': 'ready',
      'provider.error': 'error',
      'provider.missing': 'missing',
    },
    ko: {
      'workspace.title': '기차 워크스페이스',
      'workspace.subtitle': 'Korail 스타일 모달 선택과 역 카탈로그 안전 검증 흐름을 제공합니다.',
      'preflight.loading': '공급자 준비 상태를 불러오는 중...',
      'search.title': '열차 조회',
      'search.departure': '출발',
      'search.arrival': '도착',
      'search.departure_date': '출발일',
      'search.departure_time': '출발시간',
      'search.passengers': '인원',
      'search.start': '열차조회',
      'search.latest_result': '최신 조회 결과',
      'search.recent': '최근 조회',
      'search.loading_history': '조회 이력을 불러오는 중...',
      'search.select_station': '역 선택',
      'search.select_date': '날짜 선택',
      'search.select_time': '시간 선택',
      'search.swap_stations': '출발/도착 교체',
      'search.none': '없음',
      'station.modal_title': '역 선택',
      'station.search_label': '역 검색',
      'station.search_placeholder': '역 이름 또는 초성 검색 (서울, ㅅㅇ)',
      'station.tab_major': '주요역',
      'station.tab_region': '지역별',
      'station.correction_prompt': '{query} 역을 찾으셨나요?',
      'date.modal_title': '출발일 선택',
      'date.label': '날짜',
      'time.modal_title': '출발시간 선택',
      'time.hour_label': '시간',
      'time.desktop_label': '데스크톱 시간 입력',
      'passenger.modal_title': '인원 선택',
      'passenger.adult': '어른(13세 이상)',
      'passenger.child': '어린이(6~12세)',
      'passenger.senior': '경로(65세 이상)',
      'passenger.disability_1_to_3': '중증 장애인',
      'passenger.disability_4_to_6': '경증 장애인',
      'passenger.count.one': '총 {count}명',
      'passenger.count.other': '총 {count}명',
      'common.close': '닫기',
      'common.cancel': '취소',
      'common.apply': '적용',
      'empty.provider_jobs': '공급자 작업이 없습니다.',
      'empty.results': '조회된 열차가 없습니다.',
      'empty.history': '조회 이력이 없습니다.',
      'empty.stations': '표시할 역이 없습니다.',
      'status.general_available': '일반석 가능',
      'status.general_unavailable': '일반석 불가',
      'status.special_available': '특실 가능',
      'status.special_unavailable': '특실 불가',
      'history.providers': '공급자',
      'error.load_history': '조회 이력을 불러오지 못했습니다.',
      'error.poll_search': '조회 상태 갱신에 실패했습니다.',
      'error.load_snapshot': '조회 스냅샷을 불러오지 못했습니다.',
      'error.load_preflight': '준비 상태를 불러오지 못했습니다.',
      'error.load_station_catalog': '역 목록을 불러오지 못했습니다.',
      'error.station_lookup': '역 검색에 실패했습니다.',
      'error.date_required': '출발일이 필요합니다.',
      'error.passenger_required': '최소 1명의 승객이 필요합니다.',
      'error.station_required': '출발역과 도착역을 선택하세요.',
      'error.search_failed': '조회 요청에 실패했습니다.',
      'success.search_accepted': '조회 {searchId} 요청이 접수되었습니다.',
      'provider.payment': '결제',
      'provider.credentials': '자격 증명',
      'provider.ready': '준비됨',
      'provider.error': '오류',
      'provider.missing': '미설정',
    },
    ja: {
      'workspace.title': '列車ワークスペース',
      'workspace.subtitle': 'Korail 風のモーダル選択と駅カタログ検証フローです。',
      'preflight.loading': 'プロバイダー準備状態を読み込み中...',
      'search.title': '列車検索',
      'search.departure': '出発',
      'search.arrival': '到着',
      'search.departure_date': '出発日',
      'search.departure_time': '出発時刻',
      'search.passengers': '人数',
      'search.start': '列車検索',
      'search.latest_result': '最新検索結果',
      'search.recent': '最近の検索',
      'search.loading_history': '検索履歴を読み込み中...',
      'search.select_station': '駅を選択',
      'search.select_date': '日付を選択',
      'search.select_time': '時刻を選択',
      'search.swap_stations': '出発/到着を入れ替え',
      'search.none': 'なし',
      'station.modal_title': '駅選択',
      'station.search_label': '駅検索',
      'station.search_placeholder': '駅名または頭子音で検索 (ソウル, ㅅㅇ)',
      'station.tab_major': '主要駅',
      'station.tab_region': '地域別',
      'station.correction_prompt': '{query} をお探しですか？',
      'date.modal_title': '出発日',
      'date.label': '日付',
      'time.modal_title': '出発時刻',
      'time.hour_label': '時',
      'time.desktop_label': 'デスクトップ時刻入力',
      'passenger.modal_title': '人数',
      'passenger.adult': '大人 (13歳以上)',
      'passenger.child': 'こども (6-12歳)',
      'passenger.senior': 'シニア (65歳以上)',
      'passenger.disability_1_to_3': '障害 (1-3級)',
      'passenger.disability_4_to_6': '障害 (4-6級)',
      'passenger.count.one': '合計 {count}名',
      'passenger.count.other': '合計 {count}名',
      'common.close': '閉じる',
      'common.cancel': 'キャンセル',
      'common.apply': '適用',
      'empty.provider_jobs': 'この検索のプロバイダージョブはありません。',
      'empty.results': '列車結果がありません。',
      'empty.history': '検索履歴がありません。',
      'empty.stations': 'この表示に駅はありません。',
      'status.general_available': '普通席 ✓',
      'status.general_unavailable': '普通席 ✕',
      'status.special_available': '特室 ✓',
      'status.special_unavailable': '特室 ✕',
      'history.providers': 'プロバイダー',
      'error.load_history': '検索履歴を読み込めませんでした。',
      'error.poll_search': '検索ポーリングに失敗しました。',
      'error.load_snapshot': '検索スナップショットを読み込めませんでした。',
      'error.load_preflight': '準備状態を読み込めませんでした。',
      'error.load_station_catalog': '駅カタログを読み込めませんでした。',
      'error.station_lookup': '駅検索に失敗しました。',
      'error.date_required': '出発日が必要です。',
      'error.passenger_required': '少なくとも1人の乗客が必要です。',
      'error.station_required': '出発駅と到着駅を選択してください。',
      'error.search_failed': '検索リクエストに失敗しました。',
      'success.search_accepted': '検索 {searchId} を受け付けました。',
      'provider.payment': '支払い',
      'provider.credentials': '認証情報',
      'provider.ready': '準備完了',
      'provider.error': 'エラー',
      'provider.missing': '未設定',
    },
  };
  const resolveLocale = () => {
    const token = String(
      document.body?.dataset?.locale || document.documentElement?.lang || 'en',
    ).trim().toLowerCase();
    const primary = token.split('-')[0];
    return primary === 'ko' || primary === 'ja' || primary === 'en' ? primary : 'en';
  };
  const activeLocale = resolveLocale();
  const t = (key, vars) => {
    const table = TRAIN_I18N[activeLocale] || TRAIN_I18N.en;
    let text = table[key] || TRAIN_I18N.en[key] || key;
    if (vars && typeof vars === 'object') {
      for (const [name, value] of Object.entries(vars)) {
        text = text.replaceAll(`{${name}}`, String(value));
      }
    }
    return text;
  };
  const applyStaticTranslations = () => {
    Array.from(document.querySelectorAll('[data-i18n]')).forEach((node) => {
      const key = node.getAttribute('data-i18n');
      if (!key) return;
      node.textContent = t(key);
    });
    Array.from(document.querySelectorAll('[data-i18n-placeholder]')).forEach((node) => {
      const key = node.getAttribute('data-i18n-placeholder');
      if (!key || !('placeholder' in node)) return;
      node.placeholder = t(key);
    });
    Array.from(document.querySelectorAll('[data-i18n-aria-label]')).forEach((node) => {
      const key = node.getAttribute('data-i18n-aria-label');
      if (!key) return;
      node.setAttribute('aria-label', t(key));
    });
  };
  const passengerKinds = [
    { key: 'adult', label: t('passenger.adult') },
    { key: 'child', label: t('passenger.child') },
    { key: 'senior', label: t('passenger.senior') },
    { key: 'disability_1_to_3', label: t('passenger.disability_1_to_3') },
    { key: 'disability_4_to_6', label: t('passenger.disability_4_to_6') },
  ];

  const escapeHtml = (value) => String(value || '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');

  const requestJson = async (url, method, payload) => {
    const response = await fetch(url, {
      method: method || 'GET',
      headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
      body: payload ? JSON.stringify(payload) : undefined,
    });
    const text = await response.text();
    let body = null;
    try { body = text ? JSON.parse(text) : null; } catch (_err) {}
    return { ok: response.ok, status: response.status, body };
  };

  const apiErrorMessage = (response, fallback) => {
    const body = response && response.body && typeof response.body === 'object' ? response.body : {};
    const message = typeof body.message === 'string' && body.message.trim() ? body.message.trim() : fallback;
    const requestId = typeof body.request_id === 'string' && body.request_id.trim()
      ? ` (request_id: ${body.request_id.trim()})`
      : '';
    return `${message}${requestId}`;
  };

  const showStatus = (kind, message) => {
    if (!statusNode) return;
    statusNode.classList.remove('hidden');
    statusNode.className = kind === 'error' ? 'mt-3 error-card' : 'mt-3 summary-card';
    statusNode.textContent = message;
  };

  const clearStatus = () => {
    if (!statusNode) return;
    statusNode.textContent = '';
    statusNode.className = 'mt-3 hidden';
  };

  const modalOpen = (node) => {
    if (!node) return;
    if (node.parentElement !== document.body) document.body.appendChild(node);
    modalLayerCounter += 1;
    node.style.zIndex = String(MODAL_BASE + modalLayerCounter);
    node.classList.remove('hidden');
    node.setAttribute('aria-hidden', 'false');
  };

  const modalClose = (node) => {
    if (!node) return;
    node.classList.add('hidden');
    node.setAttribute('aria-hidden', 'true');
    node.style.removeProperty('z-index');
    if (!document.querySelector('.app-modal-backdrop:not(.hidden)')) {
      modalLayerCounter = 0;
    }
  };

  const totalPassengers = (payload) => Object.values(payload).reduce((acc, value) => acc + Number(value || 0), 0);
  const formatPassengerCount = (count) => {
    const key = count === 1 ? 'passenger.count.one' : 'passenger.count.other';
    return t(key, { count });
  };

  const passengerPayload = () => passengerKinds
    .map((kind) => ({ kind: kind.key, count: Number(passengerCommitted[kind.key] || 0) }))
    .filter((item) => item.count > 0);

  const normalizeProvider = (value) => {
    const token = String(value || '').trim().toLowerCase();
    if (token === 'ktx') return 'KTX';
    if (token === 'srt') return 'SRT';
    return token.toUpperCase();
  };

  const providerBullets = (station) => {
    const labels = Array.isArray(station.supported_providers)
      ? station.supported_providers.map(normalizeProvider)
      : [];
    const unique = [...new Set(labels)];
    unique.sort((left, right) => {
      const rank = (value) => value === 'KTX' ? 0 : (value === 'SRT' ? 1 : 2);
      return rank(left) - rank(right) || left.localeCompare(right);
    });
    return unique.length ? `• ${unique.join(' • ')}` : '';
  };

  const stationLocalizedName = (station) => {
    if (activeLocale === 'ja' && station.station_name_ja_katakana) return String(station.station_name_ja_katakana);
    if (activeLocale === 'en' && station.station_name_en) return String(station.station_name_en);
    return String(station.station_name_ko || '');
  };

  const stationLabel = (station) => {
    const koName = String(station.station_name_ko || '').trim();
    if (activeLocale === 'ko' || !koName) return koName;
    const localized = stationLocalizedName(station).trim();
    if (!localized || localized === koName) return koName;
    return `${localized} (${koName})`;
  };

  const updateDisplays = () => {
    const depLabel = depSelection
      ? `${stationLabel(depSelection)} · ${depSelection.station_code}`
      : t('search.select_station');
    const arrLabel = arrSelection
      ? `${stationLabel(arrSelection)} · ${arrSelection.station_code}`
      : t('search.select_station');
    depStationDisplay.textContent = depLabel;
    arrStationDisplay.textContent = arrLabel;
    dateDisplay.textContent = depDate || t('search.select_date');
    timeDisplay.textContent = depTime || t('search.select_time');
    passengerDisplay.textContent = formatPassengerCount(totalPassengers(passengerCommitted));
  };

  const formatProviderRows = (providers) => {
    if (!providers || !providers.length) return `<div class="empty-card">${escapeHtml(t('empty.provider_jobs'))}</div>`;
    return providers.map((providerJob) => `
      <div class="summary-row">
        <span>${escapeHtml(providerJob.provider.toUpperCase())} · ${escapeHtml(providerJob.runtime_job_id)}</span>
        <span class="badge">${escapeHtml(providerJob.status)}</span>
      </div>
    `).join('');
  };

  const formatResults = (results) => {
    if (!results || !results.length) return `<div class="empty-card">${escapeHtml(t('empty.results'))}</div>`;
    return results.map((item) => `
      <article class="summary-card">
        <div class="summary-row">
          <span>${escapeHtml(item.provider.toUpperCase())} · #${escapeHtml(item.train_number)}</span>
          <span class="badge">${item.general_seat_available ? escapeHtml(t('status.general_available')) : escapeHtml(t('status.general_unavailable'))}</span>
        </div>
        <div class="summary-row">
          <span>${escapeHtml(item.dep_station_code)} ${escapeHtml(item.dep_time)} → ${escapeHtml(item.arr_station_code)} ${escapeHtml(item.arr_time)}</span>
          <span class="text-xs txt-supporting">${item.special_seat_available ? escapeHtml(t('status.special_available')) : escapeHtml(t('status.special_unavailable'))}</span>
        </div>
      </article>
    `).join('');
  };

  const renderSearchSnapshot = (snapshot) => {
    activeSearchIdNode.textContent = snapshot.search_id || t('search.none');
    jobsNode.innerHTML = formatProviderRows(snapshot.providers || []);
    resultsNode.innerHTML = formatResults(snapshot.results || []);
  };

  const renderHistory = (history) => {
    if (!history || !history.length) {
      historyNode.innerHTML = `<div class="empty-card">${escapeHtml(t('empty.history'))}</div>`;
      return;
    }
    historyNode.innerHTML = history.map((item) => `
      <button type="button" class="summary-card w-full text-left" data-search-id="${escapeHtml(item.search_id)}">
        <div class="summary-row">
          <span>${escapeHtml(item.dep_station_code)} → ${escapeHtml(item.arr_station_code)}</span>
          <span class="badge">${escapeHtml(item.status)}</span>
        </div>
        <p class="mt-1 text-xs txt-supporting">${escapeHtml(item.dep_date)} ${escapeHtml(item.dep_time)} · ${escapeHtml(t('history.providers'))}: ${(item.providers || []).map((value) => escapeHtml(value.toUpperCase())).join(', ')}</p>
      </button>
    `).join('');
    Array.from(historyNode.querySelectorAll('[data-search-id]')).forEach((node) => {
      node.addEventListener('click', () => {
        const searchId = node.getAttribute('data-search-id');
        if (!searchId) return;
        loadSearch(searchId);
      });
    });
  };

  const loadHistory = async () => {
    const response = await requestJson('/api/train/search?limit=12');
    if (!response.ok) {
      historyNode.innerHTML = `<div class="error-card">${escapeHtml(apiErrorMessage(response, t('error.load_history')))}</div>`;
      return;
    }
    renderHistory(response.body && Array.isArray(response.body.searches) ? response.body.searches : []);
  };

  const pollSearch = (searchId) => {
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
    pollTimer = setInterval(async () => {
      const response = await requestJson(`/api/train/search/${encodeURIComponent(searchId)}`);
      if (!response.ok) {
        clearInterval(pollTimer);
        pollTimer = null;
        showStatus('error', apiErrorMessage(response, t('error.poll_search')));
        return;
      }
      const snapshot = response.body || {};
      renderSearchSnapshot(snapshot);
      if (snapshot.status === 'completed' || snapshot.status === 'partial' || snapshot.status === 'failed') {
        clearInterval(pollTimer);
        pollTimer = null;
        loadHistory();
      }
    }, 2500);
  };

  const loadSearch = async (searchId) => {
    const response = await requestJson(`/api/train/search/${encodeURIComponent(searchId)}`);
    if (!response.ok) {
      showStatus('error', apiErrorMessage(response, t('error.load_snapshot')));
      return;
    }
    const snapshot = response.body || {};
    renderSearchSnapshot(snapshot);
    if (snapshot.status !== 'completed' && snapshot.status !== 'partial' && snapshot.status !== 'failed') {
      pollSearch(searchId);
    }
  };

  const providerAuthProbeStatus = (provider) => {
    if (!provider || typeof provider !== 'object') return '';
    const value = typeof provider.auth_probe_status === 'string'
      ? provider.auth_probe_status.trim().toLowerCase()
      : '';
    return value === 'error' || value === 'success' || value === 'skipped' ? value : '';
  };

  const providerHasError = (provider) => {
    if (!provider || typeof provider !== 'object') return false;
    const probeStatus = providerAuthProbeStatus(provider);
    if (probeStatus === 'error') return true;
    return Boolean(
      (typeof provider.error === 'string' && provider.error.trim())
      || (typeof provider.debug === 'string' && provider.debug.trim())
    );
  };

  const statusIcon = (kind, ready, hasError) => {
    const title = kind === 'payment' ? t('provider.payment') : t('provider.credentials');
    const state = hasError ? t('provider.error') : (ready ? t('provider.ready') : t('provider.missing'));
    const iconPrefix = kind === 'payment' ? 'provider-status-payment' : 'provider-status-credentials';
    const lightVariant = hasError
      ? `${iconPrefix}-red.svgz`
      : ready
        ? `${iconPrefix}-green.svgz`
        : `${iconPrefix}-gray-light.svgz`;
    const darkVariant = hasError
      ? `${iconPrefix}-red.svgz`
      : ready
        ? `${iconPrefix}-green.svgz`
        : `${iconPrefix}-gray-dark.svgz`;
    const src = currentThemeMode() === 'dark' ? darkVariant : lightVariant;
    return `
      <span class="provider-status-chip" title="${title}: ${state}" aria-label="${title}: ${state}">
        <img class="status-icon" src="/assets/icons/runtime-ui/${src}" data-svgz-light="/assets/icons/runtime-ui/${lightVariant}" data-svgz-dark="/assets/icons/runtime-ui/${darkVariant}" alt="" aria-hidden="true" />
      </span>
    `;
  };

  const renderPreflight = (preflight) => {
    const providers = Array.isArray(preflight.providers) ? preflight.providers : [];
    const providersByName = new Map(
      providers
        .map((provider) => [String(provider.provider || '').toLowerCase(), provider])
        .filter((entry) => entry[0]),
    );
    const toProviderCard = (name) => {
      const provider = providersByName.get(name);
      const providerReady = Boolean(provider && provider.credentials_ready);
      const providerHasDebugError = Boolean(provider) && providerHasError(provider);
      return `
        <article class="train-preflight-card">
          <span class="train-preflight-label">${escapeHtml(name.toUpperCase())}</span>
          <span class="provider-status-group">
            ${statusIcon('credentials', providerReady, providerHasDebugError)}
          </span>
        </article>
      `;
    };
    const paymentProvider = providersByName.get('ktx') || (providers.length ? providers[0] : null);
    const paymentReady = Boolean(paymentProvider && paymentProvider.payment_ready);
    const paymentHasError = Boolean(paymentProvider) && providerHasError(paymentProvider);
    preflightNode.innerHTML = `
      <div class="train-preflight-grid">
        ${toProviderCard('ktx')}
        ${toProviderCard('srt')}
        <article class="train-preflight-card">
          <span class="train-preflight-label">${escapeHtml(t('provider.payment'))}</span>
          <span class="provider-status-group">
            ${statusIcon('payment', paymentReady, paymentHasError)}
          </span>
        </article>
      </div>
    `;
    syncThemedSvgzIcons(preflightNode);
  };

  const loadPreflight = async () => {
    const response = await requestJson('/api/train/preflight');
    if (!response.ok) {
      preflightNode.innerHTML = `<div class="error-card">${escapeHtml(apiErrorMessage(response, t('error.load_preflight')))}</div>`;
      return;
    }
    renderPreflight(response.body || {});
  };

  const renderStationList = (stations) => {
    if (!stations || !stations.length) {
      stationListNode.innerHTML = `<div class="empty-card">${escapeHtml(t('empty.stations'))}</div>`;
      return;
    }
    stationListNode.innerHTML = stations.map((station) => `
      <button type="button" class="summary-row w-full text-left" data-station-code="${escapeHtml(station.station_code)}">
        <span>${escapeHtml(stationLabel(station))} · ${escapeHtml(station.station_code)}</span>
        <span class="text-xs txt-supporting">${escapeHtml(providerBullets(station))}</span>
      </button>
    `).join('');
    Array.from(stationListNode.querySelectorAll('[data-station-code]')).forEach((button) => {
      button.addEventListener('click', () => {
        const stationCode = button.getAttribute('data-station-code');
        if (!stationCode || !stationRegionsData) return;
        const station = stationRegionsData.regions
          .flatMap((region) => region.stations || [])
          .find((item) => String(item.station_code) === stationCode);
        if (!station) return;
        if (stationPickerTarget === 'dep') {
          depSelection = station;
        } else {
          arrSelection = station;
        }
        updateDisplays();
        modalClose(stationModal);
      });
    });
  };

  const renderRegionChips = () => {
    if (!stationRegionsData) return;
    const regions = (stationRegionsData.regions || []).filter((region) => region.key !== 'major' && region.key !== 'all');
    stationRegionsNode.innerHTML = regions.map((region) => `
      <button type="button" class="${region.key === activeRegionKey ? 'btn-primary h-9 px-3' : 'btn-ghost h-9 px-3'}" data-region-key="${escapeHtml(region.key)}">${escapeHtml(region.label)}</button>
    `).join('');
    Array.from(stationRegionsNode.querySelectorAll('[data-region-key]')).forEach((button) => {
      button.addEventListener('click', () => {
        const key = button.getAttribute('data-region-key');
        if (!key) return;
        activeRegionKey = key;
        renderRegionChips();
      });
    });
    const region = regions.find((value) => value.key === activeRegionKey) || regions[0];
    renderStationList(region ? region.stations : []);
  };

  const renderStationTab = () => {
    if (!stationRegionsData) return;
    stationTabMajor.className = stationTab === 'major' ? 'btn-primary h-9 px-3' : 'btn-ghost h-9 px-3';
    stationTabRegion.className = stationTab === 'region' ? 'btn-primary h-9 px-3' : 'btn-ghost h-9 px-3';
    if (stationTab === 'major') {
      const major = (stationRegionsData.regions || []).find((region) => region.key === 'major');
      stationRegionsNode.innerHTML = '';
      renderStationList(major ? major.stations : []);
      return;
    }
    renderRegionChips();
  };

  const clearStationCorrection = () => {
    if (!stationCorrection) return;
    stationCorrection.innerHTML = '';
    stationCorrection.classList.add('hidden');
  };

  const suggestLangHint = () => {
    if (activeLocale === 'ko') return 'ko';
    if (activeLocale === 'ja') return 'ja';
    if (activeLocale === 'en') return 'en';
    return 'auto';
  };

  const suggestLayoutHint = (query) => {
    const compact = String(query || '').replace(/\s+/g, '');
    if (compact && /^[a-z0-9]+$/i.test(compact)) return 'qwerty';
    return 'auto';
  };

  const buildStationSuggestUrl = (query) => {
    const params = new URLSearchParams();
    params.set('q', query);
    params.set('limit', '10');
    params.set('apply_mode', 'suggest');
    params.set('lang_hint', suggestLangHint());
    params.set('layout_hint', suggestLayoutHint(query));
    return `/api/train/stations/suggest?${params.toString()}`;
  };

  const renderStationCorrection = (body) => {
    if (!stationCorrection) return;
    const correctedQuery = String(body?.corrected_query || '').trim();
    const autocorrectApplied = Boolean(body?.autocorrect_applied);
    const currentQuery = String(stationQuery?.value || '').trim();
    if (!autocorrectApplied || !correctedQuery || correctedQuery === currentQuery) {
      clearStationCorrection();
      return;
    }
    stationCorrection.classList.remove('hidden');
    stationCorrection.innerHTML = `
      <button type="button" class="summary-row w-full text-left" data-station-use-correction="${escapeHtml(correctedQuery)}">
        <span>${escapeHtml(t('station.correction_prompt', { query: correctedQuery }))}</span>
      </button>
    `;
    const button = stationCorrection.querySelector('[data-station-use-correction]');
    if (!button) return;
    button.addEventListener('click', async () => {
      stationQuery.value = correctedQuery;
      await queryStationSuggestions(correctedQuery);
      stationQuery.focus();
    });
  };

  const loadStationRegions = async () => {
    if (stationRegionsData) return stationRegionsData;
    const response = await requestJson('/api/train/stations/regions');
    if (!response.ok) {
      showStatus('error', apiErrorMessage(response, t('error.load_station_catalog')));
      return null;
    }
    stationRegionsData = response.body || { quick: [], regions: [] };
    return stationRegionsData;
  };

  const queryStationSuggestions = async (query) => {
    stationQueryCounter += 1;
    const requestId = stationQueryCounter;
    const response = await requestJson(buildStationSuggestUrl(query));
    if (requestId !== stationQueryCounter) return;
    if (!response.ok) {
      clearStationCorrection();
      stationSuggestions.innerHTML = `<div class="error-card">${escapeHtml(apiErrorMessage(response, t('error.station_lookup')))}</div>`;
      return;
    }
    renderStationCorrection(response.body);
    const suggestions = Array.isArray(response.body?.suggestions) ? response.body.suggestions : [];
    const merged = new Map();
    for (const station of suggestions) {
      const key = String(station.station_code || '').trim();
      const provider = normalizeProvider(station.provider);
      if (!merged.has(key)) {
        merged.set(key, {
          station_code: key,
          station_name_ko: station.station_name_ko || '',
          station_name_en: station.station_name_en || '',
          station_name_ja_katakana: station.station_name_ja_katakana || '',
          supported_providers: provider ? [provider] : [],
        });
      } else if (provider) {
        const existing = merged.get(key);
        if (!existing.supported_providers.includes(provider)) {
          existing.supported_providers.push(provider);
        }
      }
    }
    renderStationList(Array.from(merged.values()));
  };

  const openStationPicker = async (target) => {
    stationPickerTarget = target;
    stationQuery.value = '';
    clearStationCorrection();
    stationSuggestions.innerHTML = '';
    stationQueryCounter += 1;
    if (stationSuggestDebounceTimer) {
      clearTimeout(stationSuggestDebounceTimer);
      stationSuggestDebounceTimer = null;
    }
    const loaded = await loadStationRegions();
    if (!loaded) return;
    renderStationTab();
    modalOpen(stationModal);
    stationQuery.focus();
  };

  const renderHourButtons = () => {
    timeHourList.innerHTML = Array.from({ length: 24 }).map((_, idx) => {
      const token = `${String(idx).padStart(2, '0')}:00`;
      const selected = depTime.slice(0, 2) === token.slice(0, 2);
      return `<button type="button" class="${selected ? 'btn-primary h-9 w-full' : 'btn-ghost h-9 w-full'}" data-hour="${token}">${token}</button>`;
    }).join('');
    Array.from(timeHourList.querySelectorAll('[data-hour]')).forEach((button) => {
      button.addEventListener('click', () => {
        const token = button.getAttribute('data-hour');
        if (!token) return;
        depTime = token;
        if (timeDesktopInput) timeDesktopInput.value = token;
        renderHourButtons();
      });
    });
  };

  const renderPassengerRows = () => {
    passengerRows.innerHTML = passengerKinds.map((kind) => `
      <div class="summary-row">
        <span>${escapeHtml(kind.label)}</span>
        <span class="inline-flex items-center gap-2">
          <button type="button" class="btn-ghost h-8 w-8 p-0" data-passenger-op="minus" data-passenger-kind="${escapeHtml(kind.key)}">−</button>
          <span class="w-6 text-center">${escapeHtml(String(passengerDraft[kind.key] || 0))}</span>
          <button type="button" class="btn-ghost h-8 w-8 p-0" data-passenger-op="plus" data-passenger-kind="${escapeHtml(kind.key)}">＋</button>
        </span>
      </div>
    `).join('');
    Array.from(passengerRows.querySelectorAll('[data-passenger-op]')).forEach((button) => {
      button.addEventListener('click', () => {
        const kind = button.getAttribute('data-passenger-kind');
        const op = button.getAttribute('data-passenger-op');
        if (!kind || !Object.hasOwn(passengerDraft, kind)) return;
        const total = totalPassengers(passengerDraft);
        const current = Number(passengerDraft[kind] || 0);
        if (op === 'plus') {
          if (total >= 9) return;
          passengerDraft[kind] = current + 1;
        } else {
          passengerDraft[kind] = Math.max(0, current - 1);
        }
        renderPassengerRows();
      });
    });
  };

  depStationOpen.addEventListener('click', () => openStationPicker('dep'));
  arrStationOpen.addEventListener('click', () => openStationPicker('arr'));
  swapButton.addEventListener('click', () => {
    const prevDep = depSelection;
    depSelection = arrSelection;
    arrSelection = prevDep;
    updateDisplays();
  });

  dateOpen.addEventListener('click', () => {
    dateInput.value = depDate;
    modalOpen(dateModal);
  });

  timeOpen.addEventListener('click', () => {
    if (timeDesktopInput) timeDesktopInput.value = depTime;
    renderHourButtons();
    modalOpen(timeModal);
  });

  passengerOpen.addEventListener('click', () => {
    passengerDraft = { ...passengerCommitted };
    renderPassengerRows();
    modalOpen(passengerModal);
  });

  stationModalClose.addEventListener('click', () => modalClose(stationModal));
  dateClose.addEventListener('click', () => modalClose(dateModal));
  timeClose.addEventListener('click', () => modalClose(timeModal));
  passengerClose.addEventListener('click', () => modalClose(passengerModal));
  dateCancel.addEventListener('click', () => modalClose(dateModal));
  timeCancel.addEventListener('click', () => modalClose(timeModal));
  passengerCancel.addEventListener('click', () => modalClose(passengerModal));

  stationTabMajor.addEventListener('click', () => {
    stationTab = 'major';
    renderStationTab();
  });
  stationTabRegion.addEventListener('click', () => {
    stationTab = 'region';
    renderStationTab();
  });

  stationQuery.addEventListener('input', async () => {
    const value = stationQuery.value.trim();
    if (!value) {
      stationQueryCounter += 1;
      clearStationCorrection();
      if (stationSuggestDebounceTimer) {
        clearTimeout(stationSuggestDebounceTimer);
        stationSuggestDebounceTimer = null;
      }
      renderStationTab();
      return;
    }
    if (stationSuggestDebounceTimer) {
      clearTimeout(stationSuggestDebounceTimer);
    }
    stationSuggestDebounceTimer = setTimeout(() => {
      stationSuggestDebounceTimer = null;
      queryStationSuggestions(value).catch(() => {});
    }, 150);
  });

  dateApply.addEventListener('click', () => {
    const pickedDate = String(dateInput.value || '').trim();
    if (!pickedDate) {
      showStatus('error', t('error.date_required'));
      return;
    }
    depDate = pickedDate;
    updateDisplays();
    modalClose(dateModal);
  });

  timeApply.addEventListener('click', () => {
    if (window.matchMedia('(min-width: 768px)').matches && timeDesktopInput && timeDesktopInput.value) {
      depTime = timeDesktopInput.value;
    }
    updateDisplays();
    modalClose(timeModal);
  });

  passengerApply.addEventListener('click', () => {
    if (totalPassengers(passengerDraft) < 1) {
      showStatus('error', t('error.passenger_required'));
      return;
    }
    passengerCommitted = { ...passengerDraft };
    updateDisplays();
    modalClose(passengerModal);
  });

  [stationModal, dateModal, timeModal, passengerModal].forEach((modalNode) => {
    modalNode.addEventListener('click', (event) => {
      if (event.target === modalNode) modalClose(modalNode);
    });
  });

  form.addEventListener('submit', async (event) => {
    event.preventDefault();
    clearStatus();
    if (!depSelection || !arrSelection) {
      showStatus('error', t('error.station_required'));
      return;
    }
    if (totalPassengers(passengerCommitted) < 1) {
      showStatus('error', t('error.passenger_required'));
      return;
    }
    const response = await requestJson('/api/train/search', 'POST', {
      dep_station_code: depSelection.station_code,
      arr_station_code: arrSelection.station_code,
      dep_date: String(depDate || '').replaceAll('-', ''),
      dep_time: `${String(depTime || '00:00').replace(':', '')}00`,
      passengers: passengerPayload(),
      available_only: true,
    });
    if (!response.ok) {
      showStatus('error', apiErrorMessage(response, t('error.search_failed')));
      return;
    }
    const created = response.body || {};
    renderSearchSnapshot({ providers: created.jobs || [], results: [], search_id: created.search_id || '' });
    showStatus('success', t('success.search_accepted', { searchId: created.search_id }));
    if (created.search_id) pollSearch(created.search_id);
    loadHistory();
  });

  applyStaticTranslations();
  updateDisplays();
  syncThemedSvgzIcons();
  if (document.body && typeof MutationObserver === 'function') {
    const themeObserver = new MutationObserver(() => syncThemedSvgzIcons());
    themeObserver.observe(document.body, { attributes: true, attributeFilter: ['data-theme-mode'] });
  }
  loadPreflight();
  loadHistory();
})();
</script>"#,
    );
    html.push_str(&bottom_nav);
    html
}

pub fn render_dashboard_settings_providers(email: &str) -> String {
    let topbar = app_shell_topbar("Settings", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("security");
    let bottom_nav = render_dashboard_bottom_nav(DashboardSection::Settings);
    let settings_tabs = dashboard_settings_tabs("provider");
    let mut html = String::new();
    html.push_str(&topbar);
    html.push_str(
        r#"<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">"#,
    );
    html.push_str(&sidebar);
    html.push_str(r#"<section class="space-y-4">"#);
    html.push_str(&settings_tabs);
    html.push_str(
        r#"
      <section class="glass-card rounded-[22px] p-5">
        <h2 class="text-lg font-semibold txt-strong">Provider Authentication</h2>
        <p class="mt-1 text-sm txt-supporting">Select a provider card to open its authentication form.</p>
        <div id="provider-security-preflight" class="mt-3 grid grid-cols-2 gap-2"><div class="loading-card col-span-2">Loading status...</div></div>
      </section>

      <div id="provider-security-status" class="hidden"></div>
    </section>
  </div>
</main>
<div id="provider-auth-modal" class="app-modal-backdrop hidden" aria-hidden="true">
  <div class="app-modal-card" role="dialog" aria-modal="true" aria-labelledby="provider-auth-modal-title">
    <div class="flex items-center justify-between gap-2">
      <h3 id="provider-auth-modal-title" class="text-base font-semibold txt-strong">Provider Authentication</h3>
      <button id="provider-auth-modal-delete" type="button" class="btn-destructive h-9 w-9 p-0" aria-label="Delete credentials">
        <img class="h-4 w-4" src="/assets/icons/runtime-ui/icon-trash-destructive-light.svgz" data-svgz-light="/assets/icons/runtime-ui/icon-trash-destructive-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/icon-trash-destructive-dark.svgz" alt="" aria-hidden="true" />
      </button>
    </div>
    <p class="mt-1 text-xs txt-supporting"><span id="provider-auth-modal-provider">Provider</span> credentials are encrypted server-side.</p>
    <form id="provider-auth-form" class="mt-3 space-y-3">
      <label class="field-label" for="provider-auth-account">Account identifier</label>
      <input id="provider-auth-account" type="text" autocomplete="username" class="field-input h-11 w-full" />
      <label class="field-label" for="provider-auth-password">Password</label>
      <input id="provider-auth-password" type="password" autocomplete="current-password" class="field-input h-11 w-full" />
      <div class="grid grid-cols-2 gap-2" data-action-group="pair">
        <button id="provider-auth-modal-cancel" type="button" class="btn-ghost h-11 w-full">Cancel</button>
        <button id="provider-auth-modal-save" type="submit" class="btn-primary h-11 w-full">Save</button>
      </div>
    </form>
  </div>
</div>
<script>
(() => {
  const preflightNode = document.getElementById('provider-security-preflight');
  const statusNode = document.getElementById('provider-security-status');
  const authModal = document.getElementById('provider-auth-modal');
  const authModalTitle = document.getElementById('provider-auth-modal-title');
  const authModalProvider = document.getElementById('provider-auth-modal-provider');
  const authModalDelete = document.getElementById('provider-auth-modal-delete');
  const authModalCancel = document.getElementById('provider-auth-modal-cancel');
  const authForm = document.getElementById('provider-auth-form');
  const authAccountInput = document.getElementById('provider-auth-account');
  const authPasswordInput = document.getElementById('provider-auth-password');

  const requestJson = async (url, method, payload) => {
    const response = await fetch(url, {
      method: method || 'GET',
      headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
      body: payload ? JSON.stringify(payload) : undefined,
    });
    const text = await response.text();
    let body = null;
    try { body = text ? JSON.parse(text) : null; } catch (_err) {}
    return { ok: response.ok, status: response.status, body };
  };

  const escapeHtml = (value) => String(value || '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');

  const apiErrorMessage = (response, fallback) => {
    const body = response && response.body && typeof response.body === 'object' ? response.body : {};
    const message = typeof body.message === 'string' && body.message.trim()
      ? body.message.trim()
      : fallback;
    const requestId = typeof body.request_id === 'string' && body.request_id.trim()
      ? ` (request_id: ${body.request_id.trim()})`
      : '';
    return `${message}${requestId}`;
  };

  const providerOrder = ['srt', 'ktx'];
  const MODAL_LAYER_BASE = 70;
  let modalLayerCounter = 0;
  let activeProvider = null;
  let latestProvidersByName = new Map();
  const currentThemeMode = () => document.body?.dataset?.themeMode === 'dark' ? 'dark' : 'light';
  const syncThemedSvgzIcons = (rootNode) => {
    const mode = currentThemeMode();
    const root = rootNode && typeof rootNode.querySelectorAll === 'function' ? rootNode : document;
    const icons = root.querySelectorAll('img[data-svgz-light][data-svgz-dark]');
    icons.forEach((icon) => {
      const nextSrc = mode === 'dark' ? icon.dataset.svgzDark : icon.dataset.svgzLight;
      if (!nextSrc || icon.getAttribute('src') === nextSrc) return;
      icon.setAttribute('src', nextSrc);
    });
  };

  const showStatus = (kind, message) => {
    statusNode.classList.remove('hidden');
    statusNode.className = kind === 'error' ? 'error-card' : 'summary-card';
    statusNode.textContent = message;
  };

  const bringModalToBody = (modalNode) => {
    if (!modalNode || !document.body) return;
    if (modalNode.parentElement !== document.body) {
      document.body.appendChild(modalNode);
    }
  };

  const openModalLayer = (modalNode) => {
    if (!modalNode) return;
    bringModalToBody(modalNode);
    modalLayerCounter += 1;
    modalNode.style.zIndex = String(MODAL_LAYER_BASE + modalLayerCounter);
    modalNode.classList.remove('hidden');
    modalNode.setAttribute('aria-hidden', 'false');
  };

  const closeModalLayer = (modalNode) => {
    if (!modalNode) return;
    modalNode.classList.add('hidden');
    modalNode.setAttribute('aria-hidden', 'true');
    modalNode.style.removeProperty('z-index');
    if (!document.querySelector('.app-modal-backdrop:not(.hidden)')) {
      modalLayerCounter = 0;
    }
  };

  const normalizeProvider = (value) => {
    const normalized = String(value || '').trim().toLowerCase();
    return providerOrder.includes(normalized) ? normalized : null;
  };

  const activeProviderRecord = () => {
    if (!activeProvider) return null;
    return latestProvidersByName.get(activeProvider) || null;
  };

  const providerAuthProbeStatus = (provider) => {
    if (!provider || typeof provider !== 'object') return '';
    const value = typeof provider.auth_probe_status === 'string'
      ? provider.auth_probe_status.trim().toLowerCase()
      : '';
    return value === 'error' || value === 'success' || value === 'skipped' ? value : '';
  };

  const providerStatusMessage = (provider) => {
    if (!provider || typeof provider !== 'object') return '';
    if (typeof provider.auth_probe_message === 'string' && provider.auth_probe_message.trim()) {
      return provider.auth_probe_message.trim();
    }
    if (typeof provider.error === 'string' && provider.error.trim()) return provider.error.trim();
    if (typeof provider.debug === 'string' && provider.debug.trim()) return provider.debug.trim();
    if (typeof provider.message === 'string' && provider.message.trim()) return provider.message.trim();
    return '';
  };

  const statusIcon = (kind, ready, hasError) => {
    const title = kind === 'credentials' ? 'Credentials' : 'Payment';
    const state = hasError ? 'error' : (ready ? 'ready' : 'missing');
    const iconPrefix = kind === 'credentials' ? 'provider-status-credentials' : 'provider-status-payment';
    const lightVariant = hasError
      ? `${iconPrefix}-red.svgz`
      : ready
        ? `${iconPrefix}-green.svgz`
        : `${iconPrefix}-gray-light.svgz`;
    const darkVariant = hasError
      ? `${iconPrefix}-red.svgz`
      : ready
        ? `${iconPrefix}-green.svgz`
        : `${iconPrefix}-gray-dark.svgz`;
    const src = currentThemeMode() === 'dark' ? darkVariant : lightVariant;
    return `
      <span class="provider-status-chip" title="${title}: ${state}" aria-label="${title}: ${state}">
        <img class="status-icon" src="/assets/icons/runtime-ui/${src}" data-svgz-light="/assets/icons/runtime-ui/${lightVariant}" data-svgz-dark="/assets/icons/runtime-ui/${darkVariant}" alt="" aria-hidden="true" />
      </span>
    `;
  };

  const closeAuthModal = () => {
    closeModalLayer(authModal);
    activeProvider = null;
    if (authForm) authForm.reset();
  };

  const openAuthModal = (provider) => {
    const normalizedProvider = normalizeProvider(provider);
    if (!normalizedProvider) return;
    activeProvider = normalizedProvider;
    if (authModalTitle) authModalTitle.textContent = `${normalizedProvider.toUpperCase()} Authentication`;
    if (authModalProvider) authModalProvider.textContent = normalizedProvider.toUpperCase();
    if (authForm) authForm.reset();
    openModalLayer(authModal);
    requestAnimationFrame(() => {
      if (authAccountInput) authAccountInput.focus();
    });
  };

  const deleteProviderCredentials = async () => {
    const selected = activeProviderRecord();
    if (!selected || !activeProvider) return;
    const deletedProvider = activeProvider;
    const prompt = `Delete saved ${deletedProvider.toUpperCase()} credentials?`;
    if (!window.confirm(prompt)) return;
    const response = await requestJson(`/api/train/providers/${deletedProvider}/credentials`, 'DELETE');
    if (!response.ok) {
      showStatus(
        'error',
        apiErrorMessage(response, `Could not delete ${deletedProvider.toUpperCase()} credentials.`),
      );
      return;
    }
    closeAuthModal();
    showStatus('success', `${deletedProvider.toUpperCase()}: Credentials removed.`);
    await loadPreflight();
  };

  const loadPreflight = async () => {
    const response = await requestJson('/api/train/preflight');
    if (!response.ok) {
      preflightNode.innerHTML = `<div class="error-card col-span-2">${escapeHtml(apiErrorMessage(response, 'Could not load readiness.'))}</div>`;
      return;
    }
    const providers = response.body && Array.isArray(response.body.providers) ? response.body.providers : [];
    latestProvidersByName = new Map(
      providers
        .map((provider) => {
          const key = normalizeProvider(provider.provider);
          return key ? [key, provider] : null;
        })
        .filter((entry) => !!entry),
    );

    preflightNode.innerHTML = providers.map((provider) => {
      const providerKey = normalizeProvider(provider.provider);
      const statusMessage = providerStatusMessage(provider);
      const probeStatus = providerAuthProbeStatus(provider);
      const hasError = Boolean(provider.credentials_ready) && probeStatus === 'error';
      return `
        <button type="button" class="summary-card provider-select-card p-3 w-full text-left" data-provider-open="${providerKey || ''}">
          <span class="flex h-8 items-center justify-between gap-2">
            <span class="txt-strong">${provider.provider.toUpperCase()}</span>
            <span class="provider-status-group">
              ${statusIcon('credentials', provider.credentials_ready, hasError)}
            </span>
          </span>
          ${statusMessage ? `<span class="support-row mt-2 block">${escapeHtml(statusMessage)}</span>` : ''}
        </button>
      `;
    }).join('') || '<div class="empty-card col-span-2">No provider data.</div>';

    Array.from(preflightNode.querySelectorAll('[data-provider-open]')).forEach((button) => {
      button.addEventListener('click', () => {
        const provider = normalizeProvider(button.getAttribute('data-provider-open'));
        if (!provider) return;
        openAuthModal(provider);
      });
    });
    syncThemedSvgzIcons(preflightNode);
  };

  if (authModalCancel) {
    authModalCancel.addEventListener('click', () => {
      closeAuthModal();
    });
  }

  if (authModalDelete) {
    authModalDelete.addEventListener('click', async () => {
      await deleteProviderCredentials();
    });
  }

  if (authForm) {
    authForm.addEventListener('submit', async (event) => {
      event.preventDefault();
      if (!activeProvider) return;
      const submittedProvider = activeProvider;
      const account = String(authAccountInput && authAccountInput.value ? authAccountInput.value : '').trim();
      const password = String(authPasswordInput && authPasswordInput.value ? authPasswordInput.value : '');
      const response = await requestJson(`/api/train/providers/${submittedProvider}/credentials`, 'PUT', {
        account_identifier: account,
        password,
      });
      if (!response.ok) {
        showStatus('error', apiErrorMessage(response, `Could not save ${submittedProvider.toUpperCase()} credentials.`));
        return;
      }
      const result = response.body && typeof response.body === 'object' ? response.body : {};
      const probeStatus = typeof result.auth_probe_status === 'string'
        ? result.auth_probe_status.trim().toLowerCase()
        : '';
      const probeMessage = typeof result.auth_probe_message === 'string' && result.auth_probe_message.trim()
        ? result.auth_probe_message.trim()
        : '';
      closeAuthModal();
      await loadPreflight();
      if (probeStatus === 'error') {
        showStatus('error', probeMessage || `${submittedProvider.toUpperCase()}: Authentication failed.`);
        return;
      }
      showStatus('success', probeMessage || `${submittedProvider.toUpperCase()}: Successfully authenticated.`);
    });
  }

  if (authModal) {
    authModal.addEventListener('click', (event) => {
      if (event.target === authModal) {
        closeAuthModal();
      }
    });
  }

  document.addEventListener('keydown', (event) => {
    if (event.key !== 'Escape') return;
    if (!authModal || authModal.classList.contains('hidden')) return;
    closeAuthModal();
  });

  syncThemedSvgzIcons();
  if (document.body && typeof MutationObserver === 'function') {
    const themeObserver = new MutationObserver(() => syncThemedSvgzIcons());
    themeObserver.observe(document.body, { attributes: true, attributeFilter: ['data-theme-mode'] });
  }
  loadPreflight();
})();
</script>"#,
    );
    html.push_str(&bottom_nav);
    html
}

pub fn render_dashboard_payment(email: &str) -> String {
    let topbar = app_shell_topbar("Settings", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("security");
    let bottom_nav = render_dashboard_bottom_nav(DashboardSection::Settings);
    let settings_tabs = dashboard_settings_tabs("payment");
    let mut html = String::new();
    html.push_str(&topbar);
    html.push_str(
        r#"<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">"#,
    );
    html.push_str(&sidebar);
    html.push_str(r#"<section class="space-y-4">"#);
    html.push_str(&settings_tabs);
    html.push_str(
        r#"
      <section class="glass-card rounded-[22px] p-5">
        <div class="flex items-center justify-between gap-2">
          <h3 class="text-base font-semibold txt-strong">Saved cards</h3>
          <button
            id="payment-modal-open"
            type="button"
            class="btn-chip inline-flex h-9 w-9 items-center justify-center rounded-xl p-0"
            aria-label="Add payment card"
            title="Add payment card"
          >
            <img class="h-4 w-4 icon-muted" src="/assets/icons/runtime-ui/icon-plus-btn-chip-light.svgz" data-svgz-light="/assets/icons/runtime-ui/icon-plus-btn-chip-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/icon-plus-btn-chip-dark.svgz" alt="" aria-hidden="true" />
          </button>
        </div>
        <div id="payment-card-list" class="mt-3 space-y-2"><div class="loading-card">Loading saved cards...</div></div>
      </section>

      <div id="payment-status" class="hidden"></div>
    </section>
  </div>
</main>
<div id="payment-modal" class="app-modal-backdrop hidden" aria-hidden="true">
  <div class="app-modal-card" role="dialog" aria-modal="true" aria-labelledby="payment-modal-title">
    <div class="flex items-center justify-between gap-2">
      <h4 id="payment-modal-title" class="text-base font-semibold txt-strong">Payment information</h4>
      <button id="payment-modal-delete" type="button" class="btn-destructive h-9 w-9 p-0" aria-label="Delete card">
        <img class="h-4 w-4" src="/assets/icons/runtime-ui/icon-trash-destructive-light.svgz" data-svgz-light="/assets/icons/runtime-ui/icon-trash-destructive-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/icon-trash-destructive-dark.svgz" alt="" aria-hidden="true" />
      </button>
    </div>
    <p id="payment-modal-selected-card" class="mt-1 text-xs txt-supporting">Adding a new card</p>
    <form id="payment-form-universal" class="mt-3 space-y-3">
      <label class="field-label" for="payment-pan">pan_ev</label>
      <input id="payment-pan" type="text" class="field-input h-11 w-full" autocomplete="off" />

      <label class="field-label" for="payment-expiry-month">expiry_month_ev</label>
      <input id="payment-expiry-month" type="text" class="field-input h-11 w-full" autocomplete="off" />

      <label class="field-label" for="payment-expiry-year">expiry_year_ev</label>
      <input id="payment-expiry-year" type="text" class="field-input h-11 w-full" autocomplete="off" />

      <label class="field-label" for="payment-birth-business">birth_or_business_ev</label>
      <input id="payment-birth-business" type="text" class="field-input h-11 w-full" autocomplete="off" />

      <label class="field-label" for="payment-card-password">card_password_two_digits_ev</label>
      <input id="payment-card-password" type="text" class="field-input h-11 w-full" autocomplete="off" />

      <label class="field-label" for="payment-card-last4">card_last4 (required)</label>
      <input id="payment-card-last4" type="text" maxlength="4" class="field-input h-11 w-full" autocomplete="off" />

      <label class="field-label" for="payment-card-brand">card_brand (optional)</label>
      <input id="payment-card-brand" type="text" class="field-input h-11 w-full" autocomplete="off" />

      <div class="grid grid-cols-2 gap-2" data-action-group="pair">
        <button id="payment-modal-cancel" type="button" class="btn-ghost h-11 w-full">Cancel</button>
        <button id="payment-modal-save" type="submit" class="btn-primary h-11 w-full">Save</button>
      </div>
    </form>
  </div>
</div>
<script>
(() => {
  const statusNode = document.getElementById('payment-status');
  const form = document.getElementById('payment-form-universal');
  const cardListNode = document.getElementById('payment-card-list');
  const paymentModalOpenButton = document.getElementById('payment-modal-open');
  const paymentModal = document.getElementById('payment-modal');
  const paymentModalDeleteButton = document.getElementById('payment-modal-delete');
  const paymentModalCancelButton = document.getElementById('payment-modal-cancel');
  const paymentModalSelectedCard = document.getElementById('payment-modal-selected-card');
  const paymentPanInput = document.getElementById('payment-pan');
  const paymentExpiryMonthInput = document.getElementById('payment-expiry-month');
  const paymentExpiryYearInput = document.getElementById('payment-expiry-year');
  const paymentBirthBusinessInput = document.getElementById('payment-birth-business');
  const paymentCardPasswordInput = document.getElementById('payment-card-password');
  const paymentCardLast4Input = document.getElementById('payment-card-last4');
  const paymentCardBrandInput = document.getElementById('payment-card-brand');
  const MODAL_LAYER_BASE = 70;
  let modalLayerCounter = 0;
  let selectedPaymentMethodRef = null;
  let selectedPaymentLast4 = null;
  const currentThemeMode = () => document.body?.dataset?.themeMode === 'dark' ? 'dark' : 'light';
  const syncThemedSvgzIcons = () => {
    const mode = currentThemeMode();
    const icons = document.querySelectorAll('img[data-svgz-light][data-svgz-dark]');
    icons.forEach((icon) => {
      const nextSrc = mode === 'dark' ? icon.dataset.svgzDark : icon.dataset.svgzLight;
      if (!nextSrc || icon.getAttribute('src') === nextSrc) return;
      icon.setAttribute('src', nextSrc);
    });
  };

  const requestJson = async (url, method, payload) => {
    const response = await fetch(url, {
      method: method || 'GET',
      headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
      body: payload ? JSON.stringify(payload) : undefined,
    });
    const text = await response.text();
    let body = null;
    try { body = text ? JSON.parse(text) : null; } catch (_err) {}
    return { ok: response.ok, status: response.status, body };
  };

  const escapeHtml = (value) => String(value || '')
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;');

  const apiErrorMessage = (response, fallback) => {
    const body = response && response.body && typeof response.body === 'object' ? response.body : {};
    const message = typeof body.message === 'string' && body.message.trim()
      ? body.message.trim()
      : fallback;
    const requestId = typeof body.request_id === 'string' && body.request_id.trim()
      ? ` (request_id: ${body.request_id.trim()})`
      : '';
    return `${message}${requestId}`;
  };

  const showStatus = (kind, message) => {
    statusNode.classList.remove('hidden');
    statusNode.className = kind === 'error' ? 'error-card' : 'summary-card';
    statusNode.textContent = message;
  };

  const bringModalToBody = (modalNode) => {
    if (!modalNode || !document.body) return;
    if (modalNode.parentElement !== document.body) {
      document.body.appendChild(modalNode);
    }
  };

  const openModalLayer = (modalNode) => {
    if (!modalNode) return;
    bringModalToBody(modalNode);
    modalLayerCounter += 1;
    modalNode.style.zIndex = String(MODAL_LAYER_BASE + modalLayerCounter);
    modalNode.classList.remove('hidden');
    modalNode.setAttribute('aria-hidden', 'false');
  };

  const closeModalLayer = (modalNode) => {
    if (!modalNode) return;
    modalNode.classList.add('hidden');
    modalNode.setAttribute('aria-hidden', 'true');
    modalNode.style.removeProperty('z-index');
    if (!document.querySelector('.app-modal-backdrop:not(.hidden)')) {
      modalLayerCounter = 0;
    }
  };

  const resetPaymentForm = () => {
    if (form) form.reset();
  };

  const syncPaymentModalState = () => {
    const hasSelectedCard = !!(selectedPaymentMethodRef && selectedPaymentLast4);
    if (paymentModalDeleteButton) {
      paymentModalDeleteButton.disabled = !hasSelectedCard;
    }
    if (paymentModalSelectedCard) {
      paymentModalSelectedCard.textContent = hasSelectedCard
        ? `Selected card: •••• ${selectedPaymentLast4}`
        : 'Adding a new card';
    }
  };

  const openPaymentModal = () => {
    syncPaymentModalState();
    openModalLayer(paymentModal);
    requestAnimationFrame(() => {
      if (paymentPanInput) paymentPanInput.focus();
    });
  };

  const closePaymentModal = () => {
    closeModalLayer(paymentModal);
    resetPaymentForm();
  };

  const deleteSelectedCard = async () => {
    const paymentMethodRef = String(selectedPaymentMethodRef || '').trim();
    const last4 = String(selectedPaymentLast4 || '').trim();
    if (!paymentMethodRef) return;
    if (!window.confirm(`Delete saved card ending ${last4}?`)) return;

    const response = await requestJson(`/api/train/payment-methods/${encodeURIComponent(paymentMethodRef)}`, 'DELETE');
    if (!response.ok) {
      showStatus('error', apiErrorMessage(response, 'Could not delete saved card.'));
      return;
    }
    showStatus('success', `Deleted card ending ${last4}.`);
    selectedPaymentMethodRef = null;
    selectedPaymentLast4 = null;
    closePaymentModal();
    loadCards();
  };

  const renderCards = (cards) => {
    if (!Array.isArray(cards) || cards.length === 0) {
      cardListNode.innerHTML = '<div class="empty-card">No saved cards.</div>';
      return;
    }

    cardListNode.innerHTML = cards.map((card) => `
      <button type="button" class="summary-card w-full text-left" data-payment-ref="${escapeHtml(card.payment_method_ref || '')}" data-card-last4="${escapeHtml(card.card_last4 || '0000')}">
        <div class="summary-row">
          <span class="txt-strong">${escapeHtml((card.card_brand || 'Card').toUpperCase())} · •••• ${escapeHtml(card.card_last4 || '0000')}</span>
          <span class="text-xs txt-supporting">${escapeHtml(card.payment_method_ref || '')}</span>
        </div>
      </button>
    `).join('');

    Array.from(cardListNode.querySelectorAll('[data-payment-ref]')).forEach((button) => {
      button.addEventListener('click', async () => {
        selectedPaymentMethodRef = String(button.getAttribute('data-payment-ref') || '').trim() || null;
        selectedPaymentLast4 = String(button.getAttribute('data-card-last4') || '').trim() || null;
        openPaymentModal();
      });
    });
  };

  const loadCards = async () => {
    const response = await requestJson('/api/train/payment-methods');
    if (!response.ok) {
      cardListNode.innerHTML = `<div class="error-card">${escapeHtml(apiErrorMessage(response, 'Could not load saved cards.'))}</div>`;
      return;
    }
    const cards = response.body && Array.isArray(response.body.cards) ? response.body.cards : [];
    renderCards(cards);
  };

  if (paymentModalOpenButton) {
    paymentModalOpenButton.addEventListener('click', () => {
      selectedPaymentMethodRef = null;
      selectedPaymentLast4 = null;
      openPaymentModal();
    });
  }

  if (paymentModalCancelButton) {
    paymentModalCancelButton.addEventListener('click', () => {
      closePaymentModal();
    });
  }

  if (paymentModalDeleteButton) {
    paymentModalDeleteButton.addEventListener('click', async () => {
      await deleteSelectedCard();
    });
  }

  if (paymentModal) {
    paymentModal.addEventListener('click', (event) => {
      if (event.target === paymentModal) {
        closePaymentModal();
      }
    });
  }

  document.addEventListener('keydown', (event) => {
    if (event.key !== 'Escape') return;
    if (!paymentModal || paymentModal.classList.contains('hidden')) return;
    closePaymentModal();
  });

  form.addEventListener('submit', async (event) => {
    event.preventDefault();
    const response = await requestJson('/api/train/payment-methods', 'PUT', {
      pan_ev: String(paymentPanInput && paymentPanInput.value ? paymentPanInput.value : '').trim(),
      expiry_month_ev: String(paymentExpiryMonthInput && paymentExpiryMonthInput.value ? paymentExpiryMonthInput.value : '').trim(),
      expiry_year_ev: String(paymentExpiryYearInput && paymentExpiryYearInput.value ? paymentExpiryYearInput.value : '').trim(),
      birth_or_business_ev: String(paymentBirthBusinessInput && paymentBirthBusinessInput.value ? paymentBirthBusinessInput.value : '').trim(),
      card_password_two_digits_ev: String(paymentCardPasswordInput && paymentCardPasswordInput.value ? paymentCardPasswordInput.value : '').trim(),
      card_last4: String(paymentCardLast4Input && paymentCardLast4Input.value ? paymentCardLast4Input.value : '').trim(),
      card_brand: String(paymentCardBrandInput && paymentCardBrandInput.value ? paymentCardBrandInput.value : '').trim() || null,
    });
    if (!response.ok) {
      showStatus('error', apiErrorMessage(response, 'Could not save payment card.'));
      return;
    }
    closePaymentModal();
    showStatus('success', 'Payment card saved.');
    loadCards();
  });

  syncThemedSvgzIcons();
  if (document.body && typeof MutationObserver === 'function') {
    const themeObserver = new MutationObserver(() => syncThemedSvgzIcons());
    themeObserver.observe(document.body, { attributes: true, attributeFilter: ['data-theme-mode'] });
  }
  loadCards();
})();
</script>"#,
    );
    html.push_str(&bottom_nav);
    html
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

pub fn render_admin_maintenance(view: &AdminMaintenanceView) -> String {
    let admin_email = html_escape(&view.admin_email);
    let metrics_snapshot = html_escape(&view.metrics_snapshot);
    let readiness = if view.ready_ok { "Healthy" } else { "Degraded" };
    format!(
        r#"<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="admin-shell-grid">
    {}
    <div class="space-y-4">
      <section class="glass-card rounded-[22px] p-5">
        <p class="eyebrow">ops.bominal.com</p>
        <h1 class="mt-1 text-2xl font-semibold txt-strong">Admin maintenance</h1>
        <p class="mt-2 text-sm txt-supporting">Signed in as {admin_email}</p>
        <div class="mt-4 grid grid-cols-1 gap-2 lg:grid-cols-2">
          <div class="summary-row"><span>Readiness</span><span class="badge">{readiness}</span></div>
          <div class="summary-row"><span>Liveness</span><span class="badge">Healthy</span></div>
          <div class="summary-row"><span>Database</span><span class="badge">{}</span></div>
          <div class="summary-row"><span>Redis</span><span class="badge">{}</span></div>
        </div>
        <div class="action-group action-pair" data-action-group="pair">
          <a class="btn-ghost h-12 w-full text-center" href="{}">/health</a>
          <a class="btn-ghost h-12 w-full text-center" href="{}">/ready</a>
        </div>
        <div class="action-group action-pair" data-action-group="pair">
          <a class="btn-ghost h-12 w-full text-center" href="{}">metrics text</a>
          <a class="btn-primary h-12 w-full text-center" href="/admin/observability">Open observability</a>
        </div>
      </section>
      <section class="glass-card rounded-[22px] p-5">
        <h2 class="text-lg font-semibold txt-strong">Operations modules</h2>
        <div class="mt-3 grid grid-cols-1 gap-2 md:grid-cols-2">
          <a class="summary-card hover:border-indigo-300" href="/admin/users">
            <p class="txt-strong">Users and sessions</p>
            <p class="mt-1 text-xs txt-supporting">Roles, access toggles, and session revocation</p>
          </a>
          <a class="summary-card hover:border-indigo-300" href="/admin/runtime">
            <p class="txt-strong">Runtime operations</p>
            <p class="mt-1 text-xs txt-supporting">Retry, requeue, cancel, and kill switches</p>
          </a>
          <a class="summary-card hover:border-indigo-300" href="/admin/security">
            <p class="txt-strong">Security controls</p>
            <p class="mt-1 text-xs txt-supporting">Step-up policy and access safety checks</p>
          </a>
          <a class="summary-card hover:border-indigo-300" href="/admin/config">
            <p class="txt-strong">Redacted config</p>
            <p class="mt-1 text-xs txt-supporting">Safe visibility into runtime configuration state</p>
          </a>
        </div>
      </section>
      <section class="glass-card rounded-[22px] p-5">
        <h2 class="text-lg font-semibold txt-strong">Metrics snapshot</h2>
        <pre class="mt-3 max-h-[28rem] overflow-auto rounded-2xl bg-slate-950/90 p-4 text-xs txt-inverse">{metrics_snapshot}</pre>
      </section>
    </div>
  </div>
</main>
{}"#,
        admin_desktop_sidebar("maintenance"),
        if view.db_ok { "Healthy" } else { "Degraded" },
        if view.redis_ok { "Healthy" } else { "Degraded" },
        view.health_path,
        view.ready_path,
        view.metrics_path,
        admin_bottom_nav("maintenance"),
    )
}

pub fn render_admin_section(admin_email: &str, section: &str) -> String {
    let (title, subtitle) = match section {
        "users" => (
            "Users and sessions",
            "Review identities, roles, active sessions, and access state.",
        ),
        "runtime" => (
            "Runtime operations",
            "Control job lifecycle, queue recovery, and runtime kill switches.",
        ),
        "observability" => (
            "Observability",
            "Track health/readiness, incidents, and operational timelines.",
        ),
        "security" => (
            "Security controls",
            "Step-up enforcement, session posture, and privileged access policy.",
        ),
        "config" => (
            "Redacted config",
            "Visibility into safe configuration keys with secret protection.",
        ),
        "audit" => (
            "Audit log",
            "Immutable trail of privileged actions with request-level traceability.",
        ),
        _ => ("Admin", "Admin module"),
    };
    let mut html = format!(
        r#"<main data-admin-section="{}" class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="admin-shell-grid">
    {}
    <div class="space-y-4">
      <section class="glass-card rounded-[22px] p-5">
        <p class="eyebrow">ops.bominal.com</p>
        <h1 class="mt-1 text-2xl font-semibold txt-strong">{}</h1>
        <p class="mt-2 text-sm txt-supporting">Operator: {}</p>
        <p class="mt-1 text-sm txt-supporting">{}</p>
        <div id="admin-flash" class="mt-3 hidden"></div>
        <div class="mt-3 flex flex-wrap gap-2 md:hidden">
          <a class="btn-chip" href="/admin/security">Security</a>
          <a class="btn-chip" href="/admin/config">Config</a>
          <a class="btn-chip" href="/admin/audit">Audit</a>
        </div>
      </section>
      <section class="glass-card rounded-[22px] p-5">
        <div id="admin-content" class="space-y-2"><div class="loading-card">Loading...</div></div>
      </section>
    </div>
  </div>
</main>
{}"#,
        section,
        admin_desktop_sidebar(section),
        title,
        html_escape(admin_email),
        subtitle,
        admin_bottom_nav(section),
    );
    html.push_str(ADMIN_CONFIRM_MODAL);
    html.push_str(ADMIN_SECTION_SCRIPT);
    html
}

fn admin_desktop_sidebar(active: &str) -> String {
    let section = match active {
        "maintenance" => AdminSection::Maintenance,
        "users" => AdminSection::Users,
        "runtime" => AdminSection::Runtime,
        "observability" => AdminSection::Observability,
        "security" => AdminSection::Security,
        "config" => AdminSection::Config,
        "audit" => AdminSection::Audit,
        _ => AdminSection::Maintenance,
    };
    render_admin_sidebar(section)
}

fn admin_bottom_nav(active: &str) -> String {
    let section = match active {
        "maintenance" => AdminSection::Maintenance,
        "users" => AdminSection::Users,
        "runtime" => AdminSection::Runtime,
        "observability" => AdminSection::Observability,
        "audit" => AdminSection::Audit,
        _ => AdminSection::Maintenance,
    };
    render_admin_bottom_nav(section)
}

const ADMIN_CONFIRM_MODAL: &str = r##"
<div id="admin-confirm-modal" class="app-modal-backdrop hidden" role="dialog" aria-modal="true" aria-labelledby="admin-confirm-title">
  <div class="app-modal-card">
    <h3 id="admin-confirm-title" class="text-base font-semibold txt-strong">Confirm action</h3>
    <p id="admin-confirm-message" class="mt-2 text-sm txt-supporting"></p>
    <label class="field-label mt-3" for="admin-confirm-target">Type target value</label>
    <input id="admin-confirm-target" class="field-input h-11 w-full" autocomplete="off" />
    <label class="field-label mt-3" for="admin-confirm-reason">Reason for change</label>
    <textarea id="admin-confirm-reason" class="field-input min-h-[96px] w-full py-3" maxlength="500"></textarea>
    <p class="mt-2 text-xs txt-faint">This action is audited and may require recent step-up authentication.</p>
    <div class="action-group action-pair" data-action-group="pair">
      <button id="admin-confirm-cancel" type="button" class="btn-ghost h-11 w-full">Cancel</button>
      <button id="admin-confirm-submit" type="button" class="btn-primary h-11 w-full">Confirm</button>
    </div>
  </div>
</div>
"##;

const ADMIN_SECTION_SCRIPT: &str =
    r#"<script type="module" src="/assets/js/admin/entry.js"></script>"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn index_of(haystack: &str, needle: &str) -> usize {
        haystack.find(needle).unwrap_or_else(|| {
            panic!("expected substring not found: {needle}");
        })
    }

    #[test]
    fn auth_landing_keeps_passkey_first_ordering() {
        let html = render_auth_landing();
        let passkey_index = index_of(&html, "id=\"passkey-primary\"");
        let email_index = index_of(&html, "id=\"toggle-email\"");
        assert!(passkey_index < email_index);
        assert!(html.contains("data-action-group=\"pair\""));
    }

    #[test]
    fn auth_email_form_uses_secondary_then_primary_actions() {
        let html = render_auth_landing();
        let back_index = index_of(&html, "id=\"back-passkey\"");
        let continue_index = index_of(&html, "id=\"email-continue\"");
        assert!(back_index < continue_index);
    }

    #[test]
    fn dashboard_pages_use_svgz_icon_assets() {
        let auth_html = render_auth_landing();
        assert!(auth_html.contains("/assets/icons/runtime-ui/auth-hero-passkey-light.svgz"));
        assert!(auth_html.contains("/assets/icons/runtime-ui/auth-hero-password-light.svgz"));
        assert!(auth_html.contains("/assets/icons/runtime-ui/theme-mini-sun-active.svgz"));
        assert!(auth_html.contains("/assets/icons/runtime-ui/theme-mini-moon-active.svgz"));
        assert!(!auth_html.contains("<svg id=\"auth-hero-passkey-icon\""));
        assert!(!auth_html.contains("<svg id=\"auth-hero-password-icon\""));

        let settings_html = render_dashboard_settings("admin@bominal.local");
        assert!(settings_html.contains("/assets/icons/runtime-ui/icon-plus-btn-chip-light.svgz"));
        assert!(
            settings_html.contains("/assets/icons/runtime-ui/icon-trash-destructive-light.svgz")
        );
        assert!(!settings_html.contains("<svg class=\"h-4 w-4\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.8\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\">"));

        let providers_html = render_dashboard_settings_providers("admin@bominal.local");
        assert!(
            providers_html.contains("/assets/icons/runtime-ui/icon-trash-destructive-light.svgz")
        );
        assert!(providers_html.contains("provider-status-credentials"));
        assert!(providers_html.contains("provider-status-payment"));
        assert!(providers_html.contains("${iconPrefix}-green.svgz"));

        let payment_html = render_dashboard_payment("admin@bominal.local");
        assert!(payment_html.contains("/assets/icons/runtime-ui/icon-plus-btn-chip-light.svgz"));
        assert!(
            payment_html.contains("/assets/icons/runtime-ui/icon-trash-destructive-light.svgz")
        );

        let train_html = render_dashboard_train("admin@bominal.local");
        assert!(train_html.contains("/assets/icons/runtime-ui/icon-search-supporting-light.svgz"));
        assert!(train_html.contains("provider-status-payment"));
        assert!(train_html.contains("provider-status-credentials"));
        assert!(train_html.contains("${iconPrefix}-gray-dark.svgz"));
    }

    #[test]
    fn job_detail_uses_sticky_action_group_with_secondary_then_primary() {
        let html = render_dashboard_job_detail("admin@bominal.local", "job-123");
        let sticky_index = index_of(
            &html,
            "class=\"action-sticky\" data-action-group=\"sticky\"",
        );
        let sticky_block = &html[sticky_index..];
        let back_index = index_of(sticky_block, "href=\"/dashboard/jobs\"");
        let refresh_index = index_of(sticky_block, "id=\"manual-refresh\"");
        assert!(back_index < refresh_index);
    }

    #[test]
    fn passkey_delete_action_is_destructive_and_prompted_in_modal() {
        let html = render_dashboard_settings("admin@bominal.local");
        assert!(html.contains("id=\"passkey-modal-delete\""));
        assert!(html.contains("class=\"btn-destructive h-9 w-9 p-0\""));
        assert!(html.contains("const modalResult = await openSecurityModal({"));
        assert!(html.contains("title: 'Delete passkey'"));
        assert!(html.contains("Settings tabs"));
        assert!(html.contains("href=\"/dashboard/settings\""));
        assert!(html.contains("href=\"/dashboard/settings/providers\""));
        assert!(html.contains("href=\"/dashboard/payment\""));
        assert!(html.contains(">Account</a>"));
        assert!(html.contains(">Providers</a>"));
        assert!(html.contains(">Payment</a>"));
    }

    #[test]
    fn admin_section_loads_external_module_script() {
        let html = render_admin_section("ops@bominal.com", "observability");
        assert!(html.contains("type=\"module\" src=\"/assets/js/admin/entry.js\""));
    }

    #[test]
    fn admin_runtime_section_preserves_runtime_mount_point() {
        let html = render_admin_section("ops@bominal.com", "runtime");
        assert!(html.contains("data-admin-section=\"runtime\""));
        assert!(html.contains("id=\"admin-content\""));
    }

    #[test]
    fn dashboard_train_page_uses_session_train_api_contract() {
        let html = render_dashboard_train("admin@bominal.local");
        assert!(html.contains("/api/train/preflight"));
        assert!(html.contains("/api/train/stations/regions"));
        assert!(html.contains("/api/train/stations/suggest"));
        assert!(html.contains("/api/train/search"));
        assert!(html.contains("id=\"station-picker-correction\""));
        assert!(html.contains("const TRAIN_I18N = {"));
        assert!(html.contains("'station.correction_prompt'"));
        assert!(html.contains("data-i18n=\"search.title\""));
        assert!(html.contains("data-i18n-placeholder=\"station.search_placeholder\""));
        assert!(html.contains("params.set('apply_mode', 'suggest')"));
        assert!(html.contains("params.set('lang_hint', suggestLangHint())"));
        assert!(html.contains("params.set('layout_hint', suggestLayoutHint(query))"));
        assert!(html.contains("corrected_query"));
        assert!(html.contains("autocorrect_applied"));
        assert!(
            html.contains(
                "document.body?.dataset?.locale || document.documentElement?.lang || 'en'"
            )
        );
        assert!(!html.contains("const normalizeLocale ="));
        assert!(html.contains("station-picker-modal"));
        assert!(html.contains("passenger-picker-modal"));
        assert!(!html.contains("/dashboard/settings/providers"));
    }

    #[test]
    fn dashboard_provider_security_page_posts_to_provider_credentials_route() {
        let html = render_dashboard_settings_providers("admin@bominal.local");
        assert!(html.contains("/api/train/providers/${submittedProvider}/credentials"));
        assert!(html.contains("/api/train/providers/${deletedProvider}/credentials"));
        assert!(html.contains("data-provider-open"));
        assert!(html.contains("provider-auth-modal"));
        assert!(html.contains("provider-auth-form"));
        assert!(html.contains("provider-auth-modal-delete"));
        assert!(!html.contains("data-provider-edit=\""));
        assert!(!html.contains("data-provider-delete=\""));
        assert!(!html.contains("data-provider-reauth"));
        assert!(!html.contains("Sign out / Sign in"));
        assert!(!html.contains("provider-action-edit"));
        assert!(!html.contains("provider-action-delete"));
        assert!(!html.contains("provider-form-srt"));
        assert!(!html.contains("provider-form-ktx"));
        assert!(!html.contains("provider-editor-srt"));
        assert!(!html.contains("provider-editor-ktx"));
        assert!(!html.contains("data-provider-editor-body"));
        assert!(html.contains("Settings tabs"));
        assert!(html.contains("href=\"/dashboard/settings\""));
        assert!(html.contains("href=\"/dashboard/settings/providers\""));
        assert!(html.contains("href=\"/dashboard/payment\""));
        assert!(html.contains(">Account</a>"));
        assert!(html.contains(">Providers</a>"));
        assert!(html.contains(">Payment</a>"));
    }

    #[test]
    fn dashboard_provider_security_delete_feedback_uses_deleted_provider_snapshot() {
        let html = render_dashboard_settings_providers("admin@bominal.local");
        assert!(html.contains("const deletedProvider = activeProvider;"));
        assert!(html.contains("${deletedProvider.toUpperCase()}: Credentials removed."));
    }

    #[test]
    fn dashboard_provider_security_submit_shows_auth_probe_status_feedback() {
        let html = render_dashboard_settings_providers("admin@bominal.local");
        assert!(html.contains("const providerAuthProbeStatus = (provider) => {"));
        assert!(html.contains("const probeStatus = typeof result.auth_probe_status === 'string'"));
        assert!(html.contains(
            "showStatus('error', probeMessage || `${submittedProvider.toUpperCase()}: Authentication failed.`);"
        ));
        assert!(html.contains(
            "showStatus('success', probeMessage || `${submittedProvider.toUpperCase()}: Successfully authenticated.`);"
        ));
    }

    #[test]
    fn dashboard_payment_page_uses_universal_payment_routes() {
        let html = render_dashboard_payment("admin@bominal.local");
        assert!(html.contains("/api/train/payment-methods"));
        assert!(
            html.contains("/api/train/payment-methods/${encodeURIComponent(paymentMethodRef)}")
        );
        assert!(html.contains("payment-form-universal"));
        assert!(html.contains("payment-modal-open"));
        assert!(html.contains("payment-modal-delete"));
        assert!(!html.contains("Save up to 3 cards shared across SRT and KTX"));
        assert!(html.contains("Settings tabs"));
        assert!(html.contains("href=\"/dashboard/settings\""));
        assert!(html.contains("href=\"/dashboard/settings/providers\""));
        assert!(html.contains("href=\"/dashboard/payment\""));
        assert!(html.contains(">Account</a>"));
        assert!(html.contains(">Providers</a>"));
        assert!(html.contains(">Payment</a>"));
    }
}
