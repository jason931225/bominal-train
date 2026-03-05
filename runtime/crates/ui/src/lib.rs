#![forbid(unsafe_code)]

pub use bominal_ui_patterns::*;
pub use bominal_ui_primitives::*;

#[cfg(test)]
mod tests {
    use super::{DashboardSection, ThemeMode};

    #[test]
    fn re_exports_patterns_and_primitives() {
        let _ = ThemeMode::Light;
        let _ = DashboardSection::Home;
    }
}
