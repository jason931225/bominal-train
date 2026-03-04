#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Destructive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Sm,
    Md,
    Lg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonType {
    Button,
    Submit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonRole {
    Primary,
    Secondary,
    Destructive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonProps<'a> {
    pub id: Option<&'a str>,
    pub label: &'a str,
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub button_type: ButtonType,
    pub disabled: bool,
    pub aria_label: Option<&'a str>,
    pub action_role: Option<ButtonRole>,
    pub extra_class: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkButtonProps<'a> {
    pub href: &'a str,
    pub label: &'a str,
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub active: bool,
    pub extra_class: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusTone {
    Neutral,
    Info,
    Success,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldState {
    Default,
    Error,
    Success,
    Disabled,
    ReadOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateBlockKind {
    Loading,
    Empty,
    Error,
    Success,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateBlockProps<'a> {
    pub kind: StateBlockKind,
    pub message: &'a str,
    pub request_id: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputProps<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub input_type: &'a str,
    pub placeholder: Option<&'a str>,
    pub autocomplete: Option<&'a str>,
    pub value: Option<&'a str>,
    pub state: FieldState,
    pub required: bool,
    pub max_length: Option<usize>,
    pub hint: Option<&'a str>,
    pub error: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavItem<'a> {
    pub label: &'a str,
    pub href: &'a str,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopBarProps<'a> {
    pub brand: &'a str,
    pub title: &'a str,
    pub subtitle: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarNavProps<'a> {
    pub heading: &'a str,
    pub items: &'a [NavItem<'a>],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BottomNavProps<'a> {
    pub items: &'a [NavItem<'a>],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryRowProps<'a> {
    pub label: &'a str,
    pub value: &'a str,
    pub tone: StatusTone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalAction<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub variant: ButtonVariant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalDialogProps<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub message: &'a str,
    pub actions: &'a [ModalAction<'a>],
}

pub fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn render_theme_mini_switch(extra_class: Option<&str>) -> String {
    let mut classes = String::from("theme-mini-switch");
    if let Some(extra) = extra_class.filter(|value| !value.trim().is_empty()) {
        classes.push(' ');
        classes.push_str(extra);
    }
    format!(
        r#"<button
        type="button"
        class="{classes}"
        data-theme-toggle
        data-theme-toggle-compact
        aria-label="Theme toggle"
      >
        <svg class="theme-mini-icon theme-mini-icon-sun" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="12" cy="12" r="4"></circle>
          <line x1="12" y1="2" x2="12" y2="4.5"></line>
          <line x1="12" y1="19.5" x2="12" y2="22"></line>
          <line x1="4.9" y1="4.9" x2="6.7" y2="6.7"></line>
          <line x1="17.3" y1="17.3" x2="19.1" y2="19.1"></line>
          <line x1="2" y1="12" x2="4.5" y2="12"></line>
          <line x1="19.5" y1="12" x2="22" y2="12"></line>
          <line x1="4.9" y1="19.1" x2="6.7" y2="17.3"></line>
          <line x1="17.3" y1="6.7" x2="19.1" y2="4.9"></line>
        </svg>
        <svg class="theme-mini-icon theme-mini-icon-moon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21 12.8A8.5 8.5 0 1 1 11.2 3a6.8 6.8 0 0 0 9.8 9.8z"></path>
        </svg>
        <span class="theme-mini-thumb" aria-hidden="true"></span>
      </button>"#,
    )
}

pub fn render_top_bar(props: &TopBarProps<'_>) -> String {
    format!(
        r#"<header class="mx-auto w-full max-w-[480px] px-4 pt-4 md:max-w-7xl md:px-6">
  <div class="glass-card rounded-[20px] p-4">
    <div class="flex items-start justify-between gap-3">
      <div>
        <p class="eyebrow">{}</p>
        <h1 class="mt-1 text-xl font-semibold txt-strong">{}</h1>
      </div>
      {}
    </div>
    <p class="mt-1 text-sm txt-supporting">{}</p>
  </div>
</header>"#,
        html_escape(props.brand),
        html_escape(props.title),
        render_theme_mini_switch(Some("theme-inline-switch")),
        html_escape(props.subtitle),
    )
}

pub fn render_sidebar_nav(props: &SidebarNavProps<'_>) -> String {
    let links = props
        .items
        .iter()
        .map(|item| {
            format!(
                r#"<a href="{}" class="desktop-side-link {}">{}</a>"#,
                html_escape(item.href),
                if item.active { "active" } else { "" },
                html_escape(item.label),
            )
        })
        .collect::<Vec<_>>()
        .join("\n      ");

    format!(
        r#"<aside class="hidden md:sticky md:top-6 md:block md:self-start">
  <div class="glass-card rounded-[22px] p-3">
    <p class="eyebrow px-3 pt-1">{}</p>
    <nav class="mt-2 space-y-1">
      {}
    </nav>
  </div>
</aside>"#,
        html_escape(props.heading),
        links,
    )
}

pub fn render_bottom_nav(props: &BottomNavProps<'_>) -> String {
    let links = props
        .items
        .iter()
        .map(|item| {
            format!(
                r#"<a href="{}" class="{}">{}</a>"#,
                html_escape(item.href),
                if item.active { "active" } else { "" },
                html_escape(item.label),
            )
        })
        .collect::<Vec<_>>()
        .join("\n  ");

    format!(r#"<nav class="bottom-nav">\n  {}\n</nav>"#, links)
}

fn button_height_class(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::Sm => "h-10",
        ButtonSize::Md => "h-11",
        ButtonSize::Lg => "h-12",
    }
}

fn button_variant_class(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Primary => "btn-primary",
        ButtonVariant::Secondary => "btn-primary",
        ButtonVariant::Ghost => "btn-ghost",
        ButtonVariant::Destructive => "btn-destructive",
    }
}

fn button_role_attr(role: ButtonRole) -> &'static str {
    match role {
        ButtonRole::Primary => "primary",
        ButtonRole::Secondary => "secondary",
        ButtonRole::Destructive => "destructive",
    }
}

pub fn render_button(props: &ButtonProps<'_>) -> String {
    let mut class_name = String::new();
    class_name.push_str(button_variant_class(props.variant));
    class_name.push(' ');
    class_name.push_str(button_height_class(props.size));
    class_name.push_str(" w-full");
    if let Some(extra) = props.extra_class.filter(|value| !value.trim().is_empty()) {
        class_name.push(' ');
        class_name.push_str(extra);
    }

    let mut attrs = String::new();
    if let Some(id) = props.id {
        attrs.push_str(&format!(r#" id="{}""#, html_escape(id)));
    }
    if let Some(label) = props.aria_label {
        attrs.push_str(&format!(r#" aria-label="{}""#, html_escape(label)));
    }
    if let Some(role) = props.action_role {
        attrs.push_str(&format!(
            r#" data-action-role="{}""#,
            button_role_attr(role)
        ));
    }
    if props.disabled {
        attrs.push_str(" disabled");
    }
    let button_type = match props.button_type {
        ButtonType::Button => "button",
        ButtonType::Submit => "submit",
    };

    format!(
        r#"<button type="{}" class="{}"{}>{}</button>"#,
        button_type,
        class_name,
        attrs,
        html_escape(props.label),
    )
}

pub fn render_link_button(props: &LinkButtonProps<'_>) -> String {
    let mut class_name = String::new();
    class_name.push_str(button_variant_class(props.variant));
    class_name.push(' ');
    class_name.push_str(button_height_class(props.size));
    class_name.push_str(" w-full text-center leading-[3rem]");
    if props.active {
        class_name.push_str(" active");
    }
    if let Some(extra) = props.extra_class.filter(|value| !value.trim().is_empty()) {
        class_name.push(' ');
        class_name.push_str(extra);
    }

    format!(
        r#"<a href="{}" class="{}">{}</a>"#,
        html_escape(props.href),
        class_name,
        html_escape(props.label),
    )
}

fn state_tone_class(tone: StatusTone) -> &'static str {
    match tone {
        StatusTone::Neutral => "txt-supporting",
        StatusTone::Info => "txt-accent",
        StatusTone::Success => "txt-positive",
        StatusTone::Warning => "txt-supporting",
        StatusTone::Critical => "txt-critical",
    }
}

pub fn render_summary_row(props: &SummaryRowProps<'_>) -> String {
    format!(
        r#"<div class="summary-row"><span>{}</span><span class="{}">{}</span></div>"#,
        html_escape(props.label),
        state_tone_class(props.tone),
        html_escape(props.value),
    )
}

pub fn render_text_input(props: &TextInputProps<'_>) -> String {
    let mut input_class = String::from("field-input h-11 w-full");
    if matches!(props.state, FieldState::Error) {
        input_class.push_str(" border-rose-400");
    }

    let mut attrs = String::new();
    if let Some(placeholder) = props.placeholder {
        attrs.push_str(&format!(r#" placeholder="{}""#, html_escape(placeholder)));
    }
    if let Some(autocomplete) = props.autocomplete {
        attrs.push_str(&format!(r#" autocomplete="{}""#, html_escape(autocomplete)));
    }
    if let Some(value) = props.value {
        attrs.push_str(&format!(r#" value="{}""#, html_escape(value)));
    }
    if let Some(max_length) = props.max_length {
        attrs.push_str(&format!(r#" maxlength="{}""#, max_length));
    }
    if props.required {
        attrs.push_str(" required");
    }
    if matches!(props.state, FieldState::Disabled) {
        attrs.push_str(" disabled");
    }
    if matches!(props.state, FieldState::ReadOnly) {
        attrs.push_str(" readonly");
    }

    let mut description = String::new();
    if let Some(hint) = props.hint.filter(|value| !value.trim().is_empty()) {
        description.push_str(&format!(
            r#"<p class="mt-1 text-xs txt-supporting">{}</p>"#,
            html_escape(hint),
        ));
    }
    if let Some(error) = props.error.filter(|value| !value.trim().is_empty()) {
        description.push_str(&format!(
            r#"<p class="mt-1 text-xs txt-critical">{}</p>"#,
            html_escape(error),
        ));
    }

    format!(
        r#"<div>
  <label class="field-label" for="{}">{}</label>
  <input id="{}" type="{}" class="{}"{} />
  {}
</div>"#,
        html_escape(props.id),
        html_escape(props.label),
        html_escape(props.id),
        html_escape(props.input_type),
        input_class,
        attrs,
        description,
    )
}

pub fn render_state_block(props: &StateBlockProps<'_>) -> String {
    let mut message = html_escape(props.message);
    if let Some(request_id) = props.request_id.filter(|value| !value.trim().is_empty()) {
        message.push_str(" (request_id: ");
        message.push_str(&html_escape(request_id));
        message.push(')');
    }

    let class_name = match props.kind {
        StateBlockKind::Loading => "loading-card",
        StateBlockKind::Empty => "empty-card",
        StateBlockKind::Error => "error-card",
        StateBlockKind::Success => "summary-card",
    };
    format!(r#"<div class="{}">{}</div>"#, class_name, message)
}

pub fn render_modal_dialog(props: &ModalDialogProps<'_>) -> String {
    let actions = props
        .actions
        .iter()
        .map(|action| {
            render_button(&ButtonProps {
                id: Some(action.id),
                label: action.label,
                variant: action.variant,
                size: ButtonSize::Md,
                button_type: ButtonType::Button,
                disabled: false,
                aria_label: None,
                action_role: Some(match action.variant {
                    ButtonVariant::Destructive => ButtonRole::Destructive,
                    ButtonVariant::Primary | ButtonVariant::Secondary => ButtonRole::Primary,
                    ButtonVariant::Ghost => ButtonRole::Secondary,
                }),
                extra_class: Some(""),
            })
        })
        .collect::<Vec<_>>()
        .join("\n      ");

    format!(
        r#"<div id="{}" class="app-modal-backdrop hidden" role="dialog" aria-modal="true" aria-labelledby="{}-title">
  <div class="app-modal-card">
    <h3 id="{}-title" class="text-base font-semibold txt-strong">{}</h3>
    <p class="mt-2 text-sm txt-supporting">{}</p>
    <div class="action-group action-pair" data-action-group="pair">
      {}
    </div>
  </div>
</div>"#,
        html_escape(props.id),
        html_escape(props.id),
        html_escape(props.id),
        html_escape(props.title),
        html_escape(props.message),
        actions,
    )
}
