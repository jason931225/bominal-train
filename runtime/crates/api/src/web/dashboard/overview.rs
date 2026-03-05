use bominal_ui::{DashboardSection, render_dashboard_bottom_nav};

use super::super::{app_shell_topbar, dashboard_desktop_sidebar, html_escape};
pub fn render_dashboard_overview(email: &str) -> String {
    let topbar = app_shell_topbar("Dashboard", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("home");
    let bottom_nav = render_dashboard_bottom_nav(DashboardSection::Home);
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">
    {sidebar}
    <section class="glass-card rounded-[22px] p-5">
      <h2 class="text-lg font-semibold txt-strong">Operational summary</h2>
      <div id="dashboard-summary" class="mt-4 space-y-2">
        <div class="summary-row"><span>Total jobs</span><span>--</span></div>
        <div class="summary-row"><span>Queued</span><span>--</span></div>
        <div class="summary-row"><span>Running</span><span>--</span></div>
        <div class="summary-row"><span>Failed</span><span>--</span></div>
      </div>
      <div class="action-group" data-action-group="single">
        <a
          href="/dashboard/jobs"
          class="btn-primary inline-flex h-12 w-full items-center justify-center"
          data-action-role="primary"
        >
          View Jobs
        </a>
      </div>
    </section>
  </div>
</main>
{bottom_nav}

<script type="module" src="/assets/js/dashboard/overview.js"></script>"#
    )
}
