use bominal_ui::{
    DashboardSection, SettingsTab, html_escape as primitive_html_escape, render_app_topbar,
    render_dashboard_sidebar, render_settings_tabs,
};

pub fn html_escape(value: &str) -> String {
    primitive_html_escape(value)
}

pub fn app_shell_topbar(title: &str, subtitle: &str) -> String {
    render_app_topbar(title, subtitle)
}

pub fn dashboard_desktop_sidebar(active: &str) -> String {
    let section = match active {
        "train" => DashboardSection::Train,
        "jobs" => DashboardSection::Jobs,
        "security" | "settings" => DashboardSection::Settings,
        _ => DashboardSection::Home,
    };
    render_dashboard_sidebar(section)
}

pub fn dashboard_settings_tabs(active: &str) -> String {
    let tab = match active {
        "provider" => SettingsTab::Providers,
        "payment" => SettingsTab::Payment,
        _ => SettingsTab::Account,
    };
    render_settings_tabs(tab)
}
