#![forbid(unsafe_code)]

use bominal_ui_primitives::{
    BottomNavProps, ButtonProps, ButtonRole, ButtonSize, ButtonType, ButtonVariant, FieldState,
    LinkButtonProps, ModalAction, ModalDialogProps, NavItem, SidebarNavProps, StateBlockKind,
    StateBlockProps, StatusTone, SummaryRowProps, TextInputProps, TopBarProps, render_bottom_nav,
    render_button, render_link_button, render_modal_dialog, render_sidebar_nav, render_state_block,
    render_summary_row, render_text_input, render_top_bar,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardSection {
    Home,
    Train,
    Jobs,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Account,
    Providers,
    Payment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminSection {
    Maintenance,
    Users,
    Runtime,
    Observability,
    Security,
    Config,
    Audit,
}

pub fn render_compact_theme_toggle() -> String {
    r#"<button
        type="button"
        class="theme-mini-switch"
        data-theme-toggle
        data-theme-toggle-compact
        aria-label="Theme toggle"
      >
        <img class="theme-mini-icon theme-mini-icon-sun" src="/assets/icons/runtime-ui/theme-mini-sun-active.svgz" data-svgz-light="/assets/icons/runtime-ui/theme-mini-sun-active.svgz" data-svgz-dark="/assets/icons/runtime-ui/theme-mini-sun-default.svgz" alt="" aria-hidden="true" />
        <img class="theme-mini-icon theme-mini-icon-moon" src="/assets/icons/runtime-ui/theme-mini-moon-default.svgz" data-svgz-light="/assets/icons/runtime-ui/theme-mini-moon-default.svgz" data-svgz-dark="/assets/icons/runtime-ui/theme-mini-moon-active.svgz" alt="" aria-hidden="true" />
        <span class="theme-mini-thumb" aria-hidden="true"></span>
      </button>"#
        .to_string()
}

pub fn render_app_topbar(title: &str, subtitle: &str) -> String {
    render_top_bar(&TopBarProps {
        brand: "bominal",
        title,
        subtitle,
    })
}

pub fn render_dashboard_sidebar(active: DashboardSection) -> String {
    let items = [
        NavItem {
            label: "Overview",
            href: "/dashboard",
            active: matches!(active, DashboardSection::Home),
        },
        NavItem {
            label: "Train",
            href: "/dashboard/train",
            active: matches!(active, DashboardSection::Train),
        },
        NavItem {
            label: "Jobs",
            href: "/dashboard/jobs",
            active: matches!(active, DashboardSection::Jobs),
        },
        NavItem {
            label: "Settings",
            href: "/dashboard/settings",
            active: matches!(active, DashboardSection::Settings),
        },
    ];

    render_sidebar_nav(&SidebarNavProps {
        heading: "navigation",
        items: &items,
    })
}

pub fn render_dashboard_bottom_nav(active: DashboardSection) -> String {
    let items = [
        NavItem {
            label: "Home",
            href: "/dashboard",
            active: matches!(active, DashboardSection::Home),
        },
        NavItem {
            label: "Train",
            href: "/dashboard/train",
            active: matches!(active, DashboardSection::Train),
        },
        NavItem {
            label: "Jobs",
            href: "/dashboard/jobs",
            active: matches!(active, DashboardSection::Jobs),
        },
        NavItem {
            label: "Settings",
            href: "/dashboard/settings",
            active: matches!(active, DashboardSection::Settings),
        },
    ];

    render_bottom_nav(&BottomNavProps { items: &items })
}

pub fn render_settings_tabs(active: SettingsTab) -> String {
    let account = render_link_button(&LinkButtonProps {
        href: "/dashboard/settings",
        label: "Account",
        variant: if matches!(active, SettingsTab::Account) {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Ghost
        },
        size: ButtonSize::Sm,
        active: false,
        extra_class: Some("inline-flex items-center justify-center !leading-[2.5rem]"),
    });
    let providers = render_link_button(&LinkButtonProps {
        href: "/dashboard/settings/providers",
        label: "Providers",
        variant: if matches!(active, SettingsTab::Providers) {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Ghost
        },
        size: ButtonSize::Sm,
        active: false,
        extra_class: Some("inline-flex items-center justify-center !leading-[2.5rem]"),
    });
    let payment = render_link_button(&LinkButtonProps {
        href: "/dashboard/payment",
        label: "Payment",
        variant: if matches!(active, SettingsTab::Payment) {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Ghost
        },
        size: ButtonSize::Sm,
        active: false,
        extra_class: Some("inline-flex items-center justify-center !leading-[2.5rem]"),
    });

    format!(
        r#"<nav class="grid grid-cols-3 gap-2" aria-label="Settings tabs">
      {account}
      {providers}
      {payment}
    </nav>"#
    )
}

pub fn render_admin_sidebar(active: AdminSection) -> String {
    let items = [
        NavItem {
            label: "Maintenance",
            href: "/admin/maintenance",
            active: matches!(active, AdminSection::Maintenance),
        },
        NavItem {
            label: "Users",
            href: "/admin/users",
            active: matches!(active, AdminSection::Users),
        },
        NavItem {
            label: "Runtime",
            href: "/admin/runtime",
            active: matches!(active, AdminSection::Runtime),
        },
        NavItem {
            label: "Observability",
            href: "/admin/observability",
            active: matches!(active, AdminSection::Observability),
        },
        NavItem {
            label: "Security",
            href: "/admin/security",
            active: matches!(active, AdminSection::Security),
        },
        NavItem {
            label: "Config",
            href: "/admin/config",
            active: matches!(active, AdminSection::Config),
        },
        NavItem {
            label: "Audit",
            href: "/admin/audit",
            active: matches!(active, AdminSection::Audit),
        },
    ];

    render_sidebar_nav(&SidebarNavProps {
        heading: "ops navigation",
        items: &items,
    })
}

pub fn render_admin_bottom_nav(active: AdminSection) -> String {
    let items = [
        NavItem {
            label: "Maint",
            href: "/admin/maintenance",
            active: matches!(active, AdminSection::Maintenance),
        },
        NavItem {
            label: "Users",
            href: "/admin/users",
            active: matches!(active, AdminSection::Users),
        },
        NavItem {
            label: "Runtime",
            href: "/admin/runtime",
            active: matches!(active, AdminSection::Runtime),
        },
        NavItem {
            label: "Obs",
            href: "/admin/observability",
            active: matches!(active, AdminSection::Observability),
        },
        NavItem {
            label: "Audit",
            href: "/admin/audit",
            active: matches!(active, AdminSection::Audit),
        },
    ];

    render_bottom_nav(&BottomNavProps { items: &items })
}

pub fn render_dev_ui_showcase() -> String {
    let topbar = render_app_topbar(
        "UI Kitchen Sink",
        "Primitives and patterns preview for bominal",
    );

    let buttons = [
        render_button(&ButtonProps {
            id: Some("ui-btn-primary"),
            label: "Primary",
            variant: ButtonVariant::Primary,
            size: ButtonSize::Md,
            button_type: ButtonType::Button,
            disabled: false,
            aria_label: None,
            action_role: Some(ButtonRole::Primary),
            extra_class: Some(""),
        }),
        render_button(&ButtonProps {
            id: Some("ui-btn-ghost"),
            label: "Ghost",
            variant: ButtonVariant::Ghost,
            size: ButtonSize::Md,
            button_type: ButtonType::Button,
            disabled: false,
            aria_label: None,
            action_role: Some(ButtonRole::Secondary),
            extra_class: Some(""),
        }),
        render_button(&ButtonProps {
            id: Some("ui-btn-destructive"),
            label: "Destructive",
            variant: ButtonVariant::Destructive,
            size: ButtonSize::Md,
            button_type: ButtonType::Button,
            disabled: false,
            aria_label: None,
            action_role: Some(ButtonRole::Destructive),
            extra_class: Some(""),
        }),
    ]
    .join("\n");

    let text_input = render_text_input(&TextInputProps {
        id: "ui-email",
        label: "Email",
        input_type: "email",
        placeholder: Some("operator@bominal.com"),
        autocomplete: Some("email"),
        value: None,
        state: FieldState::Default,
        required: true,
        max_length: None,
        hint: Some("Used for operational alerts."),
        error: None,
    });

    let summary_rows = [
        render_summary_row(&SummaryRowProps {
            label: "Readiness",
            value: "Healthy",
            tone: StatusTone::Success,
        }),
        render_summary_row(&SummaryRowProps {
            label: "Error rate",
            value: "0.3%",
            tone: StatusTone::Warning,
        }),
        render_summary_row(&SummaryRowProps {
            label: "Request ID",
            value: "demo-request-id",
            tone: StatusTone::Info,
        }),
    ]
    .join("\n");

    let states = [
        render_state_block(&StateBlockProps {
            kind: StateBlockKind::Loading,
            message: "Loading component preview...",
            request_id: None,
        }),
        render_state_block(&StateBlockProps {
            kind: StateBlockKind::Empty,
            message: "No records available.",
            request_id: None,
        }),
        render_state_block(&StateBlockProps {
            kind: StateBlockKind::Error,
            message: "Could not load preview",
            request_id: Some("ui-demo-req-123"),
        }),
        render_state_block(&StateBlockProps {
            kind: StateBlockKind::Success,
            message: "Saved successfully.",
            request_id: None,
        }),
    ]
    .join("\n");

    let modal = render_modal_dialog(&ModalDialogProps {
        id: "ui-demo-modal",
        title: "Confirm action",
        message: "Example modal with primary and cancel actions.",
        actions: &[
            ModalAction {
                id: "ui-demo-modal-cancel",
                label: "Cancel",
                variant: ButtonVariant::Ghost,
            },
            ModalAction {
                id: "ui-demo-modal-confirm",
                label: "Confirm",
                variant: ButtonVariant::Primary,
            },
        ],
    });

    let nav_preview = render_dashboard_bottom_nav(DashboardSection::Home);

    format!(
        r#"{topbar}
<main class="mx-auto w-full max-w-[480px] px-4 pb-24 pt-4 md:max-w-7xl md:px-6 md:pt-8">
  <section class="space-y-4">
    <section class="glass-card rounded-[22px] p-5">
      <h2 class="text-lg font-semibold txt-strong">Buttons</h2>
      <div class="mt-3 grid grid-cols-1 gap-2 md:grid-cols-3">{buttons}</div>
    </section>
    <section class="glass-card rounded-[22px] p-5">
      <h2 class="text-lg font-semibold txt-strong">Field</h2>
      <div class="mt-3">{text_input}</div>
    </section>
    <section class="glass-card rounded-[22px] p-5">
      <h2 class="text-lg font-semibold txt-strong">Summary Rows</h2>
      <div class="mt-3 space-y-2">{summary_rows}</div>
    </section>
    <section class="glass-card rounded-[22px] p-5">
      <h2 class="text-lg font-semibold txt-strong">Async States</h2>
      <div class="mt-3 space-y-2">{states}</div>
    </section>
    <section class="glass-card rounded-[22px] p-5">
      <h2 class="text-lg font-semibold txt-strong">Modal Preview</h2>
      <p class="mt-2 text-sm txt-supporting">Modal markup is rendered below for visual QA.</p>
      <div class="mt-3">{modal}</div>
    </section>
  </section>
</main>
{nav_preview}"#
    )
}
