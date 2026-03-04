import { asText, escapeHtml } from "../common/utils.js";

export const renderConfigSection = async (ctx) => {
  const result = await ctx.requestJson("/api/admin/config/redacted");
  if (!result.ok) {
    throw new Error(ctx.errorMessage(result));
  }
  const config = result.body?.config || {};
  const rows = Object.keys(config)
    .sort()
    .map(
      (key) => `
        <div class="summary-row">
          <span class="truncate">${escapeHtml(key)}</span>
          <span class="truncate text-xs txt-supporting">${escapeHtml(asText(config[key]))}</span>
        </div>
      `,
    )
    .join("");

  ctx.content.innerHTML = `
    <section class="space-y-2">
      <h2 class="text-lg font-semibold txt-strong">Redacted configuration</h2>
      ${rows || '<div class="empty-card">No config keys available.</div>'}
    </section>
  `;
};
