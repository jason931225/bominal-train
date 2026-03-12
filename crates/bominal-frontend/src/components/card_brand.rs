//! Credit card brand detection and number formatting utilities.

/// Known card brands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardBrand {
    Visa,
    Mastercard,
    Amex,
    Unknown,
}

impl CardBrand {
    /// Short text label for display in the UI.
    pub fn label(self) -> &'static str {
        match self {
            Self::Visa => "VISA",
            Self::Mastercard => "MC",
            Self::Amex => "AMEX",
            Self::Unknown => "",
        }
    }

    /// Badge background color CSS variable.
    pub fn badge_color(self) -> &'static str {
        match self {
            Self::Visa => "#1a1f71",
            Self::Mastercard => "#eb001b",
            Self::Amex => "#006fcf",
            Self::Unknown => "var(--color-bg-sunken)",
        }
    }
}

/// Detect card brand from the first digits of a card number.
///
/// Rules:
/// - Visa: starts with 4
/// - Mastercard: starts with 51-55 or 2221-2720
/// - AMEX: starts with 34 or 37
pub fn detect_brand(digits: &str) -> CardBrand {
    if digits.is_empty() {
        return CardBrand::Unknown;
    }

    // AMEX: starts with 34 or 37
    if digits.len() >= 2 {
        let two = &digits[..2];
        if two == "34" || two == "37" {
            return CardBrand::Amex;
        }
    }

    // Mastercard: 51-55 range
    if digits.len() >= 2 {
        if let Ok(two_digit) = digits[..2].parse::<u16>() {
            if (51..=55).contains(&two_digit) {
                return CardBrand::Mastercard;
            }
        }
    }

    // Mastercard: 2221-2720 range
    if digits.len() >= 4 {
        if let Ok(four_digit) = digits[..4].parse::<u16>() {
            if (2221..=2720).contains(&four_digit) {
                return CardBrand::Mastercard;
            }
        }
    }

    // Visa: starts with 4
    if digits.starts_with('4') {
        return CardBrand::Visa;
    }

    CardBrand::Unknown
}

/// Format a raw digit string with spaces every 4 characters.
///
/// Example: `"4111111111111111"` -> `"4111 1111 1111 1111"`
pub fn format_card_number(digits: &str) -> String {
    digits
        .chars()
        .filter(|c| c.is_ascii_digit())
        .enumerate()
        .fold(String::with_capacity(20), |mut acc, (i, ch)| {
            if i > 0 && i % 4 == 0 {
                acc.push(' ');
            }
            acc.push(ch);
            acc
        })
}

/// Strip all non-digit characters from a string.
pub fn strip_non_digits(input: &str) -> String {
    input.chars().filter(|c| c.is_ascii_digit()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_visa() {
        assert_eq!(detect_brand("4111111111111111"), CardBrand::Visa);
        assert_eq!(detect_brand("4"), CardBrand::Visa);
        assert_eq!(detect_brand("4999"), CardBrand::Visa);
    }

    #[test]
    fn detect_mastercard_51_55() {
        assert_eq!(detect_brand("5100000000000000"), CardBrand::Mastercard);
        assert_eq!(detect_brand("5500000000000000"), CardBrand::Mastercard);
        assert_eq!(detect_brand("53"), CardBrand::Mastercard);
    }

    #[test]
    fn detect_mastercard_2221_2720() {
        assert_eq!(detect_brand("2221000000000000"), CardBrand::Mastercard);
        assert_eq!(detect_brand("2720999999999999"), CardBrand::Mastercard);
        assert_eq!(detect_brand("2500111111111111"), CardBrand::Mastercard);
    }

    #[test]
    fn detect_amex() {
        assert_eq!(detect_brand("341111111111111"), CardBrand::Amex);
        assert_eq!(detect_brand("371111111111111"), CardBrand::Amex);
        assert_eq!(detect_brand("34"), CardBrand::Amex);
    }

    #[test]
    fn detect_unknown() {
        assert_eq!(detect_brand(""), CardBrand::Unknown);
        assert_eq!(detect_brand("6011111111111117"), CardBrand::Unknown);
        assert_eq!(detect_brand("9"), CardBrand::Unknown);
    }

    #[test]
    fn format_number_groups_of_four() {
        assert_eq!(format_card_number("4111111111111111"), "4111 1111 1111 1111");
        assert_eq!(format_card_number("341111111111111"), "3411 1111 1111 111");
        assert_eq!(format_card_number("4111"), "4111");
        assert_eq!(format_card_number(""), "");
    }

    #[test]
    fn strip_digits() {
        assert_eq!(strip_non_digits("4111 1111 1111 1111"), "4111111111111111");
        assert_eq!(strip_non_digits("abc123def456"), "123456");
    }
}
