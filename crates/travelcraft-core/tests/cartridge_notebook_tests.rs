use travelcraft_baedeker::cartridge::{
    italy_country_cartridge, venice_city_cartridge, Cartridge, CartridgeKind, CurrentEvent,
    EventCategory, Gotcha, LocalSpot, MediaAsset, PriceTier, SpotCategory, Zone,
};
use travelcraft_core::cartridges::{CartridgeFilter, CartridgeItem, CartridgeMetaPatch};
use travelcraft_core::notebook::{
    NewNotebookEntry, NewTrip, NotebookKind, NotebookPatch, NotebookQuery, TripPatch, TripStatus,
};
use travelcraft_core::TravelCraftEngine;

#[test]
fn venice_seed_and_country_city_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    let engine = TravelCraftEngine::new_in_memory()?;
    let seeded = engine.cartridge_seed_venice()?;
    assert_eq!(seeded.len(), 2);

    let resolved = engine.cartridge_resolve("it-venice")?;
    assert_eq!(resolved.parent_id.as_deref(), Some("it"));
    assert_eq!(resolved.parent_title.as_deref(), Some("Italy"));
    // Inherited from Italy country cartridge:
    assert_eq!(resolved.effective_currency, "EUR");
    assert_eq!(resolved.effective_timezone, "Europe/Rome");
    assert_eq!(resolved.effective_languages, vec!["it".to_string()]);
    assert!(resolved
        .effective_survival_kit
        .emergency_number
        .contains("112"));
    // Overridden by Venice city cartridge:
    assert!(resolved
        .effective_survival_kit
        .transit_basics
        .contains("ACTV"));
    // Merged phrases, gotchas, and knowledge branches from both IT + Venice:
    assert!(resolved.effective_phrases.len() >= 4);
    assert!(resolved.effective_gotchas.len() >= 5);
    assert!(resolved.effective_knowledge_branches.len() >= 2);
    // 7-day "A Week in Venice" itinerary present:
    assert!(resolved
        .cartridge
        .itineraries
        .iter()
        .any(|i| i.total_days == 7 && i.days.len() == 7));
    Ok(())
}

#[test]
fn cartridge_crud_delete_guard_and_json_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let engine = TravelCraftEngine::new_in_memory()?;
    let it = italy_country_cartridge();
    let ve = venice_city_cartridge();

    // City cannot be created before its parent country exists.
    assert!(engine.cartridge_create(&ve).is_err());

    engine.cartridge_create(&it)?;
    // Duplicate create rejected.
    assert!(engine.cartridge_create(&it).is_err());
    engine.cartridge_create(&ve)?;

    // Filtered list.
    let cities = engine.cartridge_list(&CartridgeFilter {
        kind: Some(CartridgeKind::City),
        country_code: Some("IT".to_string()),
        parent_id: Some("it".to_string()),
    })?;
    assert_eq!(cities.len(), 1);
    assert_eq!(cities[0].id, "it-venice");

    // Meta update.
    let updated = engine.cartridge_update_meta(
        "it-venice",
        &CartridgeMetaPatch {
            version: Some("1.1.0".to_string()),
            ..CartridgeMetaPatch::default()
        },
    )?;
    assert_eq!(updated.version, "1.1.0");

    // Export -> delete -> import round-trip.
    let exported = engine.cartridge_export_json("it-venice")?;
    // Country delete blocked while child exists.
    assert!(engine.cartridge_delete("it", false).is_err());
    let deleted = engine.cartridge_delete("it", true)?;
    assert_eq!(deleted, 2);
    assert!(engine.cartridge_get("it-venice").is_err());

    engine.cartridge_upsert(&it)?;
    let reimported = engine.cartridge_import_json(&exported)?;
    assert_eq!(reimported.id, "it-venice");
    assert_eq!(reimported.version, "1.1.0");
    Ok(())
}

#[test]
fn cartridge_item_level_crud() -> Result<(), Box<dyn std::error::Error>> {
    let engine = TravelCraftEngine::new_in_memory()?;
    engine.cartridge_seed_venice()?;

    let before = engine.cartridge_get("it-venice")?;
    let spot_count = before.spots.len();
    let event_count = before.events.len();
    let media_count = before.media.len();

    let new_spot = LocalSpot {
        id: "spot-all-arco".to_string(),
        name: "Osteria All'Arco".to_string(),
        category: SpotCategory::Bacaro,
        tier: Some(PriceTier::Budget),
        zone_id: "san-polo".to_string(),
        address: "Calle Arco 436, San Polo".to_string(),
        lat: Some(45.4381),
        lon: Some(12.3346),
        price_band: "EUR".to_string(),
        booking_note: "Walk-in only; closed Wednesdays.".to_string(),
        note: "Tiny counter, crostini topped to order.".to_string(),
        tags: vec!["bacaro".to_string(), "rialto".to_string()],
        photo_uris: Vec::new(),
        source_urls: vec!["https://example.org/all-arco".to_string()],
        verified_at: Some("2026-11-20".to_string()),
    };
    let s1 = engine.cartridge_item_upsert("it-venice", CartridgeItem::Spot(new_spot))?;
    assert_eq!(s1.spot_count, spot_count + 1);

    let ev = CurrentEvent {
        id: "ev-biennale-2026".to_string(),
        title: "Venice Biennale Art Exhibition".to_string(),
        venue: "Giardini & Arsenale".to_string(),
        category: EventCategory::Exhibition,
        start_date: Some("2026-05-09".to_string()),
        end_date: Some("2026-11-22".to_string()),
        booking_url: "https://www.labiennale.org".to_string(),
        note: "Allow two separate half-days for Giardini and Arsenale.".to_string(),
        source_url: "https://www.labiennale.org".to_string(),
        verified_at: Some("2026-11-20".to_string()),
    };
    let s_ev = engine.cartridge_item_upsert("it-venice", CartridgeItem::Event(ev))?;
    assert_eq!(s_ev.event_count, event_count + 1);

    let media = MediaAsset {
        id: "media-san-rocco".to_string(),
        subject_id: "ve-san-rocco".to_string(),
        url: "https://upload.wikimedia.org/wikipedia/commons/example.jpg".to_string(),
        local_path: "media/san-rocco.webp".to_string(),
        caption: "Scuola Grande di San Rocco facade".to_string(),
        license: "CC-BY-SA-4.0".to_string(),
        attribution: "Wikimedia Commons".to_string(),
    };
    let s_m = engine.cartridge_item_upsert("it-venice", CartridgeItem::Media(media))?;
    assert_eq!(s_m.media_count, media_count + 1);

    let s_ev_rm = engine.cartridge_item_remove("it-venice", "event", "ev-biennale-2026")?;
    assert_eq!(s_ev_rm.event_count, event_count);
    let s_m_rm = engine.cartridge_item_remove("it-venice", "media", "media-san-rocco")?;
    assert_eq!(s_m_rm.media_count, media_count);

    let new_gotcha = Gotcha {
        id: "ve-water-taxis".to_string(),
        title: "Private water taxi pricing".to_string(),
        season: "year-round".to_string(),
        advice: "Agree fare before boarding; airport to center is ~EUR 140+.".to_string(),
    };
    engine.cartridge_item_upsert("it-venice", CartridgeItem::Gotcha(new_gotcha))?;

    let new_zone = Zone {
        id: "giudecca".to_string(),
        name: "Giudecca".to_string(),
        summary: "Quiet southern island facing Zattere; Palladio's Redentore.".to_string(),
        lat: Some(45.4255),
        lon: Some(12.3264),
    };
    engine.cartridge_item_upsert("it-venice", CartridgeItem::Zone(new_zone))?;

    let s2 = engine.cartridge_item_remove("it-venice", "spot", "spot-all-arco")?;
    assert_eq!(s2.spot_count, spot_count);
    assert!(engine
        .cartridge_item_remove("it-venice", "spot", "nonexistent")
        .is_err());
    Ok(())
}

#[test]
fn cartridge_validation_rejects_bad_slug_and_coords() {
    let mut c = italy_country_cartridge();
    c.id = "BAD SLUG!".to_string();
    assert!(c.validate().is_err());

    let mut c2 = italy_country_cartridge();
    c2.parent_id = Some("eu".to_string());
    assert!(c2.validate().is_err());

    let mut c3: Cartridge = venice_city_cartridge();
    c3.country_code = "ITALY".to_string();
    assert!(c3.validate().is_err());
}

#[test]
fn universal_notebook_week_in_venice_cross_boundary_and_export(
) -> Result<(), Box<dyn std::error::Error>> {
    let engine = TravelCraftEngine::new_in_memory()?;
    engine.cartridge_seed_venice()?;

    let trip = engine.trip_create(&NewTrip {
        trip_id: Some("venice-2026-11".to_string()),
        title: "A Week in Venice".to_string(),
        start_date: Some("2026-11-08".to_string()),
        end_date: Some("2026-11-15".to_string()),
        cartridge_ids: vec!["it".to_string(), "it-venice".to_string()],
        status: TripStatus::Completed,
        notes: "Railjet from Wien Hbf, six sestieri, Padua day trip.".to_string(),
    })?;
    assert_eq!(trip.trip_id, "venice-2026-11");

    // Leg 1: departing Vienna (cross-country entry in the same trip, no waypoint required).
    let e1 = engine.notebook_add(&NewNotebookEntry {
        trip_id: Some(trip.trip_id.clone()),
        cartridge_id: None,
        country_code: Some("AT".to_string()),
        waypoint_id: None,
        kind: NotebookKind::Logistics,
        day_index: Some(1),
        title: "Wien Hbf -> Venezia Santa Lucia".to_string(),
        body: "Morning Railjet through Semmering and Tarvisio; coffee in the dining car."
            .to_string(),
        sensory_notes: "Rain on compartment glass, warm espresso".to_string(),
        photo_uris: Vec::new(),
        lat: Some(48.1851),
        lon: Some(16.3765),
        tags: vec!["rail".to_string(), "transit".to_string()],
    })?;

    // Leg 2: Venice Tintoretto entry.
    let e2 = engine.notebook_add(&NewNotebookEntry {
        trip_id: Some(trip.trip_id.clone()),
        cartridge_id: Some("it-venice".to_string()),
        country_code: Some("IT".to_string()),
        waypoint_id: Some("ve-san-rocco".to_string()),
        kind: NotebookKind::Reflection,
        day_index: Some(3),
        title: "Tintoretto at San Rocco".to_string(),
        body: "Sat for forty minutes in the Sala dell'Albergo in front of the Crucifixion."
            .to_string(),
        sensory_notes: "Cool marble, tilted mirrors, hushed room".to_string(),
        photo_uris: vec!["immich://san-rocco-01.jpg".to_string()],
        lat: Some(45.4365),
        lon: Some(12.3252),
        tags: vec!["tintoretto".to_string(), "art".to_string()],
    })?;

    // Leg 3: Venice bacaro food note.
    let _e3 = engine.notebook_add(&NewNotebookEntry {
        trip_id: Some(trip.trip_id.clone()),
        cartridge_id: Some("it-venice".to_string()),
        country_code: Some("IT".to_string()),
        waypoint_id: None,
        kind: NotebookKind::Food,
        day_index: Some(3),
        title: "Cantina Do Mori".to_string(),
        body: "Two francobolli tramezzini and an ombra of Soave standing under copper pots."
            .to_string(),
        sensory_notes: "Vinegar, salt cod, loud Venetian banter".to_string(),
        photo_uris: Vec::new(),
        lat: Some(45.4383),
        lon: Some(12.3340),
        tags: vec!["bacaro".to_string(), "cicchetti".to_string()],
    })?;

    // Scoped views: This Trip (3) | Venice Cartridge (2) | Country AT (1) | Search "Tintoretto" (1).
    let by_trip = engine.notebook_query(&NotebookQuery {
        trip_id: Some("venice-2026-11".to_string()),
        ..NotebookQuery::default()
    })?;
    assert_eq!(by_trip.len(), 3);

    let by_venice = engine.notebook_query(&NotebookQuery {
        cartridge_id: Some("it-venice".to_string()),
        ..NotebookQuery::default()
    })?;
    assert_eq!(by_venice.len(), 2);

    let by_at = engine.notebook_query(&NotebookQuery {
        country_code: Some("at".to_string()),
        ..NotebookQuery::default()
    })?;
    assert_eq!(by_at.len(), 1);
    assert_eq!(by_at[0].entry_id, e1.entry_id);

    let by_search = engine.notebook_query(&NotebookQuery {
        search: Some("crucifixion".to_string()),
        ..NotebookQuery::default()
    })?;
    assert_eq!(by_search.len(), 1);
    assert_eq!(by_search[0].entry_id, e2.entry_id);

    // Update entry + trip.
    let updated_entry = engine.notebook_update(
        &e2.entry_id,
        &NotebookPatch {
            tags: Some(vec!["tintoretto".to_string(), "highlight".to_string()]),
            ..NotebookPatch::default()
        },
    )?;
    assert_eq!(updated_entry.tags, vec!["tintoretto", "highlight"]);

    let updated_trip = engine.trip_update(
        "venice-2026-11",
        &TripPatch {
            title: Some("A Week in Venice (Nov 2026)".to_string()),
            ..TripPatch::default()
        },
    )?;
    assert_eq!(updated_trip.title, "A Week in Venice (Nov 2026)");

    // Markdown travelogue export.
    let md = engine.trip_export_markdown("venice-2026-11")?;
    assert!(md.contains("# A Week in Venice (Nov 2026)"));
    assert!(md.contains("## Day 1"));
    assert!(md.contains("## Day 3"));
    assert!(md.contains("Tintoretto at San Rocco"));
    assert!(md.contains("Cantina Do Mori"));

    // Deleting the trip with delete_entries=false preserves the universal entries.
    engine.trip_delete("venice-2026-11", false)?;
    let still_there = engine.notebook_query(&NotebookQuery::default())?;
    assert_eq!(still_there.len(), 3);
    assert!(still_there.iter().all(|e| e.trip_id.is_none()));
    Ok(())
}

#[test]
fn cartridges_and_notebook_survive_reopen() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join(format!(
        "travelcraft_cartridge_test_{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    ));
    let db = dir.join("travelcraft.db");
    {
        let engine = TravelCraftEngine::open(&db)?;
        engine.cartridge_seed_venice()?;
        let t = engine.trip_create(&NewTrip {
            trip_id: Some("reopen-trip".to_string()),
            title: "Venice Reopen".to_string(),
            start_date: None,
            end_date: None,
            cartridge_ids: vec!["it-venice".to_string()],
            status: TripStatus::Active,
            notes: String::new(),
        })?;
        engine.notebook_add(&NewNotebookEntry {
            trip_id: Some(t.trip_id),
            cartridge_id: Some("it-venice".to_string()),
            country_code: Some("IT".to_string()),
            waypoint_id: None,
            kind: NotebookKind::Idea,
            day_index: Some(1),
            title: "Early San Marco".to_string(),
            body: "Go before 08:30.".to_string(),
            sensory_notes: String::new(),
            photo_uris: Vec::new(),
            lat: None,
            lon: None,
            tags: vec!["plan".to_string()],
        })?;
    }
    {
        let engine = TravelCraftEngine::open(&db)?;
        let resolved = engine.cartridge_resolve("it-venice")?;
        assert_eq!(resolved.effective_currency, "EUR");
        let entries = engine.notebook_query(&NotebookQuery {
            trip_id: Some("reopen-trip".to_string()),
            ..NotebookQuery::default()
        })?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Early San Marco");
    }
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[test]
fn venice_grounded_tiers_events_and_media() -> Result<(), Box<dyn std::error::Error>> {
    let engine = TravelCraftEngine::new_in_memory()?;
    engine.cartridge_seed_venice()?;
    let ve = engine.cartridge_get("it-venice")?;

    // Tiered stays & dining cover all 3 tiers (Budget, Midrange, Splurge).
    assert!(ve.spots.iter().any(|s| s.tier == Some(PriceTier::Budget)));
    assert!(ve.spots.iter().any(|s| s.tier == Some(PriceTier::Midrange)));
    assert!(ve.spots.iter().any(|s| s.tier == Some(PriceTier::Splurge)));
    assert!(ve.spots.iter().any(|s| s.category == SpotCategory::Stay));
    // Nov 2026 grounded events & Wikimedia media present.
    assert_eq!(ve.events_refreshed_at.as_deref(), Some("2026-11-20"));
    assert!(ve.events.len() >= 3);
    assert!(ve.media.len() >= 3);
    Ok(())
}

#[test]
fn japan_model_country_and_kyoto_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    let engine = TravelCraftEngine::new_in_memory()?;
    let seeded = engine.cartridge_seed_japan()?;
    assert_eq!(seeded.len(), 2);

    let resolved = engine.cartridge_resolve("jp-kyoto")?;
    assert_eq!(resolved.parent_id.as_deref(), Some("jp"));
    assert_eq!(resolved.effective_currency, "JPY");
    assert_eq!(resolved.effective_timezone, "Asia/Tokyo");
    assert_eq!(resolved.effective_languages, vec!["ja".to_string()]);
    assert!(resolved
        .effective_survival_kit
        .emergency_number
        .contains("110"));
    // Inherits the migrated JapanCraft knowledge tree branches.
    assert!(!resolved.effective_knowledge_branches.is_empty());
    // Kyoto tiered stays & dining + Nov 2026 koyo night illuminations.
    assert!(resolved
        .cartridge
        .spots
        .iter()
        .any(|s| s.category == SpotCategory::Stay && s.tier == Some(PriceTier::Splurge)));
    assert!(resolved
        .cartridge
        .events
        .iter()
        .any(|e| e.id == "ev-ky-kiyomizu-autumn-2026"));
    Ok(())
}

#[test]
fn giscraft_cartridge_tile_plan_and_map_places() -> Result<(), Box<dyn std::error::Error>> {
    let engine = TravelCraftEngine::new_in_memory()?;
    engine.cartridge_seed_venice()?;
    engine.cartridge_seed_japan()?;

    let plan = engine.cartridge_tile_plan("it-venice", 12, 14)?;
    assert_eq!(plan.cartridge_id, "it-venice");
    assert_eq!(plan.per_zoom.len(), 3);
    assert!(plan.total_tiles > 10);
    assert!(plan.estimated_mb > 0.0);

    let ky = engine.cartridge_get("jp-kyoto")?;
    let places = travelcraft_core::cities::places_in_cartridge(&ky);
    assert!(places.iter().any(|p| p.kind == "stop"));
    assert!(places.iter().any(|p| p.kind == "spot"));
    Ok(())
}

#[test]
fn lenscraft_identify_crossmatches_venice_artwork_and_logs_notebook(
) -> Result<(), Box<dyn std::error::Error>> {
    use lenscraft_core::{LensError, PreparedImage, Verdict, VisionTransport};
    use serde_json::{json, Value};
    use travelcraft_core::lens::LensIdentifyRequest;

    struct FakeVision;
    impl VisionTransport for FakeVision {
        fn base_url(&self) -> String {
            "http://fake.local".to_string()
        }
        fn show(&self, _model: &str) -> Result<Value, LensError> {
            Ok(json!({ "capabilities": ["vision"] }))
        }
        fn chat(&self, _body: &Value) -> Result<Value, LensError> {
            let answer = json!({
                "label_text": "Jacopo Tintoretto - The Crucifixion, 1565",
                "description": "Large dramatic canvas of the Crucifixion in Sala dell'Albergo.",
                "candidates": [{
                    "title": "The Crucifixion",
                    "creator": "Jacopo Tintoretto",
                    "date": "1565",
                    "certainty": "high",
                    "evidence": ["Wall label names Jacopo Tintoretto and The Crucifixion"]
                }]
            });
            Ok(json!({ "message": { "content": answer.to_string() } }))
        }
        fn version(&self) -> Result<String, LensError> {
            Ok("0.9.0-fake".to_string())
        }
    }

    let engine = TravelCraftEngine::new_in_memory()?;
    engine.cartridge_seed_venice()?;
    let trip = engine.trip_create(&NewTrip {
        trip_id: Some("venice-lens-trip".to_string()),
        title: "Venice Lens Test".to_string(),
        start_date: None,
        end_date: None,
        cartridge_ids: vec!["it-venice".to_string()],
        status: TripStatus::Active,
        notes: String::new(),
    })?;

    let prepared = PreparedImage {
        jpeg: vec![0xFF, 0xD8, 0xFF, 0xD9],
        width: 800,
        height: 600,
        source_width: 800,
        source_height: 600,
        source_format: "jpeg".to_string(),
    };
    let req = LensIdentifyRequest {
        photo_path: "immich://tintoretto.jpg".to_string(),
        cartridge_id: Some("it-venice".to_string()),
        place: Some("Scuola Grande di San Rocco".to_string()),
        hint: None,
        lat: Some(45.4365),
        lon: Some(12.3252),
        language: Some("en".to_string()),
        model: None,
        log_trip_id: Some(trip.trip_id.clone()),
    };

    let res = engine.lens_identify_with(&FakeVision, &prepared, &req)?;
    assert_eq!(res.lens.verdict, Verdict::Identified);
    let art = res
        .matched_artwork
        .as_ref()
        .ok_or("expected matched Tintoretto artwork")?;
    assert_eq!(art.id, "art-tintoretto-crucifixion");
    let entry_id = res
        .notebook_entry_id
        .as_deref()
        .ok_or("expected notebook entry id")?;
    let logged = engine.notebook_get(entry_id)?;
    assert_eq!(logged.trip_id.as_deref(), Some("venice-lens-trip"));
    assert_eq!(logged.cartridge_id.as_deref(), Some("it-venice"));
    assert!(logged.body.contains("Scuola Grande di San Rocco"));
    Ok(())
}

#[test]
fn seed_all_installs_all_five_countries_and_five_cities() -> Result<(), Box<dyn std::error::Error>>
{
    let engine = TravelCraftEngine::new_in_memory()?;
    let seeded = engine.cartridge_seed_all()?;
    assert_eq!(seeded.len(), 10);

    for (city_id, expected_currency, expected_parent) in [
        ("it-venice", "EUR", "it"),
        ("jp-kyoto", "JPY", "jp"),
        ("at-salzburg", "EUR", "at"),
        ("gb-london", "GBP", "gb"),
        ("fr-paris", "EUR", "fr"),
    ] {
        let r = engine.cartridge_resolve(city_id)?;
        assert_eq!(r.effective_currency, expected_currency);
        assert_eq!(r.parent_id.as_deref(), Some(expected_parent));
        assert!(!r.cartridge.events.is_empty(), "{city_id} missing events");
        assert!(!r.cartridge.spots.is_empty(), "{city_id} missing spots");
        let plan = engine.cartridge_tile_plan(city_id, 12, 14)?;
        assert!(plan.total_tiles > 0, "{city_id} tile plan empty");
    }
    Ok(())
}

#[test]
fn translate_offline_phrasebook_and_companion_ollama() -> Result<(), Box<dyn std::error::Error>> {
    use travelcraft_core::translate::TranslateTransport;

    struct FakeTranslate;
    impl TranslateTransport for FakeTranslate {
        fn chat(
            &self,
            _model: &str,
            _system: &str,
            _user: &str,
        ) -> Result<String, travelcraft_core::TravelCraftError> {
            Ok(r#"{"translated":"Dove e la fermata del vaporetto?","pronunciation":"DOH-veh eh la fer-MAH-ta del va-po-RET-to"}"#.to_string())
        }
    }

    let engine = TravelCraftEngine::new_in_memory()?;
    engine.cartridge_seed_all()?;

    // Offline phrasebook resolves parent country + city phrases.
    let pb = engine.translate_phrases("fr-paris", Some("water"))?;
    assert_eq!(pb.total, 1);
    assert!(pb.phrases[0].local.contains("carafe d'eau"));

    // Exact phrasebook hit answers offline without hitting Ollama.
    let hit = engine.translate_text("The bill, please", "Italian", Some("it-venice"), None)?;
    assert_eq!(hit.source, "offline_phrasebook");
    assert_eq!(hit.translated, "Il conto, per favore");

    // Arbitrary phrase goes through companion Ollama transport.
    let live = engine.translate_text_with(
        &FakeTranslate,
        "Where is the vaporetto stop?",
        "Italian",
        None,
    )?;
    assert_eq!(live.source, "ollama");
    assert!(live.translated.contains("vaporetto"));
    assert!(!live.pronunciation.is_empty());
    Ok(())
}
