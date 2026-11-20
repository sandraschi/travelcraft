#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use travelcraft_core::cities::{places_in_city, TravelCity};
use travelcraft_core::TravelCraftEngine;

#[test]
fn vienna_and_tokyo_places_stay_inside_the_city() {
    let vienna = places_in_city(TravelCity::Vienna);
    assert!(vienna.iter().any(|p| p.id == "at_stephansdom"));
    assert!(vienna.iter().any(|p| p.id == "at_schoenbrunn"));
    assert!(vienna.iter().any(|p| p.id == "leopold_schiele_wally"));
    assert!(vienna.iter().all(|p| p.id != "at_hallstatt"));
    assert!(vienna.iter().all(|p| p.id != "at_wachau"));
    assert!(vienna.iter().all(|p| p.id != "khm_caravaggio_rosenkranz"));

    let tokyo = places_in_city(TravelCity::Tokyo);
    assert!(tokyo.iter().any(|p| p.id == "jp_nihonbashi"));
    assert!(tokyo.iter().any(|p| p.id == "tnm_hokusai_wave"));
    assert!(tokyo.iter().all(|p| p.id != "jp_fushimi"));
    assert!(tokyo.iter().all(|p| p.id != "jp_hakone"));
}

#[test]
fn seed_puts_city_stops_on_the_board_once() {
    let engine = TravelCraftEngine::new_in_memory().expect("engine");
    let inserted = engine.kanban_seed_cities().expect("seed");
    assert!(inserted >= 8);
    let cards = engine.kanban_list().expect("list");
    assert_eq!(cards.len(), inserted);
    let stephans = cards
        .iter()
        .find(|c| c.title.contains("Stephansdom"))
        .expect("stephansdom card");
    assert_eq!(stephans.city, "vienna");
    assert_eq!(stephans.column_id, "this_trip");
    assert!(stephans.lat.is_some());
    let nihon = cards
        .iter()
        .find(|c| c.title.contains("Nihonbashi"))
        .expect("nihonbashi card");
    assert_eq!(nihon.city, "tokyo");
    assert_eq!(nihon.column_id, "this_trip");
    assert!(cards.iter().all(|c| !c.title.contains("Hallstatt")));
    assert_eq!(engine.kanban_seed_cities().expect("second seed"), 0);
    assert_eq!(engine.kanban_list().expect("list again").len(), inserted);
}

#[test]
fn add_move_and_delete_round_trip() {
    let engine = TravelCraftEngine::new_in_memory().expect("engine");
    let err = engine
        .kanban_add("  ", "vienna", "")
        .expect_err("blank title");
    assert_eq!(err.kind(), "invalid");
    let err = engine
        .kanban_add("Night bus", "osaka", "")
        .expect_err("bad city");
    assert_eq!(err.kind(), "invalid");
    let card = engine
        .kanban_add("Ueno morning", "tokyo", "Before the museums open")
        .expect("add");
    assert_eq!(card.column_id, "backlog");
    assert_eq!(card.city, "tokyo");
    engine.kanban_move(&card.card_id, "booked").expect("move");
    let listed = engine.kanban_list().expect("list");
    let found = listed
        .iter()
        .find(|c| c.card_id == card.card_id)
        .expect("card");
    assert_eq!(found.column_id, "booked");
    let missing = engine
        .kanban_move("no-such-card", "done")
        .expect_err("missing");
    assert_eq!(missing.kind(), "not_found");
    let bad_col = engine
        .kanban_move(&card.card_id, "later")
        .expect_err("bad column");
    assert_eq!(bad_col.kind(), "invalid");
    engine.kanban_delete(&card.card_id).expect("delete");
    assert!(engine
        .kanban_list()
        .expect("list")
        .iter()
        .all(|c| c.card_id != card.card_id));
}

#[test]
fn moved_card_survives_reopen() {
    let path = temp_db_path();
    cleanup(&path);
    let card_id = {
        let engine = TravelCraftEngine::open(&path).expect("open");
        engine.kanban_seed_cities().expect("seed");
        let cards = engine.kanban_list().expect("list");
        let card = cards
            .iter()
            .find(|c| c.title.contains("Stephansdom"))
            .expect("card");
        engine.kanban_move(&card.card_id, "done").expect("move");
        card.card_id.clone()
    };
    let engine = TravelCraftEngine::open(&path).expect("reopen");
    let cards = engine.kanban_list().expect("list");
    let card = cards.iter().find(|c| c.card_id == card_id).expect("kept");
    assert_eq!(card.column_id, "done");
    assert_eq!(engine.kanban_seed_cities().expect("no reseed"), 0);
    drop(engine);
    cleanup(&path);
}

fn temp_db_path() -> PathBuf {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("travelcraft-kanban-{}-{ms}.db", std::process::id()))
}

fn cleanup(path: &std::path::Path) {
    let text = path.to_string_lossy().to_string();
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{text}-wal"));
    let _ = std::fs::remove_file(format!("{text}-shm"));
}
