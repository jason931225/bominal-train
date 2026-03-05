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
<script type="module" src="/assets/js/dashboard/job-detail.js"></script>"#
    )
}
