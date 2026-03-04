import { appendQuery, asText, escapeHtml, formatDate, toLower } from "../common/utils.js";
import { itemsFromEnvelope, pageFromEnvelope } from "../common/pagination.js";

const USER_LIMIT = 25;
const SESSION_LIMIT = 25;

export const renderUsersSection = async (ctx) => {
  const state = {
    filters: {
      q: "",
      role: "",
      status: "",
      access: "all",
      revoked: "all",
      step_up: "all",
    },
    users: {
      items: [],
      nextCursor: null,
      hasMore: false,
      loading: false,
    },
    sessions: {
      items: [],
      nextCursor: null,
      hasMore: false,
      loading: false,
    },
  };

  const fetchUsers = async ({ reset }) => {
    if (state.users.loading) return;
    state.users.loading = true;
    const result = await ctx.requestJson(
      appendQuery("/api/admin/users", {
        limit: USER_LIMIT,
        cursor: reset ? null : state.users.nextCursor,
        q: state.filters.q || null,
        role: state.filters.role || null,
        status: state.filters.status || null,
        access: state.filters.access,
      }),
    );
    state.users.loading = false;
    if (!result.ok) {
      throw new Error(ctx.errorMessage(result));
    }
    const items = itemsFromEnvelope(result.body);
    const page = pageFromEnvelope(result.body);
    state.users.items = reset ? items : state.users.items.concat(items);
    state.users.hasMore = page.hasMore;
    state.users.nextCursor = page.nextCursor;
  };

  const fetchSessions = async ({ reset }) => {
    if (state.sessions.loading) return;
    state.sessions.loading = true;
    const result = await ctx.requestJson(
      appendQuery("/api/admin/sessions", {
        limit: SESSION_LIMIT,
        cursor: reset ? null : state.sessions.nextCursor,
        q: state.filters.q || null,
        role: state.filters.role || null,
        revoked: state.filters.revoked,
        step_up: state.filters.step_up,
      }),
    );
    state.sessions.loading = false;
    if (!result.ok) {
      throw new Error(ctx.errorMessage(result));
    }
    const items = itemsFromEnvelope(result.body);
    const page = pageFromEnvelope(result.body);
    state.sessions.items = reset ? items : state.sessions.items.concat(items);
    state.sessions.hasMore = page.hasMore;
    state.sessions.nextCursor = page.nextCursor;
  };

  const render = () => {
    const userRows = state.users.items
      .map((user) => {
        const nextAccess = user.access_enabled ? "false" : "true";
        const accessLabel = user.access_enabled ? "Disable access" : "Enable access";
        return `
          <article class="admin-row" data-user-id="${escapeHtml(user.user_id)}">
            <div class="min-w-0">
              <p class="truncate text-sm font-semibold txt-strong">${escapeHtml(user.email)}</p>
              <p class="truncate text-xs txt-faint">${escapeHtml(user.user_id)}</p>
              <p class="mt-1 text-xs txt-supporting">Status: ${escapeHtml(asText(user.status))} · Role: ${escapeHtml(asText(user.role))}</p>
            </div>
            <div class="admin-row-actions">
              <select class="field-input h-10 w-full md:w-[130px]" data-role-select>
                ${["user", "viewer", "operator", "admin"]
                  .map(
                    (role) =>
                      `<option value="${role}" ${role === user.role ? "selected" : ""}>${role}</option>`,
                  )
                  .join("")}
              </select>
              <button class="btn-ghost h-10 w-full md:w-auto" data-user-action="role">Update role</button>
              <button class="btn-ghost h-10 w-full md:w-auto" data-user-action="access" data-next-access="${nextAccess}">${accessLabel}</button>
              <button class="btn-destructive h-10 w-full md:w-auto" data-user-action="revoke">Revoke sessions</button>
            </div>
          </article>
        `;
      })
      .join("");

    const sessionRows = state.sessions.items
      .map(
        (session) => `
          <div class="summary-row">
            <span class="truncate">${escapeHtml(asText(session.email))}</span>
            <span class="text-xs txt-supporting">${escapeHtml(asText(session.role))} · ${escapeHtml(formatDate(session.last_seen_at))}</span>
          </div>
        `,
      )
      .join("");

    ctx.content.innerHTML = `
      <section class="space-y-2">
        <h2 class="text-lg font-semibold txt-strong">User and session management</h2>
        <form id="admin-users-filters" class="grid grid-cols-1 gap-2 md:grid-cols-6">
          <input id="users-filter-q" class="field-input h-10 md:col-span-2" placeholder="Search email or user id" value="${escapeHtml(state.filters.q)}" />
          <select id="users-filter-role" class="field-input h-10">
            <option value="">all roles</option>
            ${["admin", "operator", "viewer", "user"]
              .map(
                (role) =>
                  `<option value="${role}" ${state.filters.role === role ? "selected" : ""}>${role}</option>`,
              )
              .join("")}
          </select>
          <input id="users-filter-status" class="field-input h-10" placeholder="status" value="${escapeHtml(state.filters.status)}" />
          <select id="users-filter-access" class="field-input h-10">
            ${["all", "enabled", "disabled"]
              .map(
                (access) =>
                  `<option value="${access}" ${state.filters.access === access ? "selected" : ""}>${access}</option>`,
              )
              .join("")}
          </select>
          <div class="grid grid-cols-2 gap-2">
            <button type="submit" class="btn-primary h-10 w-full">Apply</button>
            <button type="button" id="users-filter-reset" class="btn-ghost h-10 w-full">Reset</button>
          </div>
        </form>
        ${userRows || '<div class="empty-card">No users found.</div>'}
        ${
          state.users.hasMore
            ? '<button type="button" id="users-load-more" class="btn-ghost h-10 w-full">Load more users</button>'
            : ""
        }
      </section>
      <section class="space-y-2 pt-3">
        <h3 class="text-base font-semibold txt-strong">Recent sessions</h3>
        <div class="grid grid-cols-1 gap-2 md:grid-cols-3">
          <select id="sessions-filter-revoked" class="field-input h-10">
            ${["all", "active", "revoked"]
              .map(
                (revoked) =>
                  `<option value="${revoked}" ${state.filters.revoked === revoked ? "selected" : ""}>${revoked}</option>`,
              )
              .join("")}
          </select>
          <select id="sessions-filter-step-up" class="field-input h-10">
            ${["all", "verified", "missing"]
              .map(
                (stepUp) =>
                  `<option value="${stepUp}" ${state.filters.step_up === stepUp ? "selected" : ""}>${stepUp}</option>`,
              )
              .join("")}
          </select>
          <button type="button" id="sessions-apply" class="btn-ghost h-10 w-full">Apply session filters</button>
        </div>
        ${sessionRows || '<div class="empty-card">No sessions found.</div>'}
        ${
          state.sessions.hasMore
            ? '<button type="button" id="sessions-load-more" class="btn-ghost h-10 w-full">Load more sessions</button>'
            : ""
        }
      </section>
    `;

    const usersFilterForm = ctx.content.querySelector("#admin-users-filters");
    const usersResetBtn = ctx.content.querySelector("#users-filter-reset");
    const usersLoadMoreBtn = ctx.content.querySelector("#users-load-more");
    const sessionsApplyBtn = ctx.content.querySelector("#sessions-apply");
    const sessionsLoadMoreBtn = ctx.content.querySelector("#sessions-load-more");

    usersFilterForm?.addEventListener("submit", async (event) => {
      event.preventDefault();
      state.filters.q = String(ctx.content.querySelector("#users-filter-q")?.value || "").trim();
      state.filters.role = toLower(ctx.content.querySelector("#users-filter-role")?.value || "");
      state.filters.status = String(ctx.content.querySelector("#users-filter-status")?.value || "").trim();
      state.filters.access = toLower(ctx.content.querySelector("#users-filter-access")?.value || "all");
      try {
        await Promise.all([fetchUsers({ reset: true }), fetchSessions({ reset: true })]);
        render();
      } catch (error) {
        ctx.setFlash("error", String(error));
      }
    });

    usersResetBtn?.addEventListener("click", async () => {
      state.filters = {
        q: "",
        role: "",
        status: "",
        access: "all",
        revoked: "all",
        step_up: "all",
      };
      try {
        await Promise.all([fetchUsers({ reset: true }), fetchSessions({ reset: true })]);
        render();
      } catch (error) {
        ctx.setFlash("error", String(error));
      }
    });

    usersLoadMoreBtn?.addEventListener("click", async () => {
      try {
        await fetchUsers({ reset: false });
        render();
      } catch (error) {
        ctx.setFlash("error", String(error));
      }
    });

    sessionsApplyBtn?.addEventListener("click", async () => {
      state.filters.revoked = toLower(
        ctx.content.querySelector("#sessions-filter-revoked")?.value || "all",
      );
      state.filters.step_up = toLower(
        ctx.content.querySelector("#sessions-filter-step-up")?.value || "all",
      );
      try {
        await fetchSessions({ reset: true });
        render();
      } catch (error) {
        ctx.setFlash("error", String(error));
      }
    });

    sessionsLoadMoreBtn?.addEventListener("click", async () => {
      try {
        await fetchSessions({ reset: false });
        render();
      } catch (error) {
        ctx.setFlash("error", String(error));
      }
    });

    ctx.content.querySelectorAll("[data-user-action]").forEach((button) => {
      button.addEventListener("click", async (event) => {
        const actionButton = event.currentTarget;
        const row = actionButton.closest("[data-user-id]");
        const userId = row?.getAttribute("data-user-id");
        if (!userId) return;

        if (actionButton.getAttribute("data-user-action") === "role") {
          const role = String(row.querySelector("[data-role-select]")?.value || "user");
          const payload = await ctx.openConfirmModal({
            title: "Update role",
            message: `Type ${userId} and provide a reason to update role.`,
            targetLabel: userId,
            confirmText: "Apply change",
          });
          if (!payload) return;
          const response = await ctx.requestJson(
            `/api/admin/users/${encodeURIComponent(userId)}/role`,
            "PATCH",
            { role, reason: payload.reason, confirm_target: payload.confirm_target },
          );
          if (!response.ok) {
            ctx.setFlash("error", ctx.errorMessage(response));
            return;
          }
          ctx.setFlash("success", "User role updated.");
          await Promise.all([fetchUsers({ reset: true }), fetchSessions({ reset: true })]);
          render();
          return;
        }

        if (actionButton.getAttribute("data-user-action") === "access") {
          const accessEnabled = actionButton.getAttribute("data-next-access") === "true";
          const payload = await ctx.openConfirmModal({
            title: "Update user access",
            message: `Type ${userId} and provide a reason to update access.`,
            targetLabel: userId,
            confirmText: "Apply change",
          });
          if (!payload) return;
          const response = await ctx.requestJson(
            `/api/admin/users/${encodeURIComponent(userId)}/access`,
            "PATCH",
            {
              access_enabled: accessEnabled,
              reason: payload.reason,
              confirm_target: payload.confirm_target,
            },
          );
          if (!response.ok) {
            ctx.setFlash("error", ctx.errorMessage(response));
            return;
          }
          ctx.setFlash("success", "User access updated.");
          await Promise.all([fetchUsers({ reset: true }), fetchSessions({ reset: true })]);
          render();
          return;
        }

        const payload = await ctx.openConfirmModal({
          title: "Revoke sessions",
          message: `Type ${userId} and provide a reason to revoke sessions.`,
          targetLabel: userId,
          confirmText: "Revoke",
        });
        if (!payload) return;
        const response = await ctx.requestJson(
          `/api/admin/users/${encodeURIComponent(userId)}/sessions/revoke`,
          "POST",
          payload,
        );
        if (!response.ok) {
          ctx.setFlash("error", ctx.errorMessage(response));
          return;
        }
        const revokedCount = asText(response.body?.revoked, "0");
        ctx.setFlash("success", `Revoked ${revokedCount} sessions.`);
        await Promise.all([fetchUsers({ reset: true }), fetchSessions({ reset: true })]);
        render();
      });
    });
  };

  await Promise.all([fetchUsers({ reset: true }), fetchSessions({ reset: true })]);
  render();
};
