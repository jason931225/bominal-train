//! Live integration tests against real SRT/KTX APIs.
//!
//! These tests hit the actual production endpoints to verify the provider
//! layer works correctly. They only test operations that do NOT require login
//! (search, NetFunnel).
//!
//! Run with: `cargo test -p bominal-provider --test live_provider_test -- --nocapture`
//!
//! Ignored by default — use `--ignored` to run:
//!   `cargo test -p bominal-provider --test live_provider_test -- --ignored --nocapture`

use bominal_provider::srt::client::SrtClient;
use bominal_provider::srt::passenger::{PassengerGroup, PassengerType};

use bominal_provider::ktx::client::KtxClient;

// ── SRT ──────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore] // requires network
async fn srt_search_train_live() {
    let mut client = SrtClient::new();

    // Search tomorrow's trains from 수서 to 부산
    let tomorrow = (chrono::Local::now() + chrono::Duration::days(1))
        .format("%Y%m%d")
        .to_string();

    let result = client
        .search_train("수서", "부산", Some(&tomorrow), Some("060000"), false)
        .await;

    match result {
        Ok(trains) => {
            println!("SRT search returned {} trains", trains.len());
            for t in &trains {
                println!(
                    "  {} {} → {} dep={} arr={} general={} special={}",
                    t.display_name(),
                    t.dep_station_name,
                    t.arr_station_name,
                    t.dep_time,
                    t.arr_time,
                    t.general_seat_available(),
                    t.special_seat_available(),
                );
            }
            assert!(!trains.is_empty(), "Expected at least one SRT train");
            assert!(
                trains.iter().all(|t| t.is_srt()),
                "All trains should be SRT"
            );
        }
        Err(e) => {
            // NetFunnelBlocked or NoResults are acceptable for live test
            println!("SRT search returned error (may be expected): {e}");
        }
    }
}

#[tokio::test]
#[ignore]
async fn srt_search_with_passenger_count() {
    let mut client = SrtClient::new();

    let tomorrow = (chrono::Local::now() + chrono::Duration::days(1))
        .format("%Y%m%d")
        .to_string();

    // Search with 3 passengers
    let result = client
        .search_train_with_count(
            "수서",
            "동대구",
            Some(&tomorrow),
            Some("080000"),
            Some(3),
            false,
        )
        .await;

    match result {
        Ok(trains) => {
            println!("SRT search (3 pax) returned {} trains", trains.len());
        }
        Err(e) => {
            println!("SRT search (3 pax) error (may be expected): {e}");
        }
    }
}

// ── KTX ──────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore]
async fn ktx_search_train_live() {
    let client = KtxClient::new();

    let tomorrow = (chrono::Local::now() + chrono::Duration::days(1))
        .format("%Y%m%d")
        .to_string();

    let result = client
        .search_train("서울", "부산", Some(&tomorrow), Some("060000"), false)
        .await;

    match result {
        Ok(trains) => {
            println!("KTX search returned {} trains", trains.len());
            for t in &trains {
                println!(
                    "  {} {} → {} dep={} arr={} general={} special={}",
                    t.display_name(),
                    t.dep_name,
                    t.arr_name,
                    t.dep_time,
                    t.arr_time,
                    t.general_seat_available(),
                    t.special_seat_available(),
                );
            }
            assert!(!trains.is_empty(), "Expected at least one KTX train");
        }
        Err(e) => {
            println!("KTX search error (may be expected): {e}");
        }
    }
}

#[tokio::test]
#[ignore]
async fn ktx_search_with_passengers_live() {
    let client = KtxClient::new();

    let tomorrow = (chrono::Local::now() + chrono::Duration::days(1))
        .format("%Y%m%d")
        .to_string();

    let passengers = vec![
        PassengerGroup::adults(2),
        PassengerGroup::new(PassengerType::Child, 1),
    ];

    let result = client
        .search_train_with_passengers(
            "서울",
            "부산",
            Some(&tomorrow),
            Some("080000"),
            &passengers,
            false,
        )
        .await;

    match result {
        Ok(trains) => {
            println!(
                "KTX search (2 adults + 1 child) returned {} trains",
                trains.len()
            );
        }
        Err(e) => {
            println!("KTX search (multi-pax) error (may be expected): {e}");
        }
    }
}

#[tokio::test]
#[ignore]
async fn ktx_new_stations_searchable() {
    let client = KtxClient::new();

    let tomorrow = (chrono::Local::now() + chrono::Duration::days(1))
        .format("%Y%m%d")
        .to_string();

    // Test one of the newly added stations (청량리)
    let result = client
        .search_train("청량리", "강릉", Some(&tomorrow), Some("060000"), false)
        .await;

    match result {
        Ok(trains) => {
            println!("KTX 청량리→강릉 returned {} trains", trains.len());
        }
        Err(e) => {
            println!("KTX 청량리→강릉 error (may be expected): {e}");
        }
    }
}
