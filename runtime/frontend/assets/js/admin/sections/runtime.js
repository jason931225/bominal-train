import { appendQuery, asText, escapeHtml, formatDate, toLower } from "../common/utils.js";
import { itemsFromEnvelope, pageFromEnvelope } from "../common/pagination.js";

const JOB_LIMIT = 25;
const JOB_EVENTS_LIMIT = 20;

const jobStatusBadgeClass = (status) => {
  const normalized = toLower(status);
  if (["failed", "cancelled", "dead_letter", "fatal"].includes(normalized)) {
    return "badge-critical";
  }
  if (["queued", "retry_scheduled", "pending"].includes(normalized)) {
    return "badge-warning";
  }
  if (["succeeded", "completed", "done"].includes(normalized)) {
    return "badge-success";
  }
  return "badge-muted";
};

const safeJsonSnippet = (value, maxLength = 420) => {
  try {
    const text = JSON.stringify(value ?? {}, null, 2);
    return text.length > maxLength ? `${text.slice(0, maxLength)}…` : text;
  } catch (_error) {
    return "{}";
  }
};

export const renderRuntimeSection = async (ctx) => {
  const state = {
    filters: {
      q: "",
      status: "",
      provider: "",
      operation: "",
    },
    jobs: {
      items: [],
      nextCursor: null,
      hasMore: false,
      loading: false,
      openEventJobId: null,
      loadingEventsFor: null,
      eventsByJobId: new Map(),
    },
    flags: [],
    streamDisabled: false,
  };

  let runtimeJobsStream = null;
  const closeStream = () => {
    if (runtimeJobsStream) {
      runtimeJobsStream.close();
      runtimeJobsStream = null;
    }
  };
  ctx.registerCleanup(closeStream);

  const fetchJobs = async ({ reset }) => {
    if (state.jobs.loading) return;
    state.jobs.loading = true;
    const result = await ctx.requestJson(
      appendQuery("/api/admin/runtime/jobs", {
        limit: JOB_LIMIT,
        cursor: reset ? null : state.jobs.nextCursor,
        q: state.filters.q || null,
        status: state.filters.status || null,
        provider: state.filters.provider || null,
        operation: state.filters.operation || null,
      }),
    );
    state.jobs.loading = false;
    if (!result.ok) {
      throw new Error(ctx.errorMessage(result));
    }
    const items = itemsFromEnvelope(result.body);
    const page = pageFromEnvelope(result.body);
    state.jobs.items = reset ? items : state.jobs.items.concat(items);
    state.jobs.hasMore = page.hasMore;
    state.jobs.nextCursor = page.nextCursor;
  };

  const fetchFlags = async () => {
    const result = await ctx.requestJson("/api/admin/runtime/kill-switches");
    if (!result.ok) throw new Error(ctx.errorMessage(result));
    state.flags = Array.isArray(result.body?.flags) ? result.body.flags : [];
  };

  const fetchJobEvents = async (jobId) => {
    state.jobs.loadingEventsFor = jobId;
    const response = await ctx.requestJson(
      appendQuery(`/api/admin/runtime/jobs/${encodeURIComponent(jobId)}/events`, {
        limit: JOB_EVENTS_LIMIT,
      }),
    );
    state.jobs.loadingEventsFor = null;
    if (!response.ok) {
      throw new Error(ctx.errorMessage(response));
    }
    const items = Array.isArray(response.body?.items) ? response.body.items : [];
    state.jobs.eventsByJobId.set(jobId, items);
  };

  const render = () => {
    if (
      state.jobs.openEventJobId &&
      !state.jobs.items.some((job) => job.job_id === state.jobs.openEventJobId)
    ) {
      state.jobs.openEventJobId = null;
    }

    const jobRows = state.jobs.items
      .map((job) => {
        const isEventsOpen = state.jobs.openEventJobId === job.job_id;
        const isEventsLoading = state.jobs.loadingEventsFor === job.job_id;
        const jobEvents = state.jobs.eventsByJobId.get(job.job_id) || [];
        return `
          <article class="admin-row" data-job-id="${escapeHtml(job.job_id)}">
            <div class="min-w-0">
              <p class="truncate text-sm font-semibold txt-strong">${escapeHtml(job.job_id)}</p>
              <div class="admin-row-meta">
                <span class="badge ${jobStatusBadgeClass(job.status)}">${escapeHtml(asText(job.status))}</span>
                <span class="admin-pill">Attempts: ${escapeHtml(asText(job.attempt_count, "0"))}</span>
                <span class="admin-pill">Provider: ${escapeHtml(asText(job.provider))}</span>
                <span class="admin-pill">Operation: ${escapeHtml(asText(job.operation))}</span>
                <span class="admin-pill">Next run: ${escapeHtml(formatDate(job.next_run_at))}</span>
                <span class="admin-pill">Updated: ${escapeHtml(formatDate(job.updated_at))}</span>
              </div>
            </div>
            <div class="admin-row-actions">
              <button class="btn-ghost h-10 w-full md:w-auto" data-job-action="events">${isEventsOpen ? "Hide events" : "View events"}</button>
              <button class="btn-ghost h-10 w-full md:w-auto" data-job-action="retry">Retry</button>
              <button class="btn-ghost h-10 w-full md:w-auto" data-job-action="requeue">Requeue</button>
              <button class="btn-destructive h-10 w-full md:w-auto" data-job-action="cancel">Cancel</button>
            </div>
            ${
              isEventsOpen
                ? `<div class="space-y-2 rounded-2xl border border-slate-200/80 bg-white/60 p-3">
                     <p class="text-xs font-semibold uppercase tracking-[0.08em] txt-supporting">Recent events</p>
                     ${
                       isEventsLoading
                         ? '<div class="loading-card">Loading runtime events…</div>'
                         : jobEvents.length
                           ? jobEvents
                               .map(
                                 (event) => `
                                   <article class="summary-row">
                                     <span class="truncate text-xs font-semibold txt-strong">${escapeHtml(asText(event.event_type))}</span>
                                     <span class="text-xs txt-supporting">${escapeHtml(formatDate(event.created_at))}</span>
                                   </article>
                                   <details class="rounded-2xl border border-slate-200/70 bg-white/70 px-3 py-2 text-xs txt-faint">
                                     <summary class="cursor-pointer txt-supporting">Payload preview</summary>
                                     <pre class="mt-2 max-h-36 overflow-auto whitespace-pre-wrap">${escapeHtml(safeJsonSnippet(event.event_payload))}</pre>
                                   </details>
                                 `,
                               )
                               .join("")
                           : '<div class="empty-card">No runtime events found for this job.</div>'
                     }
                   </div>`
                : ""
            }
          </article>
        `;
      })
      .join("");

    const flagRows = state.flags
      .map(
        (flag) => `
          <article class="admin-row" data-flag="${escapeHtml(flag.flag)}">
            <div class="min-w-0">
              <p class="truncate text-sm font-semibold txt-strong">${escapeHtml(flag.flag)}</p>
              <div class="admin-row-meta">
                <span class="badge ${flag.enabled ? "badge-critical" : "badge-success"}">${flag.enabled ? "Enabled" : "Disabled"}</span>
                <span class="admin-pill">Reason: ${escapeHtml(asText(flag.reason))}</span>
                <span class="admin-pill">Updated: ${escapeHtml(formatDate(flag.updated_at))}</span>
              </div>
            </div>
            <div class="admin-row-actions">
              <button class="btn-ghost h-10 px-3" data-flag-action="${flag.enabled ? "disable" : "enable"}">
                ${flag.enabled ? "Disable" : "Enable"}
              </button>
            </div>
          </article>
        `,
      )
      .join("");

    ctx.content.innerHTML = `
      <section class="space-y-2">
        <h2 class="text-lg font-semibold txt-strong">Runtime operations</h2>
        <form id="runtime-filters" class="grid grid-cols-1 gap-2 md:grid-cols-5">
          <input id="runtime-filter-q" class="field-input h-10 md:col-span-2" placeholder="Search job id/provider/operation" value="${escapeHtml(state.filters.q)}" />
          <input id="runtime-filter-status" class="field-input h-10" placeholder="status" value="${escapeHtml(state.filters.status)}" />
          <input id="runtime-filter-provider" class="field-input h-10" placeholder="provider" value="${escapeHtml(state.filters.provider)}" />
          <input id="runtime-filter-operation" class="field-input h-10" placeholder="operation" value="${escapeHtml(state.filters.operation)}" />
          <div class="grid grid-cols-2 gap-2 md:col-span-5">
            <button type="submit" class="btn-primary h-10 w-full">Apply</button>
            <button type="button" id="runtime-filter-reset" class="btn-ghost h-10 w-full">Reset</button>
          </div>
        </form>
        ${jobRows || '<div class="empty-card">No runtime jobs available.</div>'}
        ${
          state.jobs.hasMore
            ? '<button type="button" id="runtime-load-more" class="btn-ghost h-10 w-full">Load more jobs</button>'
            : ""
        }
      </section>
      <section class="space-y-2 pt-2">
        <h3 class="text-base font-semibold txt-strong">Kill switches</h3>
        ${flagRows || '<div class="empty-card">No kill switches found.</div>'}
      </section>
    `;

    ctx.content.querySelector("#runtime-filters")?.addEventListener("submit", async (event) => {
      event.preventDefault();
      state.filters.q = String(ctx.content.querySelector("#runtime-filter-q")?.value || "").trim();
      state.filters.status = toLower(
        String(ctx.content.querySelector("#runtime-filter-status")?.value || ""),
      );
      state.filters.provider = toLower(
        String(ctx.content.querySelector("#runtime-filter-provider")?.value || ""),
      );
      state.filters.operation = toLower(
        String(ctx.content.querySelector("#runtime-filter-operation")?.value || ""),
      );
      state.jobs.openEventJobId = null;
      try {
        await fetchJobs({ reset: true });
        render();
      } catch (error) {
        ctx.setFlash("error", String(error));
      }
    });

    ctx.content.querySelector("#runtime-filter-reset")?.addEventListener("click", async () => {
      state.filters = { q: "", status: "", provider: "", operation: "" };
      state.jobs.openEventJobId = null;
      try {
        await fetchJobs({ reset: true });
        render();
      } catch (error) {
        ctx.setFlash("error", String(error));
      }
    });

    ctx.content.querySelector("#runtime-load-more")?.addEventListener("click", async () => {
      try {
        await fetchJobs({ reset: false });
        render();
      } catch (error) {
        ctx.setFlash("error", String(error));
      }
    });

    ctx.content.querySelectorAll("[data-job-action]").forEach((button) => {
      button.addEventListener("click", async (event) => {
        const actionButton = event.currentTarget;
        const row = actionButton.closest("[data-job-id]");
        const jobId = row?.getAttribute("data-job-id");
        const action = actionButton.getAttribute("data-job-action");
        if (!jobId || !action) return;

        if (action === "events") {
          if (state.jobs.openEventJobId === jobId) {
            state.jobs.openEventJobId = null;
            render();
            return;
          }
          state.jobs.openEventJobId = jobId;
          if (!state.jobs.eventsByJobId.has(jobId)) {
            try {
              render();
              await fetchJobEvents(jobId);
            } catch (error) {
              ctx.setFlash("error", String(error));
            }
          }
          render();
          return;
        }

        const payload = await ctx.openConfirmModal({
          title: `Runtime ${action}`,
          message: `Type ${jobId} and provide a reason for this runtime action.`,
          targetLabel: jobId,
          confirmText: "Apply change",
        });
        if (!payload) return;
        const response = await ctx.requestJson(
          `/api/admin/runtime/jobs/${encodeURIComponent(jobId)}/${action}`,
          "POST",
          payload,
        );
        if (!response.ok) {
          ctx.setFlash("error", ctx.errorMessage(response));
          return;
        }
        ctx.setFlash("success", `Job ${jobId} updated.`);
        state.jobs.eventsByJobId.delete(jobId);
        await fetchJobs({ reset: true });
        render();
      });
    });

    ctx.content.querySelectorAll("[data-flag-action]").forEach((button) => {
      button.addEventListener("click", async (event) => {
        const actionButton = event.currentTarget;
        const row = actionButton.closest("[data-flag]");
        const flag = row?.getAttribute("data-flag");
        if (!flag) return;
        const enabled = actionButton.getAttribute("data-flag-action") === "enable";
        const payload = await ctx.openConfirmModal({
          title: "Update kill switch",
          message: `Type ${flag} and provide a reason to update the kill switch.`,
          targetLabel: flag,
          confirmText: "Apply change",
        });
        if (!payload) return;
        const response = await ctx.requestJson(
          `/api/admin/runtime/kill-switches/${encodeURIComponent(flag)}`,
          "PUT",
          { enabled, reason: payload.reason, confirm_target: payload.confirm_target },
        );
        if (!response.ok) {
          ctx.setFlash("error", ctx.errorMessage(response));
          return;
        }
        ctx.setFlash("success", `Kill switch ${flag} updated.`);
        await fetchFlags();
        render();
      });
    });
  };

  const openJobsStream = () => {
    if (state.streamDisabled || !window.EventSource) return;
    closeStream();
    try {
      runtimeJobsStream = new EventSource("/api/admin/runtime/jobs/stream");
    } catch (_error) {
      state.streamDisabled = true;
      return;
    }
    runtimeJobsStream.addEventListener("runtime_jobs", (event) => {
      let payload = null;
      try {
        payload = JSON.parse(event.data || "{}");
      } catch (_error) {
        return;
      }
      if (
        state.filters.q ||
        state.filters.status ||
        state.filters.provider ||
        state.filters.operation
      ) {
        return;
      }
      const streamItems = Array.isArray(payload.items)
        ? payload.items
        : Array.isArray(payload.jobs)
          ? payload.jobs
          : [];
      state.jobs.items = streamItems;
      state.jobs.nextCursor = null;
      state.jobs.hasMore = false;
      render();
    });
    runtimeJobsStream.addEventListener("error", () => {
      closeStream();
      state.streamDisabled = true;
    });
  };

  await Promise.all([fetchJobs({ reset: true }), fetchFlags()]);
  render();
  openJobsStream();
};
