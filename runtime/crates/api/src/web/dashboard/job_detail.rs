use super::super::{app_shell_topbar, dashboard_desktop_sidebar, html_escape};
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
