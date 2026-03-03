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
    <p class="eyebrow">bominal</p>
    <h1 class="mt-1 text-xl font-semibold text-slate-900 dark:text-slate-100">{title}</h1>
    <p class="mt-1 text-sm text-slate-600 dark:text-slate-300">{subtitle}</p>
  </div>
</header>"#
    )
}

pub fn render_auth_landing() -> String {
    let html = r#"
<main class="mx-auto flex min-h-screen w-full max-w-[480px] flex-col px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <section class="glass-card rounded-[22px] p-6 md:p-8">
    <p class="eyebrow">bominal authentication</p>
    <h1 class="mt-2 text-3xl font-semibold text-slate-900 dark:text-slate-100 md:text-5xl">Sign in securely</h1>
    <p class="mt-3 text-sm leading-6 text-slate-600 dark:text-slate-300">
      Authenticate with passkey first. Fallback to email/password when needed.
    </p>
    <button id="passkey-primary" class="btn-primary mt-6 h-12 w-full">Authenticate with passkey</button>
    <button id="toggle-email" class="btn-ghost mt-3 h-12 w-full">Sign in with email/password</button>
    <div id="auth-error" class="mt-3 hidden rounded-2xl border border-rose-300 bg-rose-50 px-3 py-2 text-sm text-rose-700"></div>
  </section>

  <section id="email-panel" class="glass-card mt-4 hidden rounded-[22px] p-6">
    <h2 class="text-lg font-semibold text-slate-900 dark:text-slate-100">Email and password</h2>
    <form id="email-signin-form" class="mt-4 space-y-3">
      <label class="field-label" for="signin-email">Email</label>
      <input id="signin-email" type="email" autocomplete="email" class="field-input h-12 w-full" />
      <label class="field-label" for="signin-password">Password</label>
      <input id="signin-password" type="password" autocomplete="current-password" class="field-input h-12 w-full" />
      <button type="submit" class="btn-primary h-12 w-full">Continue</button>
    </form>
  </section>

  <section class="glass-card mt-4 rounded-[22px] p-5">
    <div class="flex items-center justify-between">
      <span class="text-sm text-slate-600 dark:text-slate-300">Theme</span>
      <div class="flex gap-2">
        <button class="btn-chip" data-theme="system">System</button>
        <button class="btn-chip" data-theme="light">Light</button>
        <button class="btn-chip" data-theme="dark">Dark</button>
      </div>
    </div>
  </section>
</main>

<script>
(() => {
  const passkeyBtn = document.getElementById('passkey-primary');
  const toggleEmailBtn = document.getElementById('toggle-email');
  const emailPanel = document.getElementById('email-panel');
  const emailForm = document.getElementById('email-signin-form');
  const authError = document.getElementById('auth-error');
  const themeButtons = Array.from(document.querySelectorAll('[data-theme]'));

  const showError = (message) => {
    authError.textContent = message;
    authError.classList.remove('hidden');
  };

  const clearError = () => {
    authError.textContent = '';
    authError.classList.add('hidden');
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
      showError(String(err));
    }
  };

  passkeyBtn.addEventListener('click', passkeyAuth);
  toggleEmailBtn.addEventListener('click', () => emailPanel.classList.toggle('hidden'));

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

  themeButtons.forEach((button) => {
    button.addEventListener('click', async () => {
      await requestJson('/api/ui/theme', 'POST', { mode: button.dataset.theme });
      window.location.reload();
    });
  });
})();
</script>
"#;
    html.to_string()
}

pub fn render_dashboard_overview(email: &str) -> String {
    let topbar = app_shell_topbar("Dashboard", &format!("Signed in as {}", html_escape(email)));
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <section class="glass-card rounded-[22px] p-5">
    <h2 class="text-lg font-semibold text-slate-900 dark:text-slate-100">Operational summary</h2>
    <div id="dashboard-summary" class="mt-4 space-y-2">
      <div class="summary-row"><span>Total jobs</span><span>--</span></div>
      <div class="summary-row"><span>Queued</span><span>--</span></div>
      <div class="summary-row"><span>Running</span><span>--</span></div>
      <div class="summary-row"><span>Failed</span><span>--</span></div>
    </div>
    <a href="/dashboard/jobs" class="btn-primary mt-4 inline-flex h-12 w-full items-center justify-center">View Jobs</a>
  </section>
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
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <section class="glass-card rounded-[22px] p-5">
    <h2 class="text-lg font-semibold text-slate-900 dark:text-slate-100">My runtime jobs</h2>
    <div id="jobs-list" class="mt-4 space-y-2"><div class="loading-card">Loading jobs...</div></div>
  </section>
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
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-28 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <section class="glass-card rounded-[22px] p-5">
    <div class="summary-row"><span>Job ID</span><span id="job-id">{}</span></div>
    <div class="summary-row"><span>Status</span><span id="job-status">--</span></div>
    <div id="events" class="mt-4 space-y-2"></div>
  </section>
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
</script>"#,
        html_escape(job_id),
    )
}

pub fn render_dashboard_security(email: &str) -> String {
    let topbar = app_shell_topbar("Security", &format!("Signed in as {}", html_escape(email)));
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <section class="glass-card rounded-[22px] p-5">
    <h2 class="text-lg font-semibold text-slate-900 dark:text-slate-100">Passkeys</h2>
    <div id="passkeys-list" class="mt-4 space-y-2"><div class="loading-card">Loading passkeys...</div></div>
  </section>
</main>
<nav class="bottom-nav">
  <a href="/dashboard">Home</a>
  <a href="/dashboard/jobs">Jobs</a>
  <a href="/dashboard/security" class="active">Security</a>
</nav>
<script>
(() => {{
  const list = document.getElementById('passkeys-list');
  const load = () => fetch('/api/auth/passkeys', {{ headers: {{ Accept: 'application/json' }} }})
    .then((res) => res.json().then((json) => [res.ok, json]))
    .then(([ok, data]) => {{
      if (!ok) {{
        list.innerHTML = `<div class="error-card">Failed to load passkeys. request_id: ${{data.request_id || 'n/a'}}</div>`;
        return;
      }}
      const passkeys = data.passkeys || [];
      if (!passkeys.length) {{
        list.innerHTML = '<div class="empty-card">No passkeys registered.</div>';
        return;
      }}
      list.innerHTML = passkeys.map((item) => `
        <div class="summary-card">
          <div class="summary-row"><span>${{item.friendly_name || 'Unnamed passkey'}}</span><span>${{item.credential_id}}</span></div>
          <button class="btn-ghost h-11 w-full mt-2" data-remove="${{item.credential_id}}">Remove</button>
        </div>
      `).join('');
      list.querySelectorAll('[data-remove]').forEach((button) => {{
        button.addEventListener('click', () => {{
          fetch(`/api/auth/passkeys/${{button.dataset.remove}}`, {{ method: 'DELETE' }}).then(load);
        }});
      }});
    }});
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
    <h1 class="mt-1 text-2xl font-semibold text-slate-900 dark:text-slate-100">Admin maintenance</h1>
    <p class="mt-2 text-sm text-slate-600 dark:text-slate-300">Signed in as {admin_email}</p>
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
    <h2 class="text-lg font-semibold text-slate-900 dark:text-slate-100">Metrics snapshot</h2>
    <pre class="mt-3 max-h-[28rem] overflow-auto rounded-2xl bg-slate-950/90 p-4 text-xs text-slate-100">{metrics_snapshot}</pre>
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
    <h1 class="mt-1 text-2xl font-semibold text-slate-900 dark:text-slate-100">{}</h1>
    <p class="mt-2 text-sm text-slate-600 dark:text-slate-300">Operator: {}</p>
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
      content.innerHTML = `<pre class="rounded-2xl bg-slate-950/90 p-4 text-xs text-slate-100 overflow-auto">${{JSON.stringify(data, null, 2)}}</pre>`;
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
