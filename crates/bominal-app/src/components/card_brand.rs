use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardBrandKind {
    Visa,
    Mastercard,
    Amex,
    Unknown,
}

impl CardBrandKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Visa => "VISA",
            Self::Mastercard => "MC",
            Self::Amex => "AMEX",
            Self::Unknown => "CARD",
        }
    }

    fn class_name(self) -> &'static str {
        match self {
            Self::Visa => "visa",
            Self::Mastercard => "mastercard",
            Self::Amex => "amex",
            Self::Unknown => "generic",
        }
    }
}

pub fn detect_brand(digits: &str) -> CardBrandKind {
    if digits.is_empty() {
        return CardBrandKind::Unknown;
    }

    if digits.len() >= 2 {
        let prefix = &digits[..2];
        if prefix == "34" || prefix == "37" {
            return CardBrandKind::Amex;
        }
    }

    if digits.len() >= 2
        && let Ok(prefix) = digits[..2].parse::<u16>()
        && (51..=55).contains(&prefix)
    {
        return CardBrandKind::Mastercard;
    }

    if digits.len() >= 4
        && let Ok(prefix) = digits[..4].parse::<u16>()
        && (2221..=2720).contains(&prefix)
    {
        return CardBrandKind::Mastercard;
    }

    if digits.starts_with('4') {
        return CardBrandKind::Visa;
    }

    CardBrandKind::Unknown
}

pub fn format_card_number(digits: &str) -> String {
    digits
        .chars()
        .filter(|char| char.is_ascii_digit())
        .enumerate()
        .fold(String::with_capacity(20), |mut value, (index, char)| {
            if index > 0 && index % 4 == 0 {
                value.push(' ');
            }
            value.push(char);
            value
        })
}

pub fn strip_non_digits(input: &str) -> String {
    input.chars().filter(|char| char.is_ascii_digit()).collect()
}

#[component]
pub fn CardBrand(
    #[prop(optional)] kind: Option<CardBrandKind>,
    #[prop(into, optional)] label: Option<String>,
    #[prop(into, default = String::new())] class: String,
) -> impl IntoView {
    let kind = kind.unwrap_or(CardBrandKind::Unknown);
    let label = label.unwrap_or_else(|| kind.label().to_string());

    view! {
        <span class=format!("lg-card-brand lg-card-brand--{} {class}", kind.class_name())>
            {label}
        </span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_supported_card_brands() {
        assert_eq!(detect_brand("4111111111111111"), CardBrandKind::Visa);
        assert_eq!(detect_brand("5100000000000000"), CardBrandKind::Mastercard);
        assert_eq!(detect_brand("341111111111111"), CardBrandKind::Amex);
    }

    #[test]
    fn formats_card_numbers_into_groups() {
        assert_eq!(
            format_card_number("4111111111111111"),
            "4111 1111 1111 1111"
        );
    }

    #[test]
    fn strips_non_digit_characters() {
        assert_eq!(strip_non_digits("4111-1111 1111"), "411111111111");
    }
}
