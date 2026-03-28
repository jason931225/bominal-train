use leptos::prelude::*;

fn icon_path(name: &str) -> (&'static str, &'static str) {
    match name {
        "train" => (
            "M3 13h2l2-6h10l2 6h2M5 17a2 2 0 104 0m6 0a2 2 0 104 0",
            "round",
        ),
        "payment" => (
            "M2 7h20M4 5h16a2 2 0 012 2v10a2 2 0 01-2 2H4a2 2 0 01-2-2V7a2 2 0 012-2z",
            "round",
        ),
        "appearance" => ("M12 3a9 9 0 100 18A9 9 0 0012 3zm0 0v18M3 12h18", "round"),
        "logout" => (
            "M15 12H3m9-9l9 9-9 9m-3 0H5a2 2 0 01-2-2V5a2 2 0 012-2h4",
            "round",
        ),
        "user" => (
            "M20 21a8 8 0 10-16 0m8-10a4 4 0 100-8 4 4 0 000 8z",
            "round",
        ),
        "globe" => (
            "M2.5 12h19M12 2.5a14.5 14.5 0 010 19m0-19a14.5 14.5 0 000 19M4.9 7.5h14.2M4.9 16.5h14.2",
            "round",
        ),
        "moon" => ("M21 12.8A9 9 0 1111.2 3a7 7 0 009.8 9.8z", "round"),
        "plus" => ("M12 5v14m-7-7h14", "round"),
        "trash" => (
            "M3 6h18m-2 0l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6m3 0V4a1 1 0 011-1h6a1 1 0 011 1v2",
            "round",
        ),
        "edit" => ("M4 20h4l10-10-4-4L4 16v4zm11-13l4 4m-2-6l2 2", "round"),
        "chevron-down" => ("M6 9l6 6 6-6", "round"),
        "check" => ("M5 13l4 4L19 7", "round"),
        _ => ("M5 12h14", "round"),
    }
}

#[component]
pub fn Icon(
    #[prop(into)] name: String,
    #[prop(into, default = String::new())] class: String,
) -> impl IntoView {
    let (path, linecap) = icon_path(&name);

    view! {
        <svg
            class=format!("lg-icon {class}")
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="1.8"
            aria-hidden="true"
        >
            <path stroke-linecap=linecap stroke-linejoin="round" d=path />
        </svg>
    }
}
