use bominal_ui::{
    ButtonProps, ButtonRole, ButtonSize, ButtonType, ButtonVariant, DashboardSection,
    render_button, render_dashboard_bottom_nav,
};

use super::super::{app_shell_topbar, dashboard_desktop_sidebar, html_escape};

const TRAIN_SCRIPT_TAG: &str =
    r#"<script type="module" src="/assets/js/dashboard/train.js"></script>"#;

pub fn render_dashboard_train(email: &str) -> String {
    let topbar = app_shell_topbar("Train", &format!("Signed in as {}", html_escape(email)));
    let sidebar = dashboard_desktop_sidebar("train");
    let bottom_nav = render_dashboard_bottom_nav(DashboardSection::Train);

    let mut html = String::new();
    html.push_str(&topbar);
    html.push_str(
        r#"<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <div class="md:grid md:grid-cols-[220px_minmax(0,1fr)] md:items-start md:gap-4">"#,
    );
    html.push_str(&sidebar);
    html.push_str(r#"<section class="train-section-stack">"#);
    html.push_str(&render_train_workspace_section());
    html.push_str(&render_train_search_section());
    html.push_str(&render_train_results_and_task_section());
    html.push_str(&render_train_history_section());
    html.push_str(&render_train_picker_modals());
    html.push_str("</section></div></main>");
    html.push_str(&bottom_nav);
    html.push_str(TRAIN_SCRIPT_TAG);
    html
}

fn render_train_workspace_section() -> String {
    r#"<section class="train-panel">
  <h2 class="text-lg font-semibold txt-strong" data-i18n="workspace.title">Train workspace</h2>
  <p class="mt-1 text-sm txt-supporting" data-i18n="workspace.subtitle">Korail-inspired hybrid flow with modal selectors and station catalog safety checks.</p>
  <div id="train-preflight" class="mt-4 space-y-2">
    <div class="loading-card" data-i18n="preflight.loading">Loading provider readiness...</div>
  </div>
</section>"#
        .to_string()
}

fn render_train_search_section() -> String {
    let search_submit = render_button(&ButtonProps {
        id: Some("train-search-submit"),
        label: "Start search",
        variant: ButtonVariant::Primary,
        size: ButtonSize::Md,
        button_type: ButtonType::Submit,
        disabled: false,
        aria_label: None,
        action_role: Some(ButtonRole::Primary),
        extra_class: Some(""),
    })
    .replacen("<button ", "<button data-i18n=\"search.start\" ", 1);

    format!(
        r#"<section class="train-panel">
  <h3 class="text-base font-semibold txt-strong" data-i18n="search.title">Search trains</h3>
  <form id="train-search-form" class="mt-3 space-y-3">
    <div class="train-control-shell">
      <div class="grid grid-cols-[minmax(0,1fr)_44px_minmax(0,1fr)] gap-2">
        <button id="dep-station-open" type="button" class="summary-card min-w-0 h-14 text-left">
          <span class="text-[11px] uppercase tracking-[0.08em] txt-supporting" data-i18n="search.departure">Departure</span>
          <div id="dep-station-display" class="mt-1 truncate text-sm font-semibold txt-strong" data-i18n="search.select_station">Select station</div>
        </button>
        <button id="station-swap" type="button" class="btn-ghost h-11 w-11 shrink-0 rounded-full p-0" aria-label="Swap stations" data-i18n-aria-label="search.swap_stations">
          <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" data-station-swap-icon>
            <path d="M7 7h10"></path>
            <path d="m13 3 4 4-4 4"></path>
            <path d="M17 17H7"></path>
            <path d="m11 13-4 4 4 4"></path>
          </svg>
        </button>
        <button id="arr-station-open" type="button" class="summary-card min-w-0 h-14 text-left">
          <span class="text-[11px] uppercase tracking-[0.08em] txt-supporting" data-i18n="search.arrival">Arrival</span>
          <div id="arr-station-display" class="mt-1 truncate text-sm font-semibold txt-strong" data-i18n="search.select_station">Select station</div>
        </button>
      </div>
      <div class="mt-2 grid grid-cols-1 gap-2 md:grid-cols-[minmax(0,1fr)_minmax(0,1.35fr)] md:items-start">
        <button id="dep-date-open" type="button" class="summary-card h-14 text-left">
          <span class="text-[11px] uppercase tracking-[0.08em] txt-supporting" data-i18n="search.departure_date">Departure date</span>
          <div id="dep-date-display" class="mt-1 text-sm font-semibold txt-strong" data-i18n="search.select_date">Select date</div>
        </button>
        <div class="summary-card">
          <span class="text-[11px] uppercase tracking-[0.08em] txt-supporting" data-i18n="search.passengers">Passengers</span>
          <div class="mt-2 grid grid-cols-[44px_minmax(0,1fr)_44px] gap-2">
            <button id="passenger-minus" type="button" class="btn-ghost h-10 w-10 rounded-full p-0 text-lg" aria-label="Decrease passengers">−</button>
            <button id="passenger-open" type="button" class="summary-row h-11 min-w-0 justify-center px-3 text-center" aria-label="Passenger details">
              <span id="passenger-display" class="truncate text-sm font-semibold txt-strong">1</span>
            </button>
            <button id="passenger-plus" type="button" class="btn-ghost h-10 w-10 rounded-full p-0 text-lg" aria-label="Increase passengers">＋</button>
          </div>
        </div>
      </div>
    </div>
    <div class="action-group" data-action-group="single">
      {search_submit}
    </div>
  </form>
  <div id="train-search-status" class="mt-3 hidden"></div>
</section>"#
    )
}

fn render_train_results_and_task_section() -> String {
    let auto_pay_toggle =
        render_task_option_toggle("task-auto-pay-toggle", "Auto pay", "card", false);
    let notify_toggle =
        render_task_option_toggle("task-notify-email-toggle", "Notify by email", "bell", true);
    let retry_toggle = render_task_option_toggle(
        "task-retry-expiry-toggle",
        "Retry on expiry (max 3)",
        "hourglass",
        false,
    );
    let create_task = render_button(&ButtonProps {
        id: Some("train-task-create"),
        label: "Create Task",
        variant: ButtonVariant::Primary,
        size: ButtonSize::Md,
        button_type: ButtonType::Button,
        disabled: false,
        aria_label: None,
        action_role: Some(ButtonRole::Primary),
        extra_class: Some(""),
    });

    format!(
        r#"<section class="train-panel">
  <div class="summary-row">
    <h3 class="text-base font-semibold txt-strong" data-i18n="search.latest_result">Latest search result</h3>
    <span id="active-search-id" class="text-xs txt-supporting" data-i18n="search.none">none</span>
  </div>
  <div id="train-results" class="mt-3 space-y-2"></div>
  <div id="train-task-controls" class="mt-4 space-y-3">
    <div class="grid grid-cols-3 gap-2">
      {auto_pay_toggle}
      {notify_toggle}
      {retry_toggle}
    </div>
    {create_task}
    <div id="train-task-status" class="hidden"></div>
    <div id="train-task-live" class="summary-card hidden"></div>
  </div>
</section>"#
    )
}

fn render_task_option_toggle(id: &str, label: &str, icon: &str, selected: bool) -> String {
    let state_class = if selected {
        " provider-select-card-selected txt-accent"
    } else {
        ""
    };
    let icon_svg = match icon {
        "card" => {
            r#"<svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  <rect x="3" y="6" width="18" height="12" rx="2.5"></rect>
  <path d="M3 10.5h18"></path>
  <path d="M7 15h3"></path>
</svg>"#
        }
        "bell" => {
            r#"<svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  <path d="M15 17H5.5a1 1 0 0 1-.8-1.6L6 13.7V10a6 6 0 1 1 12 0v3.7l1.3 1.7a1 1 0 0 1-.8 1.6H17"></path>
  <path d="M10 20a2 2 0 0 0 4 0"></path>
</svg>"#
        }
        _ => {
            r#"<svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  <path d="M7 3h10"></path>
  <path d="M7 21h10"></path>
  <path d="M8 4h8v4l-2.7 3L16 14v6H8v-6l2.7-3L8 8V4Z"></path>
</svg>"#
        }
    };

    format!(
        r#"<button id="{id}" type="button" class="summary-card provider-select-card flex h-14 items-center justify-center p-0{state_class}" aria-label="{label}" aria-pressed="{selected}" title="{label}">
  <span class="sr-only">{label}</span>
  {icon_svg}
</button>"#,
        id = html_escape(id),
        label = html_escape(label),
        selected = if selected { "true" } else { "false" },
        state_class = state_class,
        icon_svg = icon_svg,
    )
}

fn render_train_history_section() -> String {
    r#"<section class="train-panel">
  <h3 class="text-base font-semibold txt-strong" data-i18n="search.recent">Recent tasks</h3>
  <div id="train-search-history" class="mt-3 space-y-2"><div class="loading-card">Loading tasks...</div></div>
</section>"#
        .to_string()
}

fn render_train_picker_modals() -> String {
    r#"<div id="station-picker-modal" class="app-modal-backdrop hidden" aria-hidden="true">
  <div class="app-modal-card max-w-[820px]" role="dialog" aria-modal="true" aria-labelledby="station-picker-title">
    <div class="flex items-center justify-between">
      <h4 id="station-picker-title" class="text-base font-semibold txt-strong" data-i18n="station.modal_title">Station picker</h4>
      <button id="station-picker-close" type="button" class="btn-ghost h-9 w-9 p-0" aria-label="Close" data-i18n-aria-label="common.close">✕</button>
    </div>
    <div class="mt-3">
      <label class="sr-only" for="station-picker-query" data-i18n="station.search_label">Search station</label>
      <div class="relative">
        <input id="station-picker-query" type="text" autocomplete="off" class="field-input h-11 w-full pr-11 leading-none" placeholder="Search station name or initials (Seoul, ㅅㅇ)" data-i18n-placeholder="station.search_placeholder" />
        <span class="pointer-events-none absolute inset-y-0 right-3 inline-flex items-center txt-supporting" aria-hidden="true">
          <img class="h-5 w-5 icon-muted" src="/assets/icons/runtime-ui/icon-search-supporting-light.svgz" data-svgz-light="/assets/icons/runtime-ui/icon-search-supporting-light.svgz" data-svgz-dark="/assets/icons/runtime-ui/icon-search-supporting-dark.svgz" alt="" aria-hidden="true" />
        </span>
      </div>
    </div>
    <div id="station-picker-correction" class="mt-2 hidden"></div>
    <div id="station-picker-suggestions" class="mt-3 max-h-[220px] space-y-1 overflow-y-auto"></div>
    <div class="mt-3 flex flex-wrap gap-2">
      <button type="button" id="station-tab-favorites" class="btn-primary h-9 px-3" data-i18n="station.tab_favorites">Favorites</button>
      <button type="button" id="station-tab-region" class="btn-ghost h-9 px-3" data-i18n="station.tab_region">By region</button>
    </div>
    <div id="station-picker-help" class="mt-2 text-xs txt-supporting"></div>
    <div id="station-picker-regions" class="mt-3 flex flex-wrap gap-2"></div>
    <div id="station-picker-list" class="mt-3 max-h-[240px] space-y-1 overflow-y-auto"></div>
  </div>
</div>

<div id="date-picker-modal" class="app-modal-backdrop hidden" aria-hidden="true">
  <div class="app-modal-card max-w-[540px]" role="dialog" aria-modal="true" aria-labelledby="date-picker-title">
    <div class="flex items-center justify-between">
      <h4 id="date-picker-title" class="text-base font-semibold txt-strong" data-i18n="date.modal_title">Departure date</h4>
      <button id="date-picker-close" type="button" class="btn-ghost h-9 w-9 p-0" aria-label="Close" data-i18n-aria-label="common.close">✕</button>
    </div>
    <div class="mt-3">
      <label class="field-label" for="date-picker-input" data-i18n="date.label">Date</label>
      <input id="date-picker-input" type="date" class="field-input h-11 w-full" />
    </div>
    <div class="mt-4 grid grid-cols-2 gap-2">
      <button id="date-picker-cancel" type="button" class="btn-ghost h-11 w-full" data-i18n="common.cancel">Cancel</button>
      <button id="date-picker-apply" type="button" class="btn-primary h-11 w-full" data-i18n="common.apply">Apply</button>
    </div>
  </div>
</div>

<div id="passenger-picker-modal" class="app-modal-backdrop hidden" aria-hidden="true">
  <div class="app-modal-card max-w-[540px]" role="dialog" aria-modal="true" aria-labelledby="passenger-picker-title">
    <div class="flex items-center justify-between">
      <h4 id="passenger-picker-title" class="text-base font-semibold txt-strong" data-i18n="passenger.modal_title">Passengers</h4>
      <button id="passenger-picker-close" type="button" class="btn-ghost h-9 w-9 p-0" aria-label="Close" data-i18n-aria-label="common.close">✕</button>
    </div>
    <div id="passenger-picker-rows" class="mt-3 space-y-2"></div>
    <div class="mt-4 grid grid-cols-2 gap-2">
      <button id="passenger-picker-cancel" type="button" class="btn-ghost h-11 w-full" data-i18n="common.cancel">Cancel</button>
      <button id="passenger-picker-apply" type="button" class="btn-primary h-11 w-full" data-i18n="common.apply">Apply</button>
    </div>
  </div>
</div>"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::render_dashboard_train;

    #[test]
    fn train_page_keeps_station_selector_on_one_mobile_row() {
        let html = render_dashboard_train("admin@bominal.local");

        assert!(
            html.contains(r#"grid-cols-[minmax(0,1fr)_44px_minmax(0,1fr)]"#),
            "expected one-line mobile station selector grid class in train page markup: {html}"
        );
        assert!(
            !html.contains(r#"row-span-2"#),
            "swap control should no longer span two mobile rows: {html}"
        );
        assert!(
            html.contains(r#"id="station-swap""#) && html.contains("data-station-swap-icon"),
            "station swap control should use the circular rotating-arrow icon button: {html}"
        );
        assert!(
            !html.contains(">↔</button>"),
            "station swap control should no longer render the raw arrow glyph: {html}"
        );
    }

    #[test]
    fn train_page_removes_time_picker_and_uses_quick_passenger_controls_with_modal() {
        let html = render_dashboard_train("admin@bominal.local");
        let train_js = include_str!("../../../../../frontend/assets/js/dashboard/train.js");

        assert!(
            !html.contains(r#"id="dep-time-open""#),
            "time button should be removed from the train form: {html}"
        );
        assert!(
            !html.contains(r#"id="time-picker-modal""#),
            "time picker modal should not be rendered: {html}"
        );
        assert!(
            html.contains(r#"id="passenger-minus""#),
            "expected quick passenger decrement control in train page markup: {html}"
        );
        assert!(
            html.contains(r#"id="passenger-open""#),
            "expected quick passenger summary control in train page markup: {html}"
        );
        assert!(
            html.contains(r#"id="passenger-plus""#),
            "expected quick passenger increment control in train page markup: {html}"
        );
        assert!(
            html.contains(r#"id="passenger-picker-modal""#),
            "granular passenger modal should remain available from the quick control: {html}"
        );
        assert!(
            !train_js.contains("dep-time-open"),
            "train frontend should not bind a removed departure time control"
        );
        assert!(
            train_js.contains("passenger-picker-modal"),
            "train frontend should still support the granular passenger modal"
        );
        assert!(
            train_js.contains("passenger.summary.adult"),
            "quick passenger summary should include the adult-specific label path"
        );
    }

    #[test]
    fn train_page_uses_icon_toggle_buttons_for_task_options() {
        let html = render_dashboard_train("admin@bominal.local");

        assert!(
            html.contains(r#"id="task-auto-pay-toggle""#),
            "expected auto pay icon toggle button in task controls: {html}"
        );
        assert!(
            html.contains(r#"id="task-notify-email-toggle""#),
            "expected notify icon toggle button in task controls: {html}"
        );
        assert!(
            html.contains(r#"id="task-retry-expiry-toggle""#),
            "expected retry icon toggle button in task controls: {html}"
        );
        assert!(
            !html.contains(r#"id="task-auto-pay" type="checkbox""#),
            "legacy auto pay checkbox should be removed once icon toggles are in place: {html}"
        );
    }

    #[test]
    fn train_frontend_capitalizes_english_station_labels() {
        let train_js = include_str!("../../../../../frontend/assets/js/dashboard/train.js");

        assert!(
            train_js.contains("capitalizeEnglishStationName"),
            "expected english station-name capitalization helper in train frontend"
        );
        assert!(
            train_js.contains("replace(/\\b[a-z]/g"),
            "expected english station labels to capitalize the first letter of each word"
        );
    }

    #[test]
    fn train_page_uses_favorites_tab_instead_of_major_stations() {
        let html = render_dashboard_train("admin@bominal.local");
        let train_js = include_str!("../../../../../frontend/assets/js/dashboard/train.js");

        assert!(
            html.contains(r#"id="station-tab-favorites""#),
            "expected favorites station tab in picker markup: {html}"
        );
        assert!(
            !html.contains(r#"id="station-tab-major""#),
            "legacy major stations tab should be removed from picker markup: {html}"
        );
        assert!(
            train_js.contains("station.tab_favorites"),
            "expected favorites tab translation key in train frontend"
        );
        assert!(
            !train_js.contains("station.tab_major"),
            "legacy major stations translation key should be removed from train frontend"
        );
        assert!(
            train_js.contains("empty.favorites"),
            "expected dedicated empty favorites copy in train frontend"
        );
    }

    #[test]
    fn train_page_disables_search_until_required_fields_are_valid() {
        let html = render_dashboard_train("admin@bominal.local");
        let train_js = include_str!("../../../../../frontend/assets/js/dashboard/train.js");

        assert!(
            html.contains(r#"id="train-search-submit""#),
            "search submit should be present in train form markup: {html}"
        );
        assert!(
            train_js.contains("syncSearchSubmitState"),
            "train frontend should actively sync search button validity state"
        );
        assert!(
            train_js.contains("searchValidationMessage"),
            "train frontend should surface a local validation error before provider search"
        );
    }
}
