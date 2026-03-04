import { asText, escapeHtml } from "../common/utils.js";
import { itemsFromEnvelope } from "../common/pagination.js";

export const renderSecuritySection = async (ctx) => {
  const [capabilitiesResult, sessionsResult] = await Promise.all([
    ctx.requestJson("/api/admin/capabilities"),
    ctx.requestJson("/api/admin/sessions?limit=25"),
  ]);
  if (!capabilitiesResult.ok) {
    throw new Error(ctx.errorMessage(capabilitiesResult));
  }
  if (!sessionsResult.ok) {
    throw new Error(ctx.errorMessage(sessionsResult));
  }

  const capabilities = capabilitiesResult.body || {};
  const sessions = itemsFromEnvelope(sessionsResult.body);
  const sessionRows = sessions
    .map(
      (session) => `
        <div class="summary-row">
          <span class="truncate">${escapeHtml(asText(session.email))}</span>
          <span class="text-xs txt-supporting">step-up: ${session.step_up_verified_at ? "verified" : "missing"}</span>
        </div>
      `,
    )
    .join("");

  ctx.content.innerHTML = `
    <section class="space-y-2">
      <h2 class="text-lg font-semibold txt-strong">Privileges</h2>
      <div class="summary-row"><span>Role</span><span>${escapeHtml(asText(capabilities.role))}</span></div>
      <div class="summary-row"><span>Can mutate</span><span>${capabilities.can_mutate ? "yes" : "no"}</span></div>
      <div class="summary-row"><span>Step-up required</span><span>${capabilities.step_up_required_for_mutation ? "yes" : "no"}</span></div>
    </section>
    <section class="space-y-2 pt-2">
      <h3 class="text-base font-semibold txt-strong">Session posture</h3>
      ${sessionRows || '<div class="empty-card">No sessions found.</div>'}
    </section>
    <section class="action-group action-single">
      <a class="btn-primary h-11 w-full text-center leading-[2.75rem]" href="/dashboard/settings">Open account settings</a>
    </section>
  `;
};
