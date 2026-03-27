//! App-facing typed surface re-exported from the canonical domain crate.

use serde::{Deserialize, Serialize};

pub use bominal_domain::auth::AuthResponse;
pub use bominal_domain::dto::{
    CardInfo, CreateTaskInput, ProviderInfo, ReservationInfo, StationInfo, TicketInfo, TrainInfo,
    UpdateTaskInput,
};
pub use bominal_domain::i18n::Locale;
pub use bominal_domain::reservation::TrainSchedule;
pub use bominal_domain::task::{
    PassengerCount, PassengerKind, PassengerList, Provider, ReservationSnapshot, ReservationTask,
    SeatPreference, TargetTrain, TargetTrainList, TaskStatus,
};
pub use bominal_domain::task_event::TaskEvent;
pub use bominal_domain::user::User;

pub type TaskInfo = ReservationTask;
pub type UserInfo = AuthResponse;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuggestMatch {
    pub name_ko: String,
    pub name_en: String,
    pub name_ja: String,
    pub score: usize,
    pub confidence: f32,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuggestResult {
    pub matches: Vec<SuggestMatch>,
    pub corrected_query: Option<String>,
    pub autocorrect_applied: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use serde::{Serialize, de::DeserializeOwned};
    use serde_json::Value;
    use uuid::Uuid;

    fn roundtrip<T>(value: &T)
    where
        T: Serialize + DeserializeOwned,
    {
        let json = serde_json::to_value(value).expect("serialize");
        let decoded: T = serde_json::from_value(json.clone()).expect("deserialize");
        let json_after = serde_json::to_value(decoded).expect("re-serialize");
        assert_eq!(json, json_after);
    }

    fn sample_uuid() -> Uuid {
        Uuid::nil()
    }

    fn sample_time() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 27, 12, 0, 0).unwrap()
    }

    #[test]
    fn typed_domain_surface_roundtrips_through_serde() {
        roundtrip(&Provider::Srt);
        roundtrip(&SeatPreference::GeneralFirst);
        roundtrip(&TaskStatus::Confirmed);
        roundtrip(&PassengerKind::Adult);
        roundtrip(&PassengerCount::new(PassengerKind::Adult, 2));
        roundtrip(&PassengerList(vec![PassengerCount::new(
            PassengerKind::Adult,
            2,
        )]));
        roundtrip(&TargetTrain {
            train_number: "305".into(),
            dep_time: "083000".into(),
        });
        roundtrip(&TargetTrainList(vec![TargetTrain {
            train_number: "101".into(),
            dep_time: "073000".into(),
        }]));
        roundtrip(&ReservationSnapshot {
            dep_station: "서울".into(),
            arr_station: "부산".into(),
            dep_date: "20260327".into(),
            dep_time: "083000".into(),
            train_number: "305".into(),
            total_cost: "59800".into(),
            is_waiting: false,
        });
        roundtrip(&CardInfo {
            id: sample_uuid(),
            label: "Personal".into(),
            last_four: "1234".into(),
            card_type: "visa".into(),
            card_type_name: "Visa".into(),
            created_at: sample_time(),
        });
        roundtrip(&TrainInfo {
            provider: "SRT".into(),
            train_type: "SRT".into(),
            train_type_name: "SRT".into(),
            train_number: "305".into(),
            dep_station: "서울".into(),
            dep_date: "20260327".into(),
            dep_time: "083000".into(),
            arr_station: "부산".into(),
            arr_time: "104500".into(),
            general_available: true,
            special_available: false,
            standby_available: false,
        });
        roundtrip(&StationInfo {
            name_ko: "서울".into(),
            name_en: "Seoul".into(),
            name_ja: "ソウル".into(),
        });
        roundtrip(&SuggestResult {
            matches: vec![SuggestMatch {
                name_ko: "서울".into(),
                name_en: "Seoul".into(),
                name_ja: "ソウル".into(),
                score: 100,
                confidence: 0.99,
                source: "Prefix".into(),
            }],
            corrected_query: None,
            autocorrect_applied: false,
        });
        roundtrip(&ProviderInfo {
            provider: "SRT".into(),
            login_id: "us***@example.com".into(),
            status: "valid".into(),
            last_verified_at: Some("2026-03-27T12:00:00Z".into()),
        });
        roundtrip(&ReservationInfo {
            provider: "KTX".into(),
            reservation_number: "R123".into(),
            train_number: "305".into(),
            train_name: "KTX".into(),
            dep_station: "서울".into(),
            arr_station: "부산".into(),
            dep_date: "20260327".into(),
            dep_time: "083000".into(),
            arr_time: "104500".into(),
            total_cost: "59800".into(),
            seat_count: "1".into(),
            paid: true,
            is_waiting: false,
            payment_deadline_date: "20260326".into(),
            payment_deadline_time: "235900".into(),
        });
        roundtrip(&TicketInfo {
            car: "6".into(),
            seat: "3A".into(),
            seat_type: "General".into(),
            passenger_type: "Adult".into(),
            price: 59800,
            original_price: 59800,
            discount: 0,
        });
        roundtrip(&CreateTaskInput {
            provider: Provider::Srt,
            departure_station: "서울".into(),
            arrival_station: "부산".into(),
            travel_date: "20260327".into(),
            departure_time: "083000".into(),
            passengers: PassengerList(vec![PassengerCount::new(PassengerKind::Adult, 1)]),
            seat_preference: SeatPreference::GeneralFirst,
            target_trains: TargetTrainList(vec![TargetTrain {
                train_number: "305".into(),
                dep_time: "083000".into(),
            }]),
            auto_pay: true,
            payment_card_id: Some(sample_uuid()),
            notify_enabled: true,
            auto_retry: true,
        });
        roundtrip(&UpdateTaskInput {
            status: Some(TaskStatus::Running),
            notify_enabled: Some(true),
            auto_retry: Some(false),
            target_trains: Some(TargetTrainList(vec![TargetTrain {
                train_number: "111".into(),
                dep_time: "093000".into(),
            }])),
        });
        roundtrip(&AuthResponse {
            user_id: sample_uuid(),
            email: "user@example.com".into(),
            display_name: "User".into(),
            preferred_locale: "ko".into(),
        });
        roundtrip(&TrainSchedule {
            train_no: "305".into(),
            train_type: "SRT".into(),
            departure_station: "서울".into(),
            arrival_station: "부산".into(),
            departure_time: "083000".into(),
            arrival_time: "104500".into(),
            date: "20260327".into(),
            general_seat_available: true,
            special_seat_available: false,
            standby_available: false,
        });
        roundtrip(&TaskEvent {
            task_id: sample_uuid(),
            status: "running".into(),
            message: "Searching".into(),
            attempt_count: 1,
            reservation_number: None,
        });
        roundtrip(&User {
            id: sample_uuid(),
            email: "user@example.com".into(),
            display_name: "User".into(),
            preferred_locale: Locale::Ko,
            email_verified: true,
            created_at: sample_time(),
        });
        roundtrip(&ReservationTask {
            id: sample_uuid(),
            user_id: sample_uuid(),
            provider: Provider::Srt,
            departure_station: "서울".into(),
            arrival_station: "부산".into(),
            travel_date: "20260327".into(),
            departure_time: "083000".into(),
            passengers: PassengerList(vec![PassengerCount::new(PassengerKind::Adult, 1)]),
            seat_preference: SeatPreference::GeneralFirst,
            target_trains: TargetTrainList(vec![TargetTrain {
                train_number: "305".into(),
                dep_time: "083000".into(),
            }]),
            auto_pay: true,
            payment_card_id: Some(sample_uuid()),
            notify_enabled: true,
            auto_retry: true,
            status: TaskStatus::Queued,
            reservation_number: None,
            reservation: None,
            started_at: None,
            last_attempt_at: None,
            attempt_count: 0,
            created_at: sample_time(),
        });
    }

    #[test]
    fn provider_serializes_to_current_frontend_strings() {
        let value = serde_json::to_value(Provider::Srt).unwrap();
        assert_eq!(value, Value::String("SRT".into()));
    }
}
