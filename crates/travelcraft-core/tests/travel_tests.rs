use travelcraft_core::{TravelCraftEngine, TravelJournalEntry};

#[test]
fn test_travelcraft_japan_edition() {
    let engine = TravelCraftEngine::new_in_memory().expect("engine init");
    let jp = engine.get_country("JP").expect("fetch JP");
    assert_eq!(jp.country_code, "JP");
    assert!(!jp.curated_routes.is_empty());
    assert_eq!(jp.curated_routes[0].waypoints.len(), 3);
}

#[test]
fn test_travelcraft_austria_edition() {
    let engine = TravelCraftEngine::new_in_memory().expect("engine init");
    let at = engine.get_country("AT").expect("fetch AT");
    assert_eq!(at.country_code, "AT");
    assert!(!at.curated_routes.is_empty());
    assert_eq!(at.curated_routes[0].waypoints.len(), 2);
}

#[test]
fn test_travelcraft_journal_and_route_plan() {
    let engine = TravelCraftEngine::new_in_memory().expect("engine init");
    let plan = engine.plan_optimized_route("JP").expect("route plan");
    assert!(plan.total_distance_km > 0.0);
    assert!(plan.chrono_span_years > 100);

    let journal = TravelJournalEntry {
        entry_id: "JOURNAL-001".to_string(),
        waypoint_id: "jp_hakone".to_string(),
        timestamp_epoch: 1793775000,
        user_reflection: "Sulfur steam rising over Old Tokaido cedar path.".to_string(),
        sensory_notes: "Aroma of black eggs boiled in volcanic springs.".to_string(),
        photo_uris: vec!["file:///hakone_lake_ashi.jpg".to_string()],
    };
    engine.add_journal_entry(journal).expect("save journal");
}
