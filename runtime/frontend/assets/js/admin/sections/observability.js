import { appendQuery, asText, escapeHtml, formatDate, toLower } from "../common/utils.js";
import { itemsFromEnvelope, pageFromEnvelope } from "../common/pagination.js";

const EVENT_LIMIT = 40;
const INCIDENT_LIMIT = 25;

const ensureLightweightCharts = async () => {
  if (window.LightweightCharts) return window.LightweightCharts;
  if (!window.__bominalLwPromise) {
    window.__bominalLwPromise = new Promise((resolve, reject) => {
      const script = document.createElement("script");
      script.src = "/assets/lightweight-charts.standalone.production.js";
      script.async = true;
      script.onload = () => resolve(window.LightweightCharts);
      script.onerror = () => reject(new Error("lightweight chart load failed"));
      document.head.appendChild(script);
    });
  }
  return window.__bominalLwPromise;
};

export const renderObservabilitySection = async (ctx) => {
  const state = {
    summary: null,
    points: [],
    events: {
      items: [],
      nextCursor: null,
      hasMore: false,
      filters: { q: "", source: "", event_type: "", target_id: "" },
    },
    incidents: {
      items: [],
      nextCursor: null,
      hasMore: false,
      filters: { q: "", status: "", severity: "" },
    },
    streamDisabled: false,
  };

  let eventsStream = null;
  let chart = null;
  let chartResizeHandler = null;
  const closeStream = () => {
    if (eventsStream) {
      eventsStream.close();
      eventsStream = null;
    }
  };
  const teardownChart = () => {
    if (chartResizeHandler) {
      window.removeEventListener("resize", chartResizeHandler);
      chartResizeHandler = null;
    }
    if (chart?.remove) {
      chart.remove();
    }
    chart = null;
  };
  ctx.registerCleanup(() => {
    closeStream();
    teardownChart();
  });

  const fetchSummary = async () => {
    const [summaryResult, timeseriesResult] = await Promise.all([
      ctx.requestJson("/api/admin/maintenance/metrics/summary"),
      ctx.requestJson("/api/admin/observability/timeseries?window_minutes=240"),
    ]);
    if (!summaryResult.ok) throw new Error(ctx.errorMessage(summaryResult));
    if (!timeseriesResult.ok) throw new Error(ctx.errorMessage(timeseriesResult));
    state.summary = summaryResult.body || {};
    state.points = Array.isArray(timeseriesResult.body?.points) ? timeseriesResult.body.points : [];
  };

  const fetchEvents = async ({ reset }) => {
    const result = await ctx.requestJson(
      appendQuery("/api/admin/observability/events", {
        limit: EVENT_LIMIT,
        cursor: reset ? null : state.events.nextCursor,
        q: state.events.filters.q || null,
        source: state.events.filters.source || null,
        event_type: state.events.filters.event_type || null,
        target_id: state.events.filters.target_id || null,
      }),
    );
    if (!result.ok) throw new Error(ctx.errorMessage(result));
    const items = itemsFromEnvelope(result.body);
    const page = pageFromEnvelope(result.body);
    state.events.items = reset ? items : state.events.items.concat(items);
    state.events.nextCursor = page.nextCursor;
    state.events.hasMore = page.hasMore;
  };

  const fetchIncidents = async ({ reset }) => {
    const result = await ctx.requestJson(
      appendQuery("/api/admin/incidents", {
        limit: INCIDENT_LIMIT,
        cursor: reset ? null : state.incidents.nextCursor,
        q: state.incidents.filters.q || null,
        status: state.incidents.filters.status || null,
        severity: state.incidents.filters.severity || null,
      }),
    );
    if (!result.ok) throw new Error(ctx.errorMessage(result));
    const items = itemsFromEnvelope(result.body);
    const page = pageFromEnvelope(result.body);
    state.incidents.items = reset ? items : state.incidents.items.concat(items);
    state.incidents.nextCursor = page.nextCursor;
    state.incidents.hasMore = page.hasMore;
  };

  const renderChart = async () => {
    const chartEl = document.getElementById("obs-timeseries");
    if (!chartEl) {
      teardownChart();
      return;
    }
    if (!state.points.length) {
      teardownChart();
      chartEl.innerHTML = '<div class="empty-card">No timeseries data in this window.</div>';
      return;
    }
    try {
      teardownChart();
      const lw = await ensureLightweightCharts();
      if (!lw?.createChart) return;
      chartEl.innerHTML = "";
      chart = lw.createChart(chartEl, {
        width: chartEl.clientWidth || 320,
        height: 210,
        layout: {
          background: { color: "transparent" },
          textColor: getComputedStyle(document.body).getPropertyValue("--text-supporting")
            ? `rgb(${getComputedStyle(document.body).getPropertyValue("--text-supporting")})`
            : "#64748b",
        },
        grid: {
          vertLines: { color: "rgba(148,163,184,0.2)" },
          horzLines: { color: "rgba(148,163,184,0.2)" },
        },
        rightPriceScale: { borderVisible: false },
        timeScale: { borderVisible: false, timeVisible: true, secondsVisible: false },
      });
      const totalSeries = chart.addLineSeries({ color: "#635bff", lineWidth: 2 });
      const errorSeries = chart.addLineSeries({ color: "#ef4444", lineWidth: 2 });
      totalSeries.setData(
        state.points.map((point) => ({
          time: Math.floor(new Date(point.bucket).getTime() / 1000),
          value: Number(point.total_events || 0),
        })),
      );
      errorSeries.setData(
        state.points.map((point) => ({
          time: Math.floor(new Date(point.bucket).getTime() / 1000),
          value: Number(point.error_events || 0),
        })),
      );
      chart.timeScale().fitContent();
      chartResizeHandler = () => {
        chart.applyOptions({ width: chartEl.clientWidth || 320 });
      };
      window.addEventListener("resize", chartResizeHandler);
    } catch (_error) {
      chartEl.innerHTML = '<div class="empty-card">Chart unavailable in this environment.</div>';
    }
  };

  const render = () => {
    const summary = state.summary || {};
    const eventRows = state.events.items
      .map(
        (event) => `
          <article class="summary-row">
            <span class="truncate">${escapeHtml(event.source)} · ${escapeHtml(event.event_type)}</span>
            <span class="text-xs txt-supporting">${escapeHtml(formatDate(event.occurred_at))}</span>
          </article>
        `,
      )
      .join("");

    const incidentRows = state.incidents.items
      .map(
        (incident) => `
          <article class="admin-row" data-incident-id="${escapeHtml(incident.id)}">
            <div class="min-w-0">
              <p class="truncate text-sm font-semibold txt-strong">${escapeHtml(incident.title)}</p>
              <p class="mt-1 text-xs txt-supporting">${escapeHtml(incident.severity)} · ${escapeHtml(incident.status)} · ${escapeHtml(formatDate(incident.opened_at))}</p>
            </div>
            <div class="admin-row-actions">
              <button class="btn-ghost h-10 w-full md:w-auto" data-incident-status="monitoring">Monitoring</button>
              <button class="btn-ghost h-10 w-full md:w-auto" data-incident-status="resolved">Resolve</button>
            </div>
          </article>
        `,
      )
      .join("");

    ctx.content.innerHTML = `
      <section class="grid grid-cols-1 gap-2 md:grid-cols-2">
        <div class="summary-row"><span>Readiness</span><span class="badge">${summary.readiness_ok ? "Healthy" : "Degraded"}</span></div>
        <div class="summary-row"><span>Error rate</span><span>${asText(summary.error_rate, "0")}</span></div>
        <div class="summary-row"><span>Average latency (ms)</span><span>${asText(summary.avg_latency_ms, "n/a")}</span></div>
        <div class="summary-row"><span>Total requests</span><span>${asText(summary.request_total, "n/a")}</span></div>
      </section>

      <section class="pt-2">
        <h2 class="text-base font-semibold txt-strong">Events over time</h2>
        <div id="obs-timeseries" class="mt-2 h-56 rounded-2xl border border-slate-200/70 bg-white/55 p-2"></div>
      </section>

      <section class="space-y-2 pt-2">
        <h2 class="text-base font-semibold txt-strong">Incident workflow</h2>
        <form id="incident-create-form" class="grid grid-cols-1 gap-2 md:grid-cols-2">
          <input class="field-input h-11" name="title" placeholder="Incident title" required minlength="3" maxlength="140" />
          <select class="field-input h-11" name="severity">
            <option value="sev1">sev1</option>
            <option value="sev2">sev2</option>
            <option value="sev3" selected>sev3</option>
            <option value="sev4">sev4</option>
          </select>
          <input class="field-input h-11 md:col-span-2" name="summary" placeholder="Summary (optional)" maxlength="600" />
          <input class="field-input h-11 md:col-span-2" name="reason" placeholder="Reason for opening incident" required minlength="8" />
          <button class="btn-primary h-11 w-full md:w-auto" type="submit">Open incident</button>
        </form>
        <form id="incident-filters" class="grid grid-cols-1 gap-2 pt-2 md:grid-cols-4">
          <input id="incident-filter-q" class="field-input h-10 md:col-span-2" placeholder="Search incidents" value="${escapeHtml(state.incidents.filters.q)}" />
          <input id="incident-filter-status" class="field-input h-10" placeholder="status" value="${escapeHtml(state.incidents.filters.status)}" />
          <input id="incident-filter-severity" class="field-input h-10" placeholder="severity" value="${escapeHtml(state.incidents.filters.severity)}" />
        </form>
        ${incidentRows || '<div class="empty-card">No incidents.</div>'}
        ${
          state.incidents.hasMore
            ? '<button type="button" id="incidents-load-more" class="btn-ghost h-10 w-full">Load more incidents</button>'
            : ""
        }
      </section>

      <section class="space-y-2 pt-2">
        <h2 class="text-base font-semibold txt-strong">Events timeline</h2>
        <form id="obs-events-filters" class="grid grid-cols-1 gap-2 md:grid-cols-4">
          <input id="obs-filter-q" class="field-input h-10 md:col-span-2" placeholder="Search events" value="${escapeHtml(state.events.filters.q)}" />
          <input id="obs-filter-source" class="field-input h-10" placeholder="source" value="${escapeHtml(state.events.filters.source)}" />
          <input id="obs-filter-type" class="field-input h-10" placeholder="event_type" value="${escapeHtml(state.events.filters.event_type)}" />
        </form>
        ${eventRows || '<div class="empty-card">No observability events found.</div>'}
        ${
          state.events.hasMore
            ? '<button type="button" id="events-load-more" class="btn-ghost h-10 w-full">Load more events</button>'
            : ""
        }
      </section>
    `;

    renderChart();

    ctx.content.querySelector("#incident-create-form")?.addEventListener("submit", async (event) => {
      event.preventDefault();
      const formData = new FormData(event.currentTarget);
      const response = await ctx.requestJson("/api/admin/incidents", "POST", {
        title: String(formData.get("title") || ""),
        severity: String(formData.get("severity") || "sev3"),
        summary: String(formData.get("summary") || ""),
        reason: String(formData.get("reason") || ""),
      });
      if (!response.ok) {
        ctx.setFlash("error", ctx.errorMessage(response));
        return;
      }
      ctx.setFlash("success", "Incident opened.");
      await fetchIncidents({ reset: true });
      render();
    });

    ctx.content.querySelector("#incident-filters")?.addEventListener("change", async () => {
      state.incidents.filters.q = String(
        ctx.content.querySelector("#incident-filter-q")?.value || "",
      ).trim();
      state.incidents.filters.status = toLower(
        String(ctx.content.querySelector("#incident-filter-status")?.value || ""),
      );
      state.incidents.filters.severity = toLower(
        String(ctx.content.querySelector("#incident-filter-severity")?.value || ""),
      );
      try {
        await fetchIncidents({ reset: true });
        render();
      } catch (error) {
        ctx.setFlash("error", String(error));
      }
    });

    ctx.content.querySelector("#obs-events-filters")?.addEventListener("change", async () => {
      state.events.filters.q = String(ctx.content.querySelector("#obs-filter-q")?.value || "").trim();
      state.events.filters.source = toLower(
        String(ctx.content.querySelector("#obs-filter-source")?.value || ""),
      );
      state.events.filters.event_type = toLower(
        String(ctx.content.querySelector("#obs-filter-type")?.value || ""),
      );
      state.events.filters.target_id = "";
      try {
        await fetchEvents({ reset: true });
        render();
      } catch (error) {
        ctx.setFlash("error", String(error));
      }
    });

    ctx.content.querySelector("#incidents-load-more")?.addEventListener("click", async () => {
      try {
        await fetchIncidents({ reset: false });
        render();
      } catch (error) {
        ctx.setFlash("error", String(error));
      }
    });

    ctx.content.querySelector("#events-load-more")?.addEventListener("click", async () => {
      try {
        await fetchEvents({ reset: false });
        render();
      } catch (error) {
        ctx.setFlash("error", String(error));
      }
    });

    ctx.content.querySelectorAll("[data-incident-status]").forEach((button) => {
      button.addEventListener("click", async (event) => {
        const actionButton = event.currentTarget;
        const row = actionButton.closest("[data-incident-id]");
        const incidentId = row?.getAttribute("data-incident-id");
        const nextStatus = actionButton.getAttribute("data-incident-status");
        if (!incidentId || !nextStatus) return;
        const payload = await ctx.openConfirmModal({
          title: "Update incident status",
          message: `Type ${incidentId} and provide a reason to update incident status.`,
          targetLabel: incidentId,
          confirmText: "Apply change",
        });
        if (!payload) return;
        const response = await ctx.requestJson(
          `/api/admin/incidents/${encodeURIComponent(incidentId)}/status`,
          "PATCH",
          {
            status: nextStatus,
            reason: payload.reason,
            confirm_target: payload.confirm_target,
          },
        );
        if (!response.ok) {
          ctx.setFlash("error", ctx.errorMessage(response));
          return;
        }
        ctx.setFlash("success", `Incident ${incidentId} updated.`);
        await fetchIncidents({ reset: true });
        render();
      });
    });
  };

  const openEventsStream = () => {
    if (state.streamDisabled || !window.EventSource) return;
    closeStream();
    try {
      eventsStream = new EventSource("/api/admin/observability/events/stream");
    } catch (_error) {
      state.streamDisabled = true;
      return;
    }

    eventsStream.addEventListener("observability_events", (event) => {
      let payload = null;
      try {
        payload = JSON.parse(event.data || "{}");
      } catch (_error) {
        return;
      }
      if (
        state.events.filters.q ||
        state.events.filters.source ||
        state.events.filters.event_type ||
        state.events.filters.target_id
      ) {
        return;
      }
      const incoming = Array.isArray(payload.items) ? payload.items : [];
      if (!incoming.length) return;
      const seen = new Set();
      const merged = [...incoming, ...state.events.items].filter((item) => {
        const key = `${item.occurred_at}|${item.source}|${item.event_type}|${item.target_id || ""}`;
        if (seen.has(key)) return false;
        seen.add(key);
        return true;
      });
      state.events.items = merged.slice(0, EVENT_LIMIT);
      render();
    });

    eventsStream.addEventListener("error", () => {
      closeStream();
      state.streamDisabled = true;
    });
  };

  await Promise.all([
    fetchSummary(),
    fetchEvents({ reset: true }),
    fetchIncidents({ reset: true }),
  ]);
  render();
  openEventsStream();
};
