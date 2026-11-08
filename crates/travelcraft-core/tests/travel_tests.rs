#![allow(clippy::unwrap_used, clippy::expect_used)]

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

#[test]
fn test_travelcraft_knowledge_trees_integration() {
    let engine = TravelCraftEngine::new_in_memory().expect("engine init");

    // Austria Knowledge Tree: Extensive Vienna dining & culture
    let at_tree = engine.get_knowledge_tree("AT");
    assert_eq!(at_tree.country_code, "AT");
    let resto_branch = at_tree
        .branches
        .iter()
        .find(|b| b.id == "at_branch_restaurants")
        .expect("Restaurants branch");
    assert!(resto_branch.items.len() >= 6);

    let rules = resto_branch
        .items
        .iter()
        .find(|i| i.id == "at_item_beisl_rules")
        .expect("Beisl rules");
    assert!(rules.body.contains("Fotokarten"));
    assert!(rules.body.contains("Pfiff"));

    let orlik = resto_branch
        .items
        .iter()
        .find(|i| i.id == "at_item_traditional_beisln")
        .expect("Traditional beisln");
    assert!(orlik.body.contains("Gasthaus Orlik"));
    assert!(orlik.body.contains("Plachutta"));

    // Japan Knowledge Tree: Transit, Konbini, Culture
    let jp_tree = engine.get_knowledge_tree("JP");
    assert_eq!(jp_tree.country_code, "JP");
    assert!(jp_tree.branches.len() >= 4);
    let konbini = jp_tree
        .branches
        .iter()
        .find(|b| b.id == "jp_branch_dining")
        .expect("Dining branch")
        .items
        .iter()
        .find(|i| i.id == "jp_item_konbini_survival")
        .expect("Konbini item");
    assert!(konbini.body.contains("Tamago Sando"));
    assert!(konbini.body.contains("Time Sales"));
}

fn temp_db(tag: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir()
        .join(format!(
            "travelcraft-test-{tag}-{}-{nanos}",
            std::process::id()
        ))
        .join("t.db")
}

fn entry(id: &str, text: &str) -> TravelJournalEntry {
    TravelJournalEntry {
        entry_id: id.to_string(),
        waypoint_id: "at_stephansdom".to_string(),
        timestamp_epoch: 1_700_000_000,
        user_reflection: text.to_string(),
        sensory_notes: String::new(),
        photo_uris: vec!["file:///a.jpg".to_string()],
        photo_lat: Some(48.2),
        photo_lon: Some(16.3),
        photo_gps_source: Some("manual".to_string()),
    }
}

#[test]
fn journal_survives_reopen() {
    let path = temp_db("reopen");
    {
        let e = TravelCraftEngine::open(&path).unwrap();
        assert!(e.is_persistent());
        e.add_journal_entry(entry("J-1", "first")).unwrap();
        e.chat_append("at_stephansdom", "you", "hello").unwrap();
        e.setting_set("theme", "light").unwrap();
    }
    let e = TravelCraftEngine::open(&path).unwrap();
    let list = e.list_journal(10).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].entry_id, "J-1");
    assert_eq!(list[0].photo_uris, vec!["file:///a.jpg".to_string()]);
    assert_eq!(
        e.chat_recent(10).unwrap(),
        vec![("you".to_string(), "hello".to_string())]
    );
    assert_eq!(e.setting_get("theme").unwrap().as_deref(), Some("light"));
    assert_eq!(e.setting_get("missing").unwrap(), None);
}

#[test]
fn journal_is_safe_against_quotes_and_duplicates() {
    let e = TravelCraftEngine::new_in_memory().unwrap();
    assert!(!e.is_persistent());
    let nasty = "x'); DROP TABLE travel_journal; --";
    e.add_journal_entry(entry("J-1", nasty)).unwrap();
    assert!(e.add_journal_entry(entry("J-1", "dup")).is_err());
    let list = e.list_journal(10).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].user_reflection, nasty);
    assert_eq!(e.journal_count().unwrap(), 1);
}

#[test]
fn journal_rejects_bad_input() {
    let e = TravelCraftEngine::new_in_memory().unwrap();
    assert!(e.add_journal_entry(entry("J-1", "   ")).is_err());
    assert!(e.add_journal_entry(entry("", "text")).is_err());
    let mut bad = entry("J-2", "text");
    bad.photo_lat = Some(120.0);
    assert!(e.add_journal_entry(bad).is_err());
    let long = "x".repeat(16_001);
    assert!(e.add_journal_entry(entry("J-3", &long)).is_err());
    assert_eq!(e.journal_count().unwrap(), 0);
}

#[test]
fn rapid_captures_get_unique_ids() {
    let e = TravelCraftEngine::new_in_memory().unwrap();
    let a = e
        .sightlens_capture("a.jpg", None, None, "manual", "stephansdom")
        .unwrap();
    let b = e
        .sightlens_capture("b.jpg", None, None, "manual", "stephansdom")
        .unwrap();
    assert_ne!(a.journal_entry_id, b.journal_entry_id);
    assert_eq!(e.journal_count().unwrap(), 2);
}

#[test]
fn chat_history_is_capped_and_ordered() {
    let e = TravelCraftEngine::new_in_memory().unwrap();
    for i in 0..1_010 {
        e.chat_append("s", "you", &format!("q{i}")).unwrap();
    }
    let recent = e.chat_recent(1_000).unwrap();
    assert!(recent.len() <= 1_001);
    assert_eq!(recent.last().unwrap().1, "q1009");
    let last3 = e.chat_recent(3).unwrap();
    assert_eq!(last3[0].1, "q1007");
    assert_eq!(last3[2].1, "q1009");
}
