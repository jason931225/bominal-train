fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn app_shell_topbar(title: &str, subtitle: &str) -> String {
    format!(
        r#"<header class="mx-auto w-full max-w-[480px] px-4 pt-4 md:max-w-7xl md:px-6">
  <div class="glass-card rounded-[20px] p-4">
    <div class="flex items-start justify-between gap-3">
      <div>
        <p class="eyebrow">bominal</p>
        <h1 class="mt-1 text-xl font-semibold txt-strong">{title}</h1>
      </div>
      <button
        type="button"
        class="theme-mini-switch theme-inline-switch"
        data-theme-toggle
        data-theme-toggle-compact
        aria-label="Theme toggle"
      >
        <svg class="theme-mini-icon theme-mini-icon-sun" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="12" cy="12" r="4"></circle>
          <line x1="12" y1="2" x2="12" y2="4.5"></line>
          <line x1="12" y1="19.5" x2="12" y2="22"></line>
          <line x1="4.9" y1="4.9" x2="6.7" y2="6.7"></line>
          <line x1="17.3" y1="17.3" x2="19.1" y2="19.1"></line>
          <line x1="2" y1="12" x2="4.5" y2="12"></line>
          <line x1="19.5" y1="12" x2="22" y2="12"></line>
          <line x1="4.9" y1="19.1" x2="6.7" y2="17.3"></line>
          <line x1="17.3" y1="6.7" x2="19.1" y2="4.9"></line>
        </svg>
        <svg class="theme-mini-icon theme-mini-icon-moon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21 12.8A8.5 8.5 0 1 1 11.2 3a6.8 6.8 0 0 0 9.8 9.8z"></path>
        </svg>
        <span class="theme-mini-thumb" aria-hidden="true"></span>
      </button>
    </div>
    <p class="mt-1 text-sm txt-supporting">{subtitle}</p>
  </div>
</header>"#
    )
}

fn dashboard_desktop_sidebar(active: &str) -> String {
    format!(
        r#"<aside class="hidden md:sticky md:top-6 md:block md:self-start">
  <div class="glass-card rounded-[22px] p-3">
    <p class="eyebrow px-3 pt-1">navigation</p>
    <nav class="mt-2 space-y-1">
      <a href="/dashboard" class="desktop-side-link {}">Overview</a>
      <a href="/dashboard/jobs" class="desktop-side-link {}">Jobs</a>
      <a href="/dashboard/security" class="desktop-side-link {}">Security</a>
    </nav>
  </div>
</aside>"#,
        if active == "home" { "active" } else { "" },
        if active == "jobs" { "active" } else { "" },
        if active == "security" { "active" } else { "" },
    )
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
            <svg class="auth-hero-icon-main" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <rect x="6.5" y="10.25" width="11" height="9" rx="2.25"></rect>
              <path d="M8.75 10.25V7.9a3.25 3.25 0 1 1 6.5 0v2.35"></path>
            </svg>
          </div>
        </div>

        <div class="auth-action-region mt-6">
          <div id="auth-passkey-view" class="auth-pane" aria-hidden="false">
            <button id="passkey-primary" class="btn-primary h-12 w-full">Authenticate with passkey</button>
            <button id="toggle-email" class="btn-ghost mt-3 h-12 w-full">Sign in with email</button>
          </div>

          <div id="auth-email-view" class="auth-pane hidden" aria-hidden="true">
          <form id="email-signin-form" class="space-y-3">
            <label class="field-label" for="signin-email">Email</label>
            <input id="signin-email" type="email" autocomplete="email" class="field-input h-12 w-full" />
            <label class="field-label" for="signin-password">Password</label>
            <input id="signin-password" type="password" autocomplete="current-password" class="field-input h-12 w-full" />
            <button type="submit" class="btn-primary h-12 w-full">Continue</button>
          </form>
          <button id="back-passkey" type="button" class="btn-ghost mt-3 h-12 w-full">Back to passkey</button>
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
        <svg class="theme-mini-icon theme-mini-icon-sun" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="12" cy="12" r="4"></circle>
          <line x1="12" y1="2" x2="12" y2="4.5"></line>
          <line x1="12" y1="19.5" x2="12" y2="22"></line>
          <line x1="4.9" y1="4.9" x2="6.7" y2="6.7"></line>
          <line x1="17.3" y1="17.3" x2="19.1" y2="19.1"></line>
          <line x1="2" y1="12" x2="4.5" y2="12"></line>
          <line x1="19.5" y1="12" x2="22" y2="12"></line>
          <line x1="4.9" y1="19.1" x2="6.7" y2="17.3"></line>
          <line x1="17.3" y1="6.7" x2="19.1" y2="4.9"></line>
        </svg>
        <svg class="theme-mini-icon theme-mini-icon-moon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21 12.8A8.5 8.5 0 1 1 11.2 3a6.8 6.8 0 0 0 9.8 9.8z"></path>
        </svg>
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
  const emailForm = document.getElementById('email-signin-form');
  const authError = document.getElementById('auth-error');
  const reduceMotionQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
  const fadeDurationMs = reduceMotionQuery.matches ? 0 : 120;
  let swapInProgress = false;

  const showError = (message) => {
    authError.textContent = message;
    authError.classList.remove('hidden');
  };

  const clearError = () => {
    authError.textContent = '';
    authError.classList.add('hidden');
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
    if (!passkeyView.classList.contains('hidden')) return;
    switchView('passkey');
  };

  const showEmailView = () => {
    clearError();
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

  const passkeyAuth = async () => {
    clearError();
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
    try {
      const options = structuredClone(start.body.options);
      options.publicKey.challenge = b64urlToBuffer(options.publicKey.challenge);
      if (Array.isArray(options.publicKey.allowCredentials)) {
        options.publicKey.allowCredentials = options.publicKey.allowCredentials.map((item) => ({ ...item, id: b64urlToBuffer(item.id) }));
      }
      const credential = await navigator.credentials.get(options);
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
      showError('Passkey authentication failed. Use email/password if needed.');
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

})();
</script>
"#;
    html.to_string()
}

pub fn render_dashboard_overview(email: &str) -> String {
    let topbar = app_shell_topbar("Dashboard", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("home");
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
      <a href="/dashboard/jobs" class="btn-primary mt-4 inline-flex h-12 w-full items-center justify-center">View Jobs</a>
    </section>
  </div>
</main>

<nav class="bottom-nav">
  <a href="/dashboard" class="active">Home</a>
  <a href="/dashboard/jobs">Jobs</a>
  <a href="/dashboard/security">Security</a>
</nav>

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
<nav class="bottom-nav">
  <a href="/dashboard">Home</a>
  <a href="/dashboard/jobs" class="active">Jobs</a>
  <a href="/dashboard/security">Security</a>
</nav>
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
<div class="sticky-action-bar">
  <a href="/dashboard/jobs" class="btn-ghost h-12 flex-1 text-center leading-[3rem]">Back</a>
  <button id="manual-refresh" class="btn-primary h-12 flex-1">Refresh</button>
</div>
<script>
(() => {{
  const jobId = document.getElementById('job-id').textContent.trim();
  const statusEl = document.getElementById('job-status');
  const eventsEl = document.getElementById('events');
  const refreshBtn = document.getElementById('manual-refresh');
  let lastEventId = 0;
  let fallbackInterval = null;

  const renderEvents = (events) => {{
    if (!events.length && !eventsEl.innerHTML.trim()) {{
      eventsEl.innerHTML = '<div class="empty-card">No events yet.</div>';
      return;
    }}
    events.forEach((event) => {{
      lastEventId = Math.max(lastEventId, Number(event.id || 0));
      const node = document.createElement('div');
      node.className = 'summary-card';
      node.innerHTML = `<div class="summary-row"><span>${{event.event_type}}</span><span>${{event.id}}</span></div>`;
      eventsEl.prepend(node);
    }});
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

  const pollEvents = () => fetch(`/api/dashboard/jobs/${{jobId}}/events?since_id=${{lastEventId}}`, {{ headers: {{ Accept: 'application/json' }} }})
    .then((res) => res.json())
    .then((data) => renderEvents(data.events || []))
    .catch(() => {{}});

  const startFallback = () => {{
    if (fallbackInterval) return;
    fallbackInterval = setInterval(pollEvents, 10000);
  }};

  const startSse = () => {{
    if (!window.EventSource) {{
      startFallback();
      return;
    }}
    const source = new EventSource(`/api/dashboard/jobs/${{jobId}}/events/stream?since_id=${{lastEventId}}`);
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
    pollEvents();
  }});

  loadJob();
  pollEvents();
  startSse();
}})();
</script>"#
    )
}

pub fn render_dashboard_security(email: &str) -> String {
    let topbar = app_shell_topbar("Security", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("security");
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">
    {sidebar}
    <section class="glass-card rounded-[22px] p-5">
      <h2 class="text-lg font-semibold txt-strong">Account settings</h2>

      <div class="mt-4 space-y-4">
        <section class="summary-card p-4">
          <div class="flex items-center justify-between gap-2">
            <h3 class="text-sm font-semibold txt-strong">Passkeys</h3>
          </div>
          <p class="mt-1 text-sm txt-supporting">Register a passkey for faster and safer sign-in.</p>
          <div id="passkey-status" class="mt-3 hidden"></div>
          <div id="passkeys-list" class="mt-4 space-y-2"><div class="loading-card">Loading passkeys...</div></div>
          <div class="mt-3 grid grid-cols-1 gap-2 sm:grid-cols-2">
            <button id="create-passkey" class="btn-primary h-11 px-4">Create passkey</button>
            <button id="delete-passkey" type="button" class="btn-ghost h-11 w-full" disabled>Delete selected passkey</button>
          </div>
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
            <button type="submit" class="btn-primary h-12 w-full">Update password</button>
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
          <div class="mt-4 grid grid-cols-2 gap-2">
            <button id="security-modal-cancel" type="button" class="btn-ghost h-11 w-full">Cancel</button>
            <button id="security-modal-confirm" type="button" class="btn-primary h-11 w-full">Confirm</button>
          </div>
        </div>
      </div>
    </section>
  </div>
</main>
<nav class="bottom-nav">
  <a href="/dashboard">Home</a>
  <a href="/dashboard/jobs">Jobs</a>
  <a href="/dashboard/security" class="active">Security</a>
</nav>
<script>
(() => {{
  const list = document.getElementById('passkeys-list');
  const passkeyStatus = document.getElementById('passkey-status');
  const createPasskeyButton = document.getElementById('create-passkey');
  const deletePasskeyButton = document.getElementById('delete-passkey');
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
  let modalResolver = null;

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

  const closeSecurityModal = (result) => {{
    if (!modalResolver || !securityModal) {{
      return;
    }}
    const resolve = modalResolver;
    modalResolver = null;
    securityModal.classList.add('hidden');
    securityModal.setAttribute('aria-hidden', 'true');
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
    securityModal.classList.remove('hidden');
    securityModal.setAttribute('aria-hidden', 'false');
    modalResolver = resolve;

    requestAnimationFrame(() => {{
      if (options.withPassword) {{
        securityModalInput.focus();
      }} else {{
        securityModalConfirm.focus();
      }}
    }});
  }});

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
    deletePasskeyButton.disabled = !selectedCredentialId;
  }};

  const load = () => fetch('/api/auth/passkeys', {{ headers: {{ Accept: 'application/json' }} }})
    .then((res) => res.json().then((json) => [res.ok, json]))
    .then(([ok, data]) => {{
      if (!ok) {{
        selectedCredentialId = null;
        syncPasskeySelection();
        list.innerHTML = `<div class="error-card">Failed to load passkeys. request_id: ${{data.request_id || 'n/a'}}</div>`;
        return;
      }}
      const passkeys = data.passkeys || [];
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
          <div class="summary-row"><span>${{item.friendly_name || 'Unnamed passkey'}}</span><span>${{item.credential_id}}</span></div>
        </button>
      `).join('');
      list.querySelectorAll('[data-credential-id]').forEach((card) => {{
        card.addEventListener('click', () => {{
          selectedCredentialId = card.dataset.credentialId || null;
          syncPasskeySelection();
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
    deletePasskeyButton.disabled = true;
    const result = await requestJson(`/api/auth/passkeys/${{selectedCredentialId}}`, 'DELETE');
    if (!result.ok) {{
      const requestId = result.body && result.body.request_id ? ` (request_id: ${{result.body.request_id}})` : '';
      showStatus(passkeyStatus, `Passkey deletion failed${{requestId}}`, 'error');
      syncPasskeySelection();
      return;
    }}
    selectedCredentialId = null;
    showStatus(passkeyStatus, 'Passkey deleted successfully.', 'success');
    load();
  }};

  const createPasskey = async () => {{
    clearStatus(passkeyStatus);
    if (!window.PublicKeyCredential || !navigator.credentials) {{
      showStatus(passkeyStatus, 'Passkeys are unavailable in this browser.', 'error');
      return;
    }}

    createPasskeyButton.disabled = true;
    try {{
      const start = await requestJson('/api/auth/passkeys/register/start', 'POST', {{
        friendly_name: 'This device',
      }});
      if (!start.ok || !start.body || !start.body.options || !start.body.flow_id) {{
        const requestId = start.body && start.body.request_id ? ` (request_id: ${{start.body.request_id}})` : '';
        showStatus(passkeyStatus, `Passkey setup failed${{requestId}}`, 'error');
        return;
      }}

      const options = structuredClone(start.body.options);
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
      if (err && typeof err === 'object' && err.name === 'SecurityError') {{
        showStatus(passkeyStatus, 'Passkeys are unavailable on this host. Use localhost for WebAuthn.', 'error');
      }} else {{
        showStatus(passkeyStatus, 'Passkey creation failed.', 'error');
      }}
    }} finally {{
      createPasskeyButton.disabled = false;
    }}
  }};

  createPasskeyButton.addEventListener('click', createPasskey);
  deletePasskeyButton.addEventListener('click', deleteSelectedPasskey);

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
    if (!securityModal || securityModal.classList.contains('hidden')) {{
      return;
    }}
    if (event.key === 'Escape') {{
      event.preventDefault();
      closeSecurityModal({{ confirmed: false, value: '' }});
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
  load();
}})();
</script>"#
    )
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
  <section class="glass-card rounded-[22px] p-5">
    <p class="eyebrow">ops.bominal.com</p>
    <h1 class="mt-1 text-2xl font-semibold txt-strong">Admin maintenance</h1>
    <p class="mt-2 text-sm txt-supporting">Signed in as {admin_email}</p>
    <div class="summary-row mt-4"><span>Readiness</span><span class="badge">{readiness}</span></div>
    <div class="summary-row"><span>Database</span><span class="badge">{}</span></div>
    <div class="summary-row"><span>Redis</span><span class="badge">{}</span></div>
    <div class="mt-4 grid grid-cols-1 gap-2">
      <a class="btn-ghost h-12 text-center leading-[3rem]" href="{}">/health</a>
      <a class="btn-ghost h-12 text-center leading-[3rem]" href="{}">/ready</a>
      <a class="btn-ghost h-12 text-center leading-[3rem]" href="{}">metrics text</a>
    </div>
  </section>
  <section class="glass-card mt-4 rounded-[22px] p-5">
    <h2 class="text-lg font-semibold txt-strong">Metrics snapshot</h2>
    <pre class="mt-3 max-h-[28rem] overflow-auto rounded-2xl bg-slate-950/90 p-4 text-xs txt-inverse">{metrics_snapshot}</pre>
  </section>
</main>
{}"#,
        if view.db_ok { "Healthy" } else { "Degraded" },
        if view.redis_ok { "Healthy" } else { "Degraded" },
        view.health_path,
        view.ready_path,
        view.metrics_path,
        admin_bottom_nav("maintenance"),
    )
}

pub fn render_admin_section(admin_email: &str, section: &str) -> String {
    let title = match section {
        "users" => "Users and sessions",
        "runtime" => "Runtime operations",
        "observability" => "Observability",
        "security" => "Security controls",
        "config" => "Redacted config",
        "audit" => "Audit log",
        _ => "Admin",
    };
    format!(
        r#"<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <section class="glass-card rounded-[22px] p-5">
    <p class="eyebrow">ops.bominal.com</p>
    <h1 class="mt-1 text-2xl font-semibold txt-strong">{}</h1>
    <p class="mt-2 text-sm txt-supporting">Operator: {}</p>
    <div id="admin-content" class="mt-4 space-y-2"><div class="loading-card">Loading...</div></div>
  </section>
</main>
{}
<script>
(() => {{
  const section = '{}';
  const content = document.getElementById('admin-content');
  const map = {{
    users: '/api/admin/users',
    runtime: '/api/admin/runtime/jobs',
    observability: '/api/admin/observability/events',
    security: '/api/admin/sessions',
    config: '/api/admin/config/redacted',
    audit: '/api/admin/audit',
  }};
  fetch(map[section], {{ headers: {{ Accept: 'application/json' }} }})
    .then((res) => res.json().then((json) => [res.ok, json]))
    .then(([ok, data]) => {{
      if (!ok) {{
        content.innerHTML = `<div class="error-card">Request failed. request_id: ${{data.request_id || 'n/a'}}</div>`;
        return;
      }}
      content.innerHTML = `<pre class="rounded-2xl bg-slate-950/90 p-4 text-xs txt-inverse overflow-auto">${{JSON.stringify(data, null, 2)}}</pre>`;
    }})
    .catch((err) => {{
      content.innerHTML = `<div class="error-card">${{String(err)}}</div>`;
    }});
}})();
</script>"#,
        title,
        html_escape(admin_email),
        admin_bottom_nav(section),
        section,
    )
}

fn admin_bottom_nav(active: &str) -> String {
    format!(
        r#"<nav class="bottom-nav">
  <a href="/admin/maintenance" class="{}">Maint</a>
  <a href="/admin/users" class="{}">Users</a>
  <a href="/admin/runtime" class="{}">Runtime</a>
  <a href="/admin/observability" class="{}">Obs</a>
  <a href="/admin/audit" class="{}">Audit</a>
</nav>"#,
        if active == "maintenance" {
            "active"
        } else {
            ""
        },
        if active == "users" { "active" } else { "" },
        if active == "runtime" { "active" } else { "" },
        if active == "observability" {
            "active"
        } else {
            ""
        },
        if active == "audit" { "active" } else { "" },
    )
}
