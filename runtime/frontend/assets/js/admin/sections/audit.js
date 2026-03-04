import { asText, escapeHtml, formatDate } from "../common/utils.js";

export const renderAuditSection = async (ctx) => {
  const result = await ctx.requestJson("/api/admin/audit");
  if (!result.ok) {
    throw new Error(ctx.errorMessage(result));
  }
  const entries = Array.isArray(result.body?.entries) ? result.body.entries : [];
  const rows = entries
    .slice(0, 120)
    .map(
      (entry) => `
        <article class="summary-card">
          <p class="text-sm font-semibold txt-strong">${escapeHtml(entry.action)} · ${escapeHtml(entry.target_type)}</p>
          <p class="mt-1 text-xs txt-supporting">${escapeHtml(entry.actor_email)} · ${escapeHtml(formatDate(entry.created_at))}</p>
          <p class="mt-1 text-xs txt-supporting">target: ${escapeHtml(asText(entry.target_id))}</p>
          <p class="mt-1 text-xs txt-faint">request_id: ${escapeHtml(asText(entry.request_id))}</p>
        </article>
      `,
    )
    .join("");

  ctx.content.innerHTML = `
    <section class="space-y-2">
      <h2 class="text-lg font-semibold txt-strong">Immutable admin audit</h2>
      ${rows || '<div class="empty-card">No audit records found.</div>'}
    </section>
  `;
};
