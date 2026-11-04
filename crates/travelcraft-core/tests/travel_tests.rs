use travelcraft_core::{TravelCraftEngine, TravelJournalEntry};

#[test]
fn test_travelcraft_japan_edition() {
    let engine = TravelCraftEngine::new_in_memory().expect("engine init");
    let jp = engine.get_country("JP").expect("fetch JP");
    assert_eq!(jp.country_code, "JP");
    assert!(!jp.curated_routes.is_empty());
    assert!(jp.curated_routes[0].waypoints.len() >= 8);
}

#[test]
fn test_travelcraft_austria_edition() {
    let engine = TravelCraftEngine::new_in_memory().expect("engine init");
    let at = engine.get_country("AT").expect("fetch AT");
    assert_eq!(at.country_code, "AT");
    assert!(!at.curated_routes.is_empty());
    assert!(at.curated_routes[0].waypoints.len() >= 7);
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
        photo_lat: Some(35.1917),
        photo_lon: Some(139.0253),
        photo_gps_source: Some("manual".to_string()),
    };
    engine.add_journal_entry(journal).expect("save journal");
}

#[test]
fn test_itinerary_stay_lengths() {
    let engine = TravelCraftEngine::new_in_memory().expect("engine init");
    for days in [1u32, 2, 3, 5, 7, 14] {
        let itin = engine.plan_itinerary("AT", days).expect("itinerary");
        assert_eq!(itin.total_days, days);
        assert!(!itin.days.is_empty());
        assert!(!itin.practical_tip.is_empty());
    }
    let one = engine.plan_itinerary("JP", 1).expect("1-day");
    let total_stops: usize = one.days.iter().map(|d| d.waypoint_ids.len()).sum();
    assert!(total_stops <= 3, "1-day stays stay walkable");
}

#[test]
fn test_sightlens_stephansdom_and_caravaggio() {
    let engine = TravelCraftEngine::new_in_memory().expect("engine init");
    let steffl = engine
        .identify_sight(48.2085, 16.3738, "stephansdom")
        .expect("stephansdom");
    assert_eq!(steffl.id, "at_stephansdom");
    let cara = engine
        .identify_sight(48.2039, 16.3619, "caravaggio rosenkranzmadonna")
        .expect("caravaggio");
    assert_eq!(cara.id, "khm_caravaggio_rosenkranz");
    let res = engine
        .sightlens_capture(
            "immich-export.jpg",
            Some(48.2085),
            Some(16.3738),
            "immich-exif",
            "stephansdom",
        )
        .expect("capture");
    assert!(res.info_page_title.contains("Stephansdom"));
    assert!(!res.journal_entry_id.is_empty());
}

#[test]
fn test_sight_chat_context_and_offline_answer() {
    let engine = TravelCraftEngine::new_in_memory().expect("engine init");
    let art = engine
        .identify_sight(48.2039, 16.3619, "caravaggio")
        .expect("caravaggio");
    let ctx = engine.sight_chat_context(&art);
    assert!(
        ctx.system_prompt().contains("Caravaggio")
            || ctx.system_prompt().contains("caravaggio")
            || ctx.info_body.len() > 50
    );
    assert!(ctx.map_blob.contains("qgiscraft") || ctx.map_blob.contains("tile"));
    assert!(ctx.transit_blob.contains("KHM") || ctx.transit_blob.contains("TRANSIT"));
    let ans = engine.sight_chat_offline(&art, "Who painted the Rosenkranzmadonna and where is it?");
    assert!(ans.contains("OFFLINE BLOB"));
    assert!(ans.len() > 100);
}
