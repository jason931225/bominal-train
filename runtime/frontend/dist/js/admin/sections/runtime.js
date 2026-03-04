import { appendQuery, asText, escapeHtml, formatDate, toLower } from "../common/utils.js";
import { itemsFromEnvelope, pageFromEnvelope } from "../common/pagination.js";

const JOB_LIMIT = 25;

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

  const render = () => {
    const jobRows = state.jobs.items
      .map(
        (job) => `
          <article class="admin-row" data-job-id="${escapeHtml(job.job_id)}">
            <div class="min-w-0">
              <p class="truncate text-sm font-semibold txt-strong">${escapeHtml(job.job_id)}</p>
              <p class="mt-1 text-xs txt-supporting">Status: ${escapeHtml(job.status)} · Attempts: ${escapeHtml(job.attempt_count)} · Updated: ${escapeHtml(formatDate(job.updated_at))}</p>
              <p class="mt-1 text-xs txt-faint">Provider: ${escapeHtml(asText(job.provider))} · Operation: ${escapeHtml(asText(job.operation))}</p>
            </div>
            <div class="admin-row-actions">
              <button class="btn-ghost h-10 w-full md:w-auto" data-job-action="retry">Retry</button>
              <button class="btn-ghost h-10 w-full md:w-auto" data-job-action="requeue">Requeue</button>
              <button class="btn-destructive h-10 w-full md:w-auto" data-job-action="cancel">Cancel</button>
            </div>
          </article>
        `,
      )
      .join("");

    const flagRows = state.flags
      .map(
        (flag) => `
          <article class="summary-row" data-flag="${escapeHtml(flag.flag)}">
            <span>${escapeHtml(flag.flag)}</span>
            <button class="btn-ghost h-10 px-3" data-flag-action="${flag.enabled ? "disable" : "enable"}">
              ${flag.enabled ? "Disable" : "Enable"}
            </button>
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
      try {
        await fetchJobs({ reset: true });
        render();
      } catch (error) {
        ctx.setFlash("error", String(error));
      }
    });

    ctx.content.querySelector("#runtime-filter-reset")?.addEventListener("click", async () => {
      state.filters = { q: "", status: "", provider: "", operation: "" };
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
