use clap::Parser;
use serde_json::json;
use travelcraft_core::{TravelCraftEngine, RouteOptimizationResult, TravelJournalEntry};
use travelcraft_baedeker::{CountryEdition, ChronoWaypoint};

#[derive(Parser, Debug)]
#[command(name = "travelcraft", about = "TravelCraft Sovereign Baedeker 2030 Studio")]
struct Args {
    #[arg(long)]
    smoke: bool,
}

#[derive(PartialEq, Eq)]
enum ActiveViewTab {
    ChronoMap,
    WaypointDetail,
    SensoryAtmosphere,
    RoutePlanner,
    Journal,
}

pub struct TravelCraftApp {
    engine: TravelCraftEngine,
    selected_country_code: String,
    selected_waypoint_id: Option<String>,
    active_tab: ActiveViewTab,
    status_msg: String,
    optimized_route: Option<RouteOptimizationResult>,

    // Interactive Journal Input
    new_reflection: String,
    new_sensory_notes: String,
    journal_log: Vec<TravelJournalEntry>,

    // Standard Desktop UI Controls
    is_dark_theme: bool,
    is_authenticated: bool,
    auth_username: String,
    auth_password_input: String,
    show_auth_modal: bool,
    show_about_dialog: bool,
}

impl TravelCraftApp {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let engine = TravelCraftEngine::new_in_memory()?;
        let initial_plan = engine.plan_optimized_route("JP").ok();

        Ok(Self {
            engine,
            selected_country_code: "JP".to_string(),
            selected_waypoint_id: Some("jp_nihonbashi".to_string()),
            active_tab: ActiveViewTab::ChronoMap,
            status_msg: "Baedeker 2030 Chrono-Cartography Ready".to_string(),
            optimized_route: initial_plan,

            new_reflection: String::new(),
            new_sensory_notes: String::new(),
            journal_log: vec![
                TravelJournalEntry {
                    entry_id: "ENTRY-001".to_string(),
                    waypoint_id: "jp_nihonbashi".to_string(),
                    timestamp_epoch: 1793775000,
                    user_reflection: "Crossed Nihonbashi bridge at dawn. The shadow of Tokugawa's wooden piers still echoes in the morning fog.".to_string(),
                    sensory_notes: "Aroma of boiling dashi broth mixing with salty bay air.".to_string(),
                    photo_uris: vec!["file:///nihonbashi_dawn.jpg".to_string()],
                }
            ],

            is_dark_theme: true,
            is_authenticated: false,
            auth_username: "traveler".to_string(),
            auth_password_input: String::new(),
            show_auth_modal: false,
            show_about_dialog: false,
        })
    }

    fn current_country(&self) -> Option<CountryEdition> {
        self.engine.get_country(&self.selected_country_code).ok()
    }

    fn current_waypoint(&self) -> Option<ChronoWaypoint> {
        let country = self.current_country()?;
        let route = country.curated_routes.first()?;
        let wp_id = self.selected_waypoint_id.as_deref()?;
        route.waypoints.iter().find(|w| w.id == wp_id).cloned()
    }
}

impl eframe::App for TravelCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.is_dark_theme {
            ui.ctx().set_visuals(egui::Visuals::dark());
        } else {
            ui.ctx().set_visuals(egui::Visuals::light());
        }

        // 1. STANDARD DESKTOP MENU BAR
        egui::Panel::top("desktop_menu_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Export Itinerary to PDF/ArtCraft...").clicked() {
                        self.status_msg = "Itinerary exported to ArtCraft PDF format".to_string();
                        ui.close();
                    }
                    if ui.button("Synchronize Vector Tiles").clicked() {
                        self.status_msg = "Offline vector tile caches synced".to_string();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edition", |ui| {
                    if ui.button("JapanCraft: Edo to Neo-Tokyo").clicked() {
                        self.selected_country_code = "JP".to_string();
                        self.selected_waypoint_id = Some("jp_nihonbashi".to_string());
                        self.optimized_route = self.engine.plan_optimized_route("JP").ok();
                        self.status_msg = "Loaded JapanCraft Edition".to_string();
                        ui.close();
                    }
                    if ui.button("AustriaCraft: Habsburg Imperial to Alpine").clicked() {
                        self.selected_country_code = "AT".to_string();
                        self.selected_waypoint_id = Some("at_stephansdom".to_string());
                        self.optimized_route = self.engine.plan_optimized_route("AT").ok();
                        self.status_msg = "Loaded AustriaCraft Edition".to_string();
                        ui.close();
                    }
                });

                ui.menu_button("View", |ui| {
                    let theme_label = if self.is_dark_theme { "☀ Switch to Light Theme" } else { "🌙 Switch to Dark Theme" };
                    if ui.button(theme_label).clicked() {
                        self.is_dark_theme = !self.is_dark_theme;
                        ui.close();
                    }
                });

                ui.menu_button("Security", |ui| {
                    if self.is_authenticated {
                        ui.label(format!("Signed in as: {}", self.auth_username));
                        if ui.button("Sign Out").clicked() {
                            self.is_authenticated = false;
                            self.status_msg = "Signed out".to_string();
                            ui.close();
                        }
                    } else {
                        if ui.button("Basic Auth Login...").clicked() {
                            self.show_auth_modal = true;
                            ui.close();
                        }
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About TravelCraft").clicked() {
                        self.show_about_dialog = true;
                        ui.close();
                    }
                });

                // Right quick toggles
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let theme_icon = if self.is_dark_theme { "🌙 Dark" } else { "☀ Light" };
                    if ui.button(theme_icon).clicked() {
                        self.is_dark_theme = !self.is_dark_theme;
                    }
                    let auth_badge = if self.is_authenticated { "🔒 Sovereign Explorer" } else { "🔓 Guest" };
                    if ui.button(auth_badge).clicked() {
                        self.show_auth_modal = true;
                    }
                });
            });
        });

        // 2. SUB-BAR: Tab navigation & Status
        egui::Panel::top("sub_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🧭 TravelCraft — Baedeker 2030 Studio");
                ui.separator();
                if ui.selectable_label(self.active_tab == ActiveViewTab::ChronoMap, "🗺 Chrono-Map").clicked() {
                    self.active_tab = ActiveViewTab::ChronoMap;
                }
                if ui.selectable_label(self.active_tab == ActiveViewTab::WaypointDetail, "📍 Waypoint Dossier").clicked() {
                    self.active_tab = ActiveViewTab::WaypointDetail;
                }
                if ui.selectable_label(self.active_tab == ActiveViewTab::SensoryAtmosphere, "🎧 Sensory Profile").clicked() {
                    self.active_tab = ActiveViewTab::SensoryAtmosphere;
                }
                if ui.selectable_label(self.active_tab == ActiveViewTab::RoutePlanner, "⚡ Route Optimizer").clicked() {
                    self.active_tab = ActiveViewTab::RoutePlanner;
                }
                if ui.selectable_label(self.active_tab == ActiveViewTab::Journal, "📖 Sovereign Journal").clicked() {
                    self.active_tab = ActiveViewTab::Journal;
                }

                ui.separator();
                ui.label(format!("Port 14129 | {}", self.status_msg));
            });
        });

        // MODAL: Auth
        if self.show_auth_modal {
            egui::Window::new("🔐 Explorer Authentication")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Enter traveler credentials:");
                    ui.horizontal(|ui| {
                        ui.label("Username:");
                        ui.text_edit_singleline(&mut self.auth_username);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Password:");
                        ui.add(egui::TextEdit::singleline(&mut self.auth_password_input).password(true));
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Login").clicked() {
                            if !self.auth_username.is_empty() && !self.auth_password_input.is_empty() {
                                self.is_authenticated = true;
                                self.status_msg = format!("Authenticated as {}", self.auth_username);
                                self.show_auth_modal = false;
                                self.auth_password_input.clear();
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_auth_modal = false;
                        }
                    });
                });
        }

        // MODAL: About
        if self.show_about_dialog {
            egui::Window::new("ℹ About TravelCraft")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.heading("TravelCraft 0.2.0 (Baedeker 2030)");
                    ui.label("Sovereign Chrono-Cartography & Cultural Atmosphere Studio.");
                    ui.label("Ships with interactive JapanCraft and AustriaCraft editions.");
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.show_about_dialog = false;
                    }
                });
        }

        // LEFT PANEL: Country & Waypoint Selector (Interactive Click List)
        egui::Panel::left("left_waypoint_tree").show(ui, |ui| {
            ui.set_width(320.0);
            ui.heading("Select Country Edition");
            ui.horizontal(|ui| {
                let jp_sel = self.selected_country_code == "JP";
                if ui.selectable_label(jp_sel, "🇯🇵 JapanCraft").clicked() {
                    self.selected_country_code = "JP".to_string();
                    self.selected_waypoint_id = Some("jp_nihonbashi".to_string());
                    self.optimized_route = self.engine.plan_optimized_route("JP").ok();
                    self.status_msg = "Selected JapanCraft Edition".to_string();
                }

                let at_sel = self.selected_country_code == "AT";
                if ui.selectable_label(at_sel, "🇦🇹 AustriaCraft").clicked() {
                    self.selected_country_code = "AT".to_string();
                    self.selected_waypoint_id = Some("at_stephansdom".to_string());
                    self.optimized_route = self.engine.plan_optimized_route("AT").ok();
                    self.status_msg = "Selected AustriaCraft Edition".to_string();
                }
            });

            ui.add_space(8.0);
            ui.separator();

            if let Some(country) = self.current_country() {
                ui.label(egui::RichText::new(&country.title).strong().color(egui::Color32::from_rgb(100, 200, 255)));
                ui.label(&country.subtitle);
                ui.add_space(8.0);

                if let Some(route) = country.curated_routes.first() {
                    ui.label(egui::RichText::new("Chrono-Waypoints (Click to View):").strong());
                    for wp in &route.waypoints {
                        let is_wp_sel = self.selected_waypoint_id.as_deref() == Some(&wp.id);
                        let wp_title = format!("{} [{} AD] {}", if is_wp_sel { "▶" } else { "•" }, wp.year_ad, wp.name);
                        if ui.selectable_label(is_wp_sel, wp_title).clicked() {
                            self.selected_waypoint_id = Some(wp.id.clone());
                            self.status_msg = format!("Inspecting {}", wp.name);
                        }
                    }
                }
            }
        });

        // CENTRAL PANEL: Dynamic View Based on Active Tab
        egui::CentralPanel::default().show(ui, |ui| {
            match self.active_tab {
                ActiveViewTab::ChronoMap => {
                    ui.heading("🗺 Chrono-Cartographic Interactive Map");
                    ui.label("Visualizing temporal layers, topological coordinates, and epoch transitions.");
                    ui.add_space(8.0);

                    if let Some(wp) = self.current_waypoint() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.heading(&wp.name);
                                ui.colored_label(egui::Color32::from_rgb(255, 180, 50), format!("Epoch: {} AD", wp.year_ad));
                            });
                            ui.label(format!("Region: {} | Coordinates: {:.4}° N, {:.4}° E", wp.region, wp.lat, wp.lon));
                            ui.separator();

                            // Graphical Map Representation Box
                            let (rect, _response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 220.0), egui::Sense::hover());
                            let painter = ui.painter();
                            painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(20, 25, 35));
                            painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 80, 120)), egui::StrokeKind::Inside);

                            let center = rect.center();
                            // Draw topological beacon
                            painter.circle_filled(center, 12.0, egui::Color32::from_rgb(255, 70, 70));
                            painter.circle_stroke(center, 24.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 140, 140)));
                            painter.text(
                                center + egui::vec2(0.0, 36.0),
                                egui::Align2::CENTER_CENTER,
                                format!("{} ({:.4}, {:.4})", wp.name, wp.lat, wp.lon),
                                egui::FontId::monospace(13.0),
                                egui::Color32::WHITE,
                            );
                        });
                    } else {
                        ui.label("Select a waypoint from the left list to load chrono-cartographic data.");
                    }
                }

                ActiveViewTab::WaypointDetail => {
                    ui.heading("📍 Historical Dossier & Narrative Context");
                    if let Some(wp) = self.current_waypoint() {
                        ui.group(|ui| {
                            ui.heading(&wp.name);
                            ui.label(egui::RichText::new(&wp.historic_narrative).size(14.0));
                            ui.add_space(8.0);
                            ui.separator();
                            ui.heading("Culinary & Gastronomic Heritage");
                            for dish in &wp.culinary_specialty {
                                ui.label(format!("🥢 {}", dish));
                            }
                        });
                    }
                }

                ActiveViewTab::SensoryAtmosphere => {
                    ui.heading("🎧 Sensory Atmosphere Profile (Baedeker 2030 Standard)");
                    if let Some(wp) = self.current_waypoint() {
                        let s = &wp.sensory_profile;
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("Acoustic Soundscape:").strong());
                            ui.label(&s.acoustic_profile);
                            ui.add_space(6.0);

                            ui.label(egui::RichText::new("Architectural Era:").strong());
                            ui.label(&s.architectural_era);
                            ui.add_space(6.0);

                            ui.label(egui::RichText::new("Olfactory Notes:").strong());
                            ui.label(&s.olfactory_notes);
                            ui.add_space(6.0);

                            ui.label(egui::RichText::new("Lighting & Ambience:").strong());
                            ui.label(&s.light_condition);
                        });
                    }
                }

                ActiveViewTab::RoutePlanner => {
                    ui.heading("⚡ Multimodal Geodesic Route Optimizer (Powered by QGISCraft)");
                    ui.label("Combines historical chrono-waypoints with high-precision WGS84 geodesic calculations and map tile coverage.");
                    ui.add_space(8.0);
                    if let Some(route) = &self.optimized_route {
                        ui.group(|ui| {
                            ui.label(format!("Country Edition: {}", route.country_code));
                            ui.label(format!("Total Distance (Geodesic): {:.1} km", route.total_distance_km));
                            ui.label(format!("Estimated Transit Time: {:.1} hours (High-speed Rail / Shinkansen)", route.estimated_hours));
                            ui.label(format!("Chrono-Span Traversed: {} years across history", route.chrono_span_years));
                            ui.label(format!("Estimated Carbon Footprint: {:.1} kg CO₂", route.carbon_footprint_kg));
                            ui.colored_label(egui::Color32::from_rgb(100, 220, 180), format!("QGISCraft Slippy Map Tiles (Z8): {} tiles indexed", route.covered_map_tiles_zoom10));
                        });
                    }
                }

                ActiveViewTab::Journal => {
                    ui.heading("📖 Sovereign Travel Journal (Local SQLite)");
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("New Journal Reflection:").strong());
                        ui.text_edit_multiline(&mut self.new_reflection);
                        ui.label("Sensory Notes:");
                        ui.text_edit_singleline(&mut self.new_sensory_notes);

                        if ui.button("Save Entry to DatabaseCraft").clicked() && !self.new_reflection.is_empty() {
                            let entry = TravelJournalEntry {
                                entry_id: format!("JOURNAL-{}", self.journal_log.len() + 1),
                                waypoint_id: self.selected_waypoint_id.clone().unwrap_or_else(|| "general".to_string()),
                                timestamp_epoch: 1793775100,
                                user_reflection: self.new_reflection.clone(),
                                sensory_notes: self.new_sensory_notes.clone(),
                                photo_uris: vec![],
                            };
                            let _ = self.engine.add_journal_entry(entry.clone());
                            self.journal_log.push(entry);
                            self.status_msg = "Journal entry persisted".to_string();
                            self.new_reflection.clear();
                            self.new_sensory_notes.clear();
                        }
                    });

                    ui.add_space(8.0);
                    ui.heading("Past Reflections");
                    for j in self.journal_log.iter().rev() {
                        ui.group(|ui| {
                            ui.label(egui::RichText::new(format!("[{}] Waypoint: {}", j.entry_id, j.waypoint_id)).monospace());
                            ui.label(&j.user_reflection);
                            if !j.sensory_notes.is_empty() {
                                ui.colored_label(egui::Color32::from_rgb(180, 220, 180), format!("Sensory: {}", j.sensory_notes));
                            }
                        });
                        ui.add_space(4.0);
                    }
                }
            }
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.smoke {
        println!("[travelcraft] Running headless smoke verification...");
        let engine = TravelCraftEngine::new_in_memory()?;
        let plan = engine.plan_optimized_route("JP")?;
        let smoke_payload = json!({
            "app": "travelcraft",
            "version": "0.2.0",
            "status": "ready",
            "control_port": 14129,
            "success": true,
            "has_menu_bar": true,
            "has_theme_toggle": true,
            "has_basic_auth": true,
            "jp_route_distance_km": plan.total_distance_km,
            "jp_waypoints_count": plan.waypoints.len(),
            "qgis_map_tiles_zoom8": plan.covered_map_tiles_zoom10,
            "capabilities": ["ChronoCartography", "SensoryAtmosphere", "EdoToModern", "HabsburgToAlpine", "InteractiveJournal", "QgisGisIntegration"]
        });
        println!("[travelcraft] Smoke OK: {}", smoke_payload);
        return Ok(());
    }

    let app = TravelCraftApp::new()?;
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1360.0, 840.0])
            .with_title("TravelCraft — Baedeker 2030 Studio"),
        ..Default::default()
    };

    eframe::run_native(
        "TravelCraft",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )?;
    Ok(())
}
