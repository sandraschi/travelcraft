//! egui views for Country/City Cartridges, Offline Phrasebook/Translate, and Universal Notebook.

use travelcraft_baedeker::cartridge::{CartridgeKind, PriceTier, ResolvedCartridge, SpotCategory};
use travelcraft_core::cartridges::{CartridgeFilter, CartridgeTilePlan};
use travelcraft_core::notebook::{
    NewNotebookEntry, NewTrip, NotebookKind, NotebookQuery, TripStatus,
};
use travelcraft_core::TravelCraftEngine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotebookScope {
    Trip,
    Cartridge,
    Country,
    All,
}

pub struct CartridgeStudioState {
    pub selected_cartridge_id: String,
    pub resolved: Option<ResolvedCartridge>,
    pub tile_plan: Option<CartridgeTilePlan>,
    pub status: String,

    // Phrasebook & Translate
    pub phrase_query: String,
    pub translate_input: String,
    pub translate_target: String,
    pub translate_output: String,
    pub flashcard_text: String,

    // Universal Notebook
    pub scope: NotebookScope,
    pub selected_trip_id: String,
    pub new_trip_title: String,
    pub note_kind: NotebookKind,
    pub note_day: String,
    pub note_title: String,
    pub note_body: String,
    pub note_tags: String,
    pub note_search: String,
    pub exported_markdown: String,
}

impl Default for CartridgeStudioState {
    fn default() -> Self {
        Self {
            selected_cartridge_id: "it-venice".to_string(),
            resolved: None,
            tile_plan: None,
            status: String::new(),
            phrase_query: String::new(),
            translate_input: String::new(),
            translate_target: "Italian".to_string(),
            translate_output: String::new(),
            flashcard_text: String::new(),
            scope: NotebookScope::All,
            selected_trip_id: String::new(),
            new_trip_title: String::new(),
            note_kind: NotebookKind::Reflection,
            note_day: "1".to_string(),
            note_title: String::new(),
            note_body: String::new(),
            note_tags: String::new(),
            note_search: String::new(),
            exported_markdown: String::new(),
        }
    }
}

pub fn ui_cartridges_tab(
    ui: &mut egui::Ui,
    engine: &TravelCraftEngine,
    state: &mut CartridgeStudioState,
) {
    ui.horizontal_wrapped(|ui| {
        ui.heading("Cartridges (Country -> City)");
        ui.separator();
        if ui
            .button("Seed All 5 Countries + 5 Cities (Venice, Kyoto, London, Paris, Salzburg)")
            .clicked()
        {
            match engine.cartridge_seed_all() {
                Ok(list) => {
                    state.status = format!("Seeded {} cartridges.", list.len());
                    state.resolved = engine.cartridge_resolve(&state.selected_cartridge_id).ok();
                }
                Err(e) => state.status = format!("Seed failed: {e}"),
            }
        }
        if !state.status.is_empty() {
            ui.label(&state.status);
        }
    });
    ui.separator();

    let summaries = engine
        .cartridge_list(&CartridgeFilter::default())
        .unwrap_or_default();

    ui.columns(2, |cols| {
        // Left column: hierarchical Country -> indented City cartridge roster
        cols[0].heading(format!("Installed Cartridges ({})", summaries.len()));
        egui::ScrollArea::vertical()
            .id_salt("cartridge_list_scroll")
            .max_height(520.0)
            .show(&mut cols[0], |ui| {
                if summaries.is_empty() {
                    ui.label("No cartridges installed yet. Click 'Seed All' above to load Italy/Venice, Japan/Kyoto, Austria/Salzburg, UK/London, and France/Paris.");
                }
                let countries: Vec<_> = summaries
                    .iter()
                    .filter(|c| c.kind == CartridgeKind::Country)
                    .collect();
                let cities: Vec<_> = summaries
                    .iter()
                    .filter(|c| c.kind == CartridgeKind::City)
                    .collect();

                for country in &countries {
                    let child_cities: Vec<_> = cities
                        .iter()
                        .filter(|city| city.parent_id.as_deref() == Some(country.id.as_str()))
                        .collect();
                    let selected = state.selected_cartridge_id == country.id;
                    let label = format!(
                        "[COUNTRY] {} ({}) - {} stops, {} events, {} cities",
                        country.title,
                        country.id,
                        country.waypoint_count,
                        country.event_count,
                        child_cities.len()
                    );
                    if ui.selectable_label(selected, label).clicked() {
                        state.selected_cartridge_id = country.id.clone();
                        state.tile_plan = None;
                        state.flashcard_text.clear();
                        match engine.cartridge_resolve(&country.id) {
                            Ok(r) => {
                                state.resolved = Some(r);
                                state.status = format!("Loaded {}", country.id);
                            }
                            Err(e) => state.status = format!("Error loading {}: {e}", country.id),
                        }
                    }
                    for city in child_cities {
                        let city_selected = state.selected_cartridge_id == city.id;
                        let city_label = format!(
                            "   |_ [CITY] {} ({}) - {} stops, {} spots, {} events",
                            city.title,
                            city.id,
                            city.waypoint_count,
                            city.spot_count,
                            city.event_count
                        );
                        if ui.selectable_label(city_selected, city_label).clicked() {
                            state.selected_cartridge_id = city.id.clone();
                            state.tile_plan = None;
                            state.flashcard_text.clear();
                            match engine.cartridge_resolve(&city.id) {
                                Ok(r) => {
                                    state.resolved = Some(r);
                                    state.status = format!("Loaded {}", city.id);
                                }
                                Err(e) => state.status = format!("Error loading {}: {e}", city.id),
                            }
                        }
                    }
                }
                // Orphan cities (without a matching parent country)
                for city in &cities {
                    let has_parent = countries
                        .iter()
                        .any(|co| city.parent_id.as_deref() == Some(co.id.as_str()));
                    if has_parent {
                        continue;
                    }
                    let selected = state.selected_cartridge_id == city.id;
                    let label = format!(
                        "[CITY] {} ({}) - {} stops, {} spots, {} events",
                        city.title, city.id, city.waypoint_count, city.spot_count, city.event_count
                    );
                    if ui.selectable_label(selected, label).clicked() {
                        state.selected_cartridge_id = city.id.clone();
                        state.tile_plan = None;
                        state.flashcard_text.clear();
                        state.resolved = engine.cartridge_resolve(&city.id).ok();
                    }
                }
            });

        // Right column: resolved cartridge inspector + giscraft tile plan + phrasebook/translate
        if state.resolved.is_none() && !summaries.is_empty() {
            state.resolved = engine.cartridge_resolve(&state.selected_cartridge_id).ok();
        }
        egui::ScrollArea::vertical()
            .id_salt("cartridge_detail_scroll")
            .max_height(560.0)
            .show(&mut cols[1], |ui| {
                let Some(resolved) = state.resolved.clone() else {
                    ui.label("Select a cartridge on the left.");
                    return;
                };
                let c = &resolved.cartridge;
                ui.heading(format!("{} ({})", c.title, c.id));
                if !c.subtitle.is_empty() {
                    ui.label(&c.subtitle);
                }
                ui.label(format!(
                    "Currency: {} | Timezone: {} | Parent: {}",
                    resolved.effective_currency,
                    resolved.effective_timezone,
                    resolved.parent_id.as_deref().unwrap_or("none")
                ));
                if c.kind == CartridgeKind::Country {
                    let child_cities: Vec<_> = summaries
                        .iter()
                        .filter(|s| s.parent_id.as_deref() == Some(c.id.as_str()))
                        .collect();
                    if !child_cities.is_empty() {
                        ui.horizontal_wrapped(|ui| {
                            ui.strong("Child cities:");
                            for child in child_cities {
                                if ui.button(format!("Open {} ({})", child.title, child.id)).clicked() {
                                    state.selected_cartridge_id = child.id.clone();
                                    state.tile_plan = None;
                                    state.flashcard_text.clear();
                                    state.resolved = engine.cartridge_resolve(&child.id).ok();
                                    state.status = format!("Opened {}", child.id);
                                }
                            }
                        });
                    }
                }
                ui.horizontal(|ui| {
                    if ui.button("Plan Offline GISCraft Tiles (Z12-Z15)").clicked() {
                        match engine.cartridge_tile_plan(&c.id, 12, 15) {
                            Ok(plan) => state.tile_plan = Some(plan),
                            Err(e) => state.status = format!("Tile plan error: {e}"),
                        }
                    }
                });
                if let Some(plan) = &state.tile_plan {
                    ui.label(format!(
                        "GISCraft Tile Plan Z{}-Z{}: {} tiles (~{:.2} MB)",
                        plan.min_zoom, plan.max_zoom, plan.total_tiles, plan.estimated_mb
                    ));
                }

                ui.separator();
                ui.collapsing("Survival Kit (Inherited + Local)", |ui| {
                    let sk = &resolved.effective_survival_kit;
                    ui.label(format!("Emergency: {}", sk.emergency_number));
                    ui.label(format!("Plugs: {}", sk.power_plugs));
                    ui.label(format!("Water: {}", sk.tap_water));
                    ui.label(format!("Tipping/Cover: {}", sk.tipping_and_cover));
                    ui.label(format!("Transit: {}", sk.transit_basics));
                });

                if !c.transit.summary.is_empty() || !c.transit.lines.is_empty() {
                    ui.collapsing("Transit & Rail Snapshot", |ui| {
                        if !c.transit.summary.is_empty() {
                            ui.label(&c.transit.summary);
                        }
                        for line in &c.transit.lines {
                            ui.label(format!("* {line}"));
                        }
                        for pass in &c.transit.passes {
                            ui.label(format!("Pass: {pass}"));
                        }
                    });
                }

                if !resolved.effective_gotchas.is_empty() {
                    ui.collapsing(format!("Practical Gotchas ({})", resolved.effective_gotchas.len()), |ui| {
                        for g in &resolved.effective_gotchas {
                            ui.strong(format!("{} [{}]", g.title, g.season));
                            ui.label(&g.advice);
                            ui.separator();
                        }
                    });
                }

                if !resolved.effective_knowledge_branches.is_empty() {
                    ui.collapsing(
                        format!("Knowledge Tree ({} branches)", resolved.effective_knowledge_branches.len()),
                        |ui| {
                            for b in &resolved.effective_knowledge_branches {
                                ui.strong(&b.title);
                                for item in &b.items {
                                    ui.label(format!("* {}: {}", item.title, item.subtitle));
                                }
                            }
                        },
                    );
                }

                if !c.events.is_empty() {
                    let refreshed = c.events_refreshed_at.as_deref().unwrap_or("unknown");
                    ui.collapsing(
                        format!("Current Events ({}, refreshed {refreshed})", c.events.len()),
                        |ui| {
                            for ev in &c.events {
                                ui.strong(&ev.title);
                                ui.label(format!(
                                    "{} ({} to {})",
                                    ev.venue,
                                    ev.start_date.as_deref().unwrap_or("ongoing"),
                                    ev.end_date.as_deref().unwrap_or("ongoing")
                                ));
                                ui.label(&ev.note);
                                ui.separator();
                            }
                        },
                    );
                }

                if !c.spots.is_empty() {
                    ui.collapsing(format!("Tiered Stays & Dining ({})", c.spots.len()), |ui| {
                        for (tier_label, tier_val) in [
                            ("Budget", PriceTier::Budget),
                            ("Midrange", PriceTier::Midrange),
                            ("Splurge", PriceTier::Splurge),
                        ] {
                            let matches: Vec<_> = c
                                .spots
                                .iter()
                                .filter(|s| s.tier == Some(tier_val))
                                .collect();
                            if matches.is_empty() {
                                continue;
                            }
                            ui.strong(format!("-- {tier_label} --"));
                            for s in matches {
                                let kind = if s.category == SpotCategory::Stay {
                                    "Stay"
                                } else {
                                    "Food/Spot"
                                };
                                ui.label(format!("* [{}] {} ({})", kind, s.name, s.zone_id));
                                if !s.booking_note.is_empty() {
                                    ui.label(format!("  Booking: {}", s.booking_note));
                                }
                                ui.label(format!("  {}", s.note));
                            }
                        }
                    });
                }

                if !c.waypoints.is_empty() {
                    ui.collapsing(format!("Chrono-Waypoints ({})", c.waypoints.len()), |ui| {
                        for wp in &c.waypoints {
                            ui.strong(format!("{} ({} AD)", wp.name, wp.year_ad));
                            ui.label(&wp.historic_narrative);
                            ui.label(format!("Practical: {}", wp.visitor_info));
                            ui.separator();
                        }
                    });
                }

                ui.collapsing(
                    format!("Offline Phrasebook & Translate ({})", resolved.effective_phrases.len()),
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Filter:");
                            ui.text_edit_singleline(&mut state.phrase_query);
                        });
                        let q = state.phrase_query.trim().to_ascii_lowercase();
                        for ph in &resolved.effective_phrases {
                            if !q.is_empty()
                                && !ph.phrase.to_ascii_lowercase().contains(&q)
                                && !ph.local.to_ascii_lowercase().contains(&q)
                            {
                                continue;
                            }
                            ui.horizontal_wrapped(|ui| {
                                ui.strong(&ph.local);
                                ui.label(format!("({}) -- {}", ph.pronunciation, ph.phrase));
                                if ui.small_button("Flashcard").clicked() {
                                    state.flashcard_text = format!("{}\n({})", ph.local, ph.pronunciation);
                                }
                            });
                        }
                        if !state.flashcard_text.is_empty() {
                            ui.separator();
                            ui.label(egui::RichText::new(&state.flashcard_text).size(24.0).strong());
                        }
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Translate:");
                            ui.text_edit_singleline(&mut state.translate_input);
                            ui.label("To:");
                            ui.add(egui::TextEdit::singleline(&mut state.translate_target).desired_width(90.0));
                            if ui.button("Go").clicked() {
                                match engine.translate_text(
                                    &state.translate_input,
                                    &state.translate_target,
                                    Some(&c.id),
                                    None,
                                ) {
                                    Ok(res) => {
                                        state.translate_output = format!(
                                            "{} [{}] ({})",
                                            res.translated, res.source, res.pronunciation
                                        );
                                    }
                                    Err(e) => state.translate_output = format!("Translate error: {e}"),
                                }
                            }
                        });
                        if !state.translate_output.is_empty() {
                            ui.label(&state.translate_output);
                        }
                    },
                );
            });
    });
}

pub fn ui_notebook_tab(
    ui: &mut egui::Ui,
    engine: &TravelCraftEngine,
    state: &mut CartridgeStudioState,
) {
    ui.horizontal_wrapped(|ui| {
        ui.heading("Universal Travel Notebook");
        ui.separator();
        ui.selectable_value(&mut state.scope, NotebookScope::All, "All Trips");
        ui.selectable_value(&mut state.scope, NotebookScope::Trip, "This Trip");
        ui.selectable_value(&mut state.scope, NotebookScope::Cartridge, "This Cartridge");
        ui.selectable_value(&mut state.scope, NotebookScope::Country, "This Country");
    });
    ui.separator();

    // Trip bar
    let trips = engine.trip_list().unwrap_or_default();
    ui.horizontal_wrapped(|ui| {
        ui.label("Trip:");
        egui::ComboBox::from_id_salt("nb_trip_combo")
            .selected_text(if state.selected_trip_id.is_empty() {
                "(none)"
            } else {
                &state.selected_trip_id
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.selected_trip_id, String::new(), "(none)");
                for t in &trips {
                    ui.selectable_value(
                        &mut state.selected_trip_id,
                        t.trip_id.clone(),
                        format!("{} ({})", t.title, t.trip_id),
                    );
                }
            });
        ui.label("New trip:");
        ui.add(egui::TextEdit::singleline(&mut state.new_trip_title).desired_width(180.0));
        if ui.button("Create Trip").clicked() && !state.new_trip_title.trim().is_empty() {
            let req = NewTrip {
                trip_id: None,
                title: state.new_trip_title.trim().to_string(),
                start_date: None,
                end_date: None,
                cartridge_ids: vec![state.selected_cartridge_id.clone()],
                status: TripStatus::Active,
                notes: String::new(),
            };
            if let Ok(t) = engine.trip_create(&req) {
                state.selected_trip_id = t.trip_id;
                state.new_trip_title.clear();
            }
        }
        if !state.selected_trip_id.is_empty() && ui.button("Export Markdown").clicked() {
            state.exported_markdown = engine
                .trip_export_markdown(&state.selected_trip_id)
                .unwrap_or_else(|e| format!("Export error: {e}"));
        }
    });

    ui.separator();
    ui.columns(2, |cols| {
        // Left: quick note composer
        cols[0].heading("Add Entry");
        cols[0].horizontal(|ui| {
            ui.label("Kind:");
            for (k, lbl) in [
                (NotebookKind::Reflection, "Reflection"),
                (NotebookKind::Food, "Food"),
                (NotebookKind::Photo, "Photo"),
                (NotebookKind::Logistics, "Logistics"),
                (NotebookKind::Expense, "Expense"),
                (NotebookKind::Idea, "Idea"),
            ] {
                ui.selectable_value(&mut state.note_kind, k, lbl);
            }
        });
        cols[0].horizontal(|ui| {
            ui.label("Day:");
            ui.add(egui::TextEdit::singleline(&mut state.note_day).desired_width(40.0));
            ui.label("Title:");
            ui.text_edit_singleline(&mut state.note_title);
        });
        cols[0].label("Body:");
        cols[0].add(
            egui::TextEdit::multiline(&mut state.note_body)
                .desired_rows(4)
                .desired_width(f32::INFINITY),
        );
        cols[0].horizontal(|ui| {
            ui.label("Tags (comma):");
            ui.text_edit_singleline(&mut state.note_tags);
            if ui.button("Save Entry").clicked() && !state.note_body.trim().is_empty() {
                let country_code = state
                    .resolved
                    .as_ref()
                    .map(|r| r.cartridge.country_code.clone());
                let req = NewNotebookEntry {
                    trip_id: if state.selected_trip_id.is_empty() {
                        None
                    } else {
                        Some(state.selected_trip_id.clone())
                    },
                    cartridge_id: Some(state.selected_cartridge_id.clone()),
                    country_code,
                    waypoint_id: None,
                    kind: state.note_kind,
                    day_index: state.note_day.trim().parse::<u32>().ok(),
                    title: state.note_title.clone(),
                    body: state.note_body.clone(),
                    sensory_notes: String::new(),
                    photo_uris: Vec::new(),
                    lat: None,
                    lon: None,
                    tags: state
                        .note_tags
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                };
                if engine.notebook_add(&req).is_ok() {
                    state.note_title.clear();
                    state.note_body.clear();
                    state.note_tags.clear();
                }
            }
        });

        if !state.exported_markdown.is_empty() {
            cols[0].separator();
            cols[0].heading("Exported Trip Markdown");
            egui::ScrollArea::vertical()
                .id_salt("nb_md_export")
                .max_height(200.0)
                .show(&mut cols[0], |ui| {
                    ui.label(egui::RichText::new(&state.exported_markdown).monospace());
                });
        }

        // Right: scoped entry list
        cols[1].horizontal(|ui| {
            ui.heading("Entries");
            ui.label("Search:");
            ui.text_edit_singleline(&mut state.note_search);
        });
        let q = NotebookQuery {
            trip_id: if state.scope == NotebookScope::Trip && !state.selected_trip_id.is_empty() {
                Some(state.selected_trip_id.clone())
            } else {
                None
            },
            cartridge_id: if state.scope == NotebookScope::Cartridge {
                Some(state.selected_cartridge_id.clone())
            } else {
                None
            },
            country_code: if state.scope == NotebookScope::Country {
                state
                    .resolved
                    .as_ref()
                    .map(|r| r.cartridge.country_code.clone())
            } else {
                None
            },
            search: if state.note_search.trim().is_empty() {
                None
            } else {
                Some(state.note_search.clone())
            },
            limit: Some(100),
            ..NotebookQuery::default()
        };
        let entries = engine.notebook_query(&q).unwrap_or_default();
        egui::ScrollArea::vertical()
            .id_salt("nb_entries_scroll")
            .max_height(480.0)
            .show(&mut cols[1], |ui| {
                if entries.is_empty() {
                    ui.label("No entries in this scope yet.");
                }
                for e in &entries {
                    ui.horizontal(|ui| {
                        let day = e.day_index.map(|d| format!("Day {d} ")).unwrap_or_default();
                        ui.strong(format!("{day}[{}] {}", e.kind.as_str(), e.title));
                        if ui.small_button("Delete").clicked() {
                            let _ = engine.notebook_delete(&e.entry_id);
                        }
                    });
                    ui.label(&e.body);
                    if !e.tags.is_empty() {
                        ui.small(format!("tags: {}", e.tags.join(", ")));
                    }
                    ui.separator();
                }
            });
    });
}
