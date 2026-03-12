//! SRT passenger types and form-encoding for reservation requests.
//!
//! Ported from `third_party/srt/SRT/passenger.py`.
//! Type codes: "1"=adult, "5"=child, "4"=senior, "2"=disability1-3, "3"=disability4-6.

/// Passenger type for SRT reservations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassengerType {
    /// 어른/청소년 (Adult/Youth) — type code "1"
    Adult,
    /// 어린이 (Child) — type code "5"
    Child,
    /// 경로 (Senior) — type code "4"
    Senior,
    /// 장애 1~3급 (Disability grade 1-3) — type code "2"
    Disability1To3,
    /// 장애 4~6급 (Disability grade 4-6) — type code "3"
    Disability4To6,
}

impl PassengerType {
    /// SRT type code string.
    pub fn type_code(self) -> &'static str {
        match self {
            Self::Adult => "1",
            Self::Child => "5",
            Self::Senior => "4",
            Self::Disability1To3 => "2",
            Self::Disability4To6 => "3",
        }
    }
}

/// A group of passengers of the same type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassengerGroup {
    pub passenger_type: PassengerType,
    pub count: u8,
}

impl PassengerGroup {
    pub fn new(passenger_type: PassengerType, count: u8) -> Self {
        Self {
            passenger_type,
            count,
        }
    }

    pub fn adults(count: u8) -> Self {
        Self::new(PassengerType::Adult, count)
    }
}

/// Window seat preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowSeat {
    /// No preference — code "000"
    None,
    /// Window seat — code "012"
    Window,
    /// Aisle seat — code "013"
    Aisle,
}

impl WindowSeat {
    fn code(self) -> &'static str {
        match self {
            Self::None => "000",
            Self::Window => "012",
            Self::Aisle => "013",
        }
    }
}

/// Combine multiple passenger groups of the same type.
///
/// # Examples
///
/// ```
/// use bominal_provider::srt::passenger::{PassengerGroup, PassengerType, combine_passengers};
/// let groups = vec![
///     PassengerGroup::new(PassengerType::Adult, 1),
///     PassengerGroup::new(PassengerType::Adult, 2),
///     PassengerGroup::new(PassengerType::Child, 1),
/// ];
/// let combined = combine_passengers(&groups);
/// assert_eq!(combined.len(), 2);
/// assert_eq!(combined[0].count, 3); // adults combined
/// ```
pub fn combine_passengers(passengers: &[PassengerGroup]) -> Vec<PassengerGroup> {
    let mut combined: Vec<PassengerGroup> = Vec::new();
    for p in passengers {
        if let Some(existing) = combined
            .iter_mut()
            .find(|c| c.passenger_type == p.passenger_type)
        {
            existing.count = existing.count.saturating_add(p.count);
        } else {
            combined.push(p.clone());
        }
    }
    combined.retain(|p| p.count > 0);
    combined
}

/// Total passenger count.
pub fn total_count(passengers: &[PassengerGroup]) -> u8 {
    passengers.iter().map(|p| p.count).sum()
}

/// Build the form-encoded passenger dictionary for SRT reserve requests.
///
/// Follows the exact field naming from `passenger.py:get_passenger_dict`.
pub fn passenger_form_fields(
    passengers: &[PassengerGroup],
    special_seat: bool,
    window_seat: WindowSeat,
) -> Vec<(String, String)> {
    let combined = combine_passengers(passengers);
    let total: u8 = total_count(&combined);

    let mut fields = vec![
        ("totPrnb".to_string(), total.to_string()),
        ("psgGridcnt".to_string(), combined.len().to_string()),
    ];

    for (idx, group) in combined.iter().enumerate() {
        let i = idx + 1;
        fields.push((
            format!("psgTpCd{i}"),
            group.passenger_type.type_code().to_string(),
        ));
        fields.push((format!("psgInfoPerPrnb{i}"), group.count.to_string()));
        fields.push((format!("locSeatAttCd{i}"), window_seat.code().to_string()));
        fields.push((format!("rqSeatAttCd{i}"), "015".to_string()));
        fields.push((format!("dirSeatAttCd{i}"), "009".to_string()));
        fields.push((format!("smkSeatAttCd{i}"), "000".to_string()));
        fields.push((format!("etcSeatAttCd{i}"), "000".to_string()));
        // seat type: "1" = 일반실, "2" = 특실
        fields.push((
            format!("psrmClCd{i}"),
            if special_seat { "2" } else { "1" }.to_string(),
        ));
    }

    fields
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn type_codes() {
        assert_eq!(PassengerType::Adult.type_code(), "1");
        assert_eq!(PassengerType::Child.type_code(), "5");
        assert_eq!(PassengerType::Senior.type_code(), "4");
        assert_eq!(PassengerType::Disability1To3.type_code(), "2");
        assert_eq!(PassengerType::Disability4To6.type_code(), "3");
    }

    #[test]
    fn combine_same_type() {
        let groups = vec![PassengerGroup::adults(1), PassengerGroup::adults(2)];
        let combined = combine_passengers(&groups);
        assert_eq!(combined.len(), 1);
        assert_eq!(combined[0].count, 3);
    }

    #[test]
    fn combine_different_types() {
        let groups = vec![
            PassengerGroup::adults(2),
            PassengerGroup::new(PassengerType::Child, 1),
        ];
        let combined = combine_passengers(&groups);
        assert_eq!(combined.len(), 2);
    }

    #[test]
    fn total_count_works() {
        let groups = vec![
            PassengerGroup::adults(2),
            PassengerGroup::new(PassengerType::Child, 1),
        ];
        assert_eq!(total_count(&groups), 3);
    }

    #[test]
    fn passenger_form_fields_single_adult() {
        let groups = vec![PassengerGroup::adults(1)];
        let fields = passenger_form_fields(&groups, false, WindowSeat::None);

        let map: HashMap<String, String> = fields.into_iter().collect();
        assert_eq!(map.get("totPrnb").unwrap(), "1");
        assert_eq!(map.get("psgGridcnt").unwrap(), "1");
        assert_eq!(map.get("psgTpCd1").unwrap(), "1");
        assert_eq!(map.get("psgInfoPerPrnb1").unwrap(), "1");
        assert_eq!(map.get("psrmClCd1").unwrap(), "1"); // general
        assert_eq!(map.get("locSeatAttCd1").unwrap(), "000");
    }

    #[test]
    fn passenger_form_fields_special_seat() {
        let groups = vec![PassengerGroup::adults(1)];
        let fields = passenger_form_fields(&groups, true, WindowSeat::Window);

        let map: HashMap<String, String> = fields.into_iter().collect();
        assert_eq!(map.get("psrmClCd1").unwrap(), "2"); // special
        assert_eq!(map.get("locSeatAttCd1").unwrap(), "012"); // window
    }

    #[test]
    fn passenger_form_fields_mixed() {
        let groups = vec![
            PassengerGroup::adults(2),
            PassengerGroup::new(PassengerType::Senior, 1),
        ];
        let fields = passenger_form_fields(&groups, false, WindowSeat::None);

        let map: HashMap<String, String> = fields.into_iter().collect();
        assert_eq!(map.get("totPrnb").unwrap(), "3");
        assert_eq!(map.get("psgGridcnt").unwrap(), "2");
        assert_eq!(map.get("psgTpCd1").unwrap(), "1"); // adult
        assert_eq!(map.get("psgTpCd2").unwrap(), "4"); // senior
    }
}
