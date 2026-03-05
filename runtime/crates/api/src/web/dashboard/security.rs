use bominal_ui::{DashboardSection, render_dashboard_bottom_nav};

use super::super::{
    app_shell_topbar, dashboard_desktop_sidebar, dashboard_settings_tabs, html_escape,
};
pub fn render_dashboard_settings(email: &str) -> String {
    let topbar = app_shell_topbar("Settings", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("security");
    let settings_tabs = dashboard_settings_tabs("account");
    let bottom_nav = render_dashboard_bottom_nav(DashboardSection::Settings);
    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">
    {sidebar}
    <section class="space-y-4">
      {settings_tabs}
      <div class="space-y-4">
        <section class="summary-card p-4">
          <div class="flex items-center justify-between gap-2">
            <h3 class="text-sm font-semibold txt-strong">Passkeys</h3>
            <button
              id="passkey-create"
              type="button"
              class="btn-chip inline-flex h-9 w-9 items-center justify-center rounded-xl p-0"
              aria-label="Create passkey"
              title="Create passkey"
            >
              <img class="h-4 w-4 icon-muted" src="/assets/icons/runtime-ui/icon-plus-btn-chip-light.svgz" data-svgz-light="/assets/icons/runtime-ui/icon-plus-btn-chip-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/icon-plus-btn-chip-dark.svgz" alt="" aria-hidden="true" />
            </button>
          </div>
          <div id="passkey-status" class="mt-3 hidden"></div>
          <div id="passkeys-list" class="mt-4 space-y-2"><div class="loading-card">Loading passkeys...</div></div>
        </section>

        <section class="summary-card p-4">
          <h3 class="text-sm font-semibold txt-strong">Change password</h3>
          <p class="mt-1 text-sm txt-supporting">Use upper/lowercase letters, numbers, and symbols.</p>
          <form id="password-change-form" class="mt-4 space-y-3">
            <label class="field-label" for="new-password">New password</label>
            <input id="new-password" type="password" autocomplete="new-password" class="field-input h-12 w-full" />
            <label class="field-label" for="confirm-password">Confirm new password</label>
            <input id="confirm-password" type="password" autocomplete="new-password" class="field-input h-12 w-full" />
            <div class="space-y-1">
              <div class="password-strength-track">
                <div id="password-strength-fill" class="password-strength-fill"></div>
              </div>
              <p id="password-strength-text" class="text-xs txt-supporting">Strength: weak</p>
              <p id="password-match-hint" class="text-xs txt-supporting">Passwords must match.</p>
            </div>
            <div class="action-group" data-action-group="single">
              <button
                type="submit"
                class="btn-primary h-12 w-full"
                data-action-role="primary"
              >
                Update password
              </button>
            </div>
          </form>
          <div id="password-change-status" class="mt-3 hidden"></div>
        </section>
      </div>

      <div id="security-modal" class="app-modal-backdrop hidden" aria-hidden="true">
        <div class="app-modal-card" role="dialog" aria-modal="true" aria-labelledby="security-modal-title">
          <h4 id="security-modal-title" class="text-base font-semibold txt-strong"></h4>
          <p id="security-modal-message" class="mt-2 text-sm txt-supporting"></p>
          <div id="security-modal-input-wrap" class="mt-3 hidden">
            <label id="security-modal-input-label" class="field-label" for="security-modal-input">Current password</label>
            <input id="security-modal-input" type="password" autocomplete="current-password" class="field-input h-12 w-full" />
          </div>
          <div class="action-pair mt-4" data-action-group="pair">
            <button
              id="security-modal-cancel"
              type="button"
              class="btn-ghost h-11 w-full"
              data-action-role="secondary"
            >
              Cancel
            </button>
            <button
              id="security-modal-confirm"
              type="button"
              class="btn-primary h-11 w-full"
              data-action-role="primary"
            >
              Confirm
            </button>
          </div>
        </div>
      </div>

      <div id="passkey-modal" class="app-modal-backdrop hidden" aria-hidden="true">
        <div class="app-modal-card" role="dialog" aria-modal="true" aria-labelledby="passkey-modal-title">
          <div class="flex items-center justify-between gap-2">
            <h4 id="passkey-modal-title" class="text-base font-semibold txt-strong">Edit passkey</h4>
            <button id="passkey-modal-delete" type="button" class="btn-destructive h-9 w-9 p-0" aria-label="Delete passkey">
              <img class="h-4 w-4" src="/assets/icons/runtime-ui/icon-trash-destructive-light.svgz" data-svgz-light="/assets/icons/runtime-ui/icon-trash-destructive-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/icon-trash-destructive-dark.svgz" alt="" aria-hidden="true" />
            </button>
          </div>
          <div class="mt-3 space-y-3">
            <div>
              <label class="field-label" for="passkey-modal-label-input">Label</label>
              <input id="passkey-modal-label-input" type="text" maxlength="80" class="field-input h-11 w-full" />
            </div>
            <div>
              <label class="field-label" for="passkey-modal-credential-id">Credential ID</label>
              <input id="passkey-modal-credential-id" type="text" readonly class="field-input h-11 w-full" />
            </div>
            <div class="summary-row">
              <span>Last used</span>
              <span id="passkey-modal-last-used" class="txt-supporting">Never</span>
            </div>
            <div class="summary-row">
              <span>Created</span>
              <span id="passkey-modal-created-at" class="txt-supporting">-</span>
            </div>
            <div class="summary-row">
              <span>AAGUID</span>
              <span id="passkey-modal-aaguid" class="txt-supporting">Unknown</span>
            </div>
            <div class="summary-row">
              <span>Backup eligible (BE)</span>
              <span id="passkey-modal-be" class="txt-supporting">Unknown</span>
            </div>
            <div class="summary-row">
              <span>Backup state (BS)</span>
              <span id="passkey-modal-bs" class="txt-supporting">Unknown</span>
            </div>
          </div>
          <div class="action-pair mt-4" data-action-group="pair">
            <button
              id="passkey-modal-close"
              type="button"
              class="btn-ghost h-11 w-full"
              data-action-role="secondary"
            >
              Close
            </button>
            <button
              id="passkey-modal-save"
              type="button"
              class="btn-primary h-11 w-full"
              data-action-role="primary"
            >
              Save label
            </button>
          </div>
        </div>
      </div>
    </section>
  </div>
</main>
{bottom_nav}
<script type="module" src="/assets/js/dashboard/security.js"></script>"#
    )
}
