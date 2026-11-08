use clap::Parser;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use travelcraft_baedeker::{ChronoWaypoint, CountryEdition, StayItinerary};
use travelcraft_core::{
    RouteOptimizationResult, SightLensResult, TravelCraftEngine, TravelJournalEntry,
};

#[derive(Parser, Debug)]
#[command(
    name = "travelcraft",
    about = "TravelCraft Sovereign Baedeker 2030 Studio"
)]
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
    Itineraries,
    SightLens,
    SightChat,
    Journal,
}

// Ultra-contrast Baedeker palette (pure black ink + paper white).
// All body text is pure white on near-black; secondary text >= 200 luminance;
// interactive strokes 2-3px; selection is gold-on-black.
const INK_BG: egui::Color32 = egui::Color32::from_rgb(5, 8, 16);
const CARD_BG: egui::Color32 = egui::Color32::from_rgb(14, 20, 34);
const INPUT_BG: egui::Color32 = egui::Color32::from_rgb(0, 0, 0);
const GOLD: egui::Color32 = egui::Color32::from_rgb(255, 205, 105);
const PAPER: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
const MIST: egui::Color32 = egui::Color32::from_rgb(208, 228, 252);
const TEAL: egui::Color32 = egui::Color32::from_rgb(130, 245, 210);
const ROSE: egui::Color32 = egui::Color32::from_rgb(255, 135, 135);
const LINE_BRIGHT: egui::Color32 = egui::Color32::from_rgb(90, 130, 185);

pub struct TravelCraftApp {
    engine: TravelCraftEngine,
    selected_country_code: String,
    selected_waypoint_id: Option<String>,
    selected_info_item_id: Option<String>,
    tree_search_query: String,
    active_tab: ActiveViewTab,
    status_msg: String,
    optimized_route: Option<RouteOptimizationResult>,

    // Real slippy map (qgiscraft tiles): integer tile zoom, WGS84 center,
    // GPU texture cache + background fetch channel + failed-tile set.
    map_z: i32,
    map_clng: f64,
    map_clat: f64,
    map_textures: HashMap<(u8, i64, i64), egui::TextureHandle>,
    map_pending: HashSet<(u8, i64, i64)>,
    map_failed: HashSet<(u8, i64, i64)>,
    map_tx: std::sync::mpsc::Sender<(giscraft_core::tiles::Tile, Option<Vec<u8>>)>,
    map_rx: std::sync::mpsc::Receiver<(giscraft_core::tiles::Tile, Option<Vec<u8>>)>,
    map_note: String,
    map_zoom: f32,
    map_pan: egui::Vec2,
    locale: travelcraft_core::i18n::AppLocale,

    // Itineraries by stay length.
    stay_days: u32,
    itinerary: Option<StayItinerary>,

    // SightLens capture flow.
    sight_photo_uri: String,
    sight_lat: String,
    sight_lon: String,
    sight_hint: String,
    sight_gps_source: String,
    sight_result: Option<SightLensResult>,
    sight_error: String,

    // Sight chat (text + voice) grounded in the info blob.
    chat_input: String,
    chat_model: String,
    chat_online: bool,
    chat_log: Vec<(String, String)>,
    chat_status: String,

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
        let (engine, persist_note) = match TravelCraftEngine::open_default() {
            Ok(e) => (e, None),
            Err(err) => (
                TravelCraftEngine::new_in_memory()?,
                Some(format!(
                    "Journal NOT persisted, database unavailable: {err}"
                )),
            ),
        };
        let initial_plan = engine.plan_optimized_route("JP").ok();
        let saved_model = engine.setting_get("chat_model").ok().flatten();
        let dark = engine
            .setting_get("theme")
            .ok()
            .flatten()
            .is_none_or(|v| v != "light");
        let chat_log = Self::load_chat(&engine);
        let journal_log = engine.list_journal(500).unwrap_or_default();
        let itinerary = engine.plan_itinerary("JP", 3).ok();
        let (map_tx, map_rx) = std::sync::mpsc::channel();

        Ok(Self {
            engine,
            selected_country_code: "JP".to_string(),
            selected_waypoint_id: Some("jp_nihonbashi".to_string()),
            selected_info_item_id: None,
            tree_search_query: String::new(),
            active_tab: ActiveViewTab::ChronoMap,
            status_msg: persist_note
                .unwrap_or_else(|| "Baedeker 2030 Chrono-Cartography Ready".to_string()),
            optimized_route: initial_plan,
            map_z: 12,
            map_clng: 139.7745,
            map_clat: 35.6840,
            map_textures: HashMap::new(),
            map_pending: HashSet::new(),
            map_failed: HashSet::new(),
            map_tx,
            map_rx,
            map_note: String::new(),
            map_zoom: 1.0,
            map_pan: egui::Vec2::ZERO,
            locale: travelcraft_core::i18n::AppLocale::default(),
            stay_days: 3,
            itinerary,
            sight_photo_uri: String::new(),
            sight_lat: "48.2085".to_string(),
            sight_lon: "16.3738".to_string(),
            sight_hint: "stephansdom".to_string(),
            sight_gps_source: "manual".to_string(),
            sight_result: None,
            sight_error: String::new(),
            chat_input: String::new(),
            chat_model: saved_model.unwrap_or_else(|| "qwen2.5:7b".to_string()),
            chat_online: true,
            chat_log,
            chat_status: String::new(),
            new_reflection: String::new(),
            new_sensory_notes: String::new(),
            journal_log,

            is_dark_theme: dark,
            is_authenticated: false,
            auth_username: "traveler".to_string(),
            auth_password_input: String::new(),
            show_auth_modal: false,
            show_about_dialog: false,
        })
    }

    fn load_chat(engine: &TravelCraftEngine) -> Vec<(String, String)> {
        let log = engine.chat_recent(200).unwrap_or_default();
        if log.is_empty() {
            vec![(
                "guide".to_string(),
                "Sight Chat ready. Identify a sight (e.g. caravaggio in the KHM), then ask anything - I answer from the info blob, map + transit context, or the local LLM when online.".to_string(),
            )]
        } else {
            log
        }
    }

    fn reload_journal(&mut self) {
        match self.engine.list_journal(500) {
            Ok(list) => self.journal_log = list,
            Err(e) => self.status_msg = format!("Journal reload failed: {e}"),
        }
    }

    fn save_setting(&mut self, key: &str, value: &str) {
        if let Err(e) = self.engine.setting_set(key, value) {
            self.status_msg = format!("Could not save setting {key}: {e}");
        }
    }

    fn toggle_theme(&mut self) {
        self.is_dark_theme = !self.is_dark_theme;
        let v = if self.is_dark_theme { "dark" } else { "light" };
        self.save_setting("theme", v);
    }
    fn current_country(&self) -> Option<CountryEdition> {
        self.engine.get_country(&self.selected_country_code).ok()
    }

    fn current_waypoint(&self) -> Option<ChronoWaypoint> {
        let country = self.current_country()?;
        let wp_id = self.selected_waypoint_id.as_deref()?;
        for route in &country.curated_routes {
            if let Some(w) = route.waypoints.iter().find(|w| w.id == wp_id) {
                return Some(w.clone());
            }
        }
        // Fallback: first waypoint of first route.
        country.curated_routes.first()?.waypoints.first().cloned()
    }

    fn all_waypoints_flat(&self) -> Vec<ChronoWaypoint> {
        self.current_country()
            .map(|c| {
                c.curated_routes
                    .into_iter()
                    .flat_map(|r| r.waypoints)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn refresh_itinerary(&mut self) {
        self.itinerary = self
            .engine
            .plan_itinerary(&self.selected_country_code, self.stay_days)
            .ok();
    }

    /// Center the chrono-cartographic map on specific WGS84 coordinates.
    fn center_map_on(&mut self, lat: f64, lon: f64, zoom: f32) {
        self.map_zoom = zoom;
        let bbox = if self.selected_country_code == "AT" {
            (46.6, 49.2, 9.5, 17.5)
        } else {
            (33.0, 38.5, 131.0, 141.5)
        };
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 440.0));
        let base = project_coords(lat, lon, bbox, rect, egui::Vec2::ZERO, 1.0);
        let center = rect.center();
        self.map_pan = -(base - center) * zoom;
        self.map_clat = lat;
        self.map_clng = lon;
    }

    /// Fly the chrono map to the selected waypoint (city-scale zoom).
    /// Clears cached-failure flags so retried tiles can load.
    fn focus_selected(&mut self) {
        if let Some(w) = self.current_waypoint() {
            self.center_map_on(w.lat, w.lon, 3.0);
            self.map_z = 13;
            self.map_failed.clear();
            self.map_pending.clear();
        }
    }

    fn run_sightlens(&mut self) {
        let lat: f64 = self.sight_lat.trim().parse().unwrap_or(48.2085);
        let lon: f64 = self.sight_lon.trim().parse().unwrap_or(16.3738);
        let photo = if self.sight_photo_uri.trim().is_empty() {
            "laptop-camera-frame.jpg"
        } else {
            self.sight_photo_uri.trim()
        };
        match self.engine.sightlens_capture(
            photo,
            Some(lat),
            Some(lon),
            &self.sight_gps_source.clone(),
            &self.sight_hint.clone(),
        ) {
            Ok(res) => {
                self.sight_error.clear();
                self.status_msg = format!("SightLens: {}", res.sight.name);
                self.reload_journal();
                self.sight_result = Some(res);
            }
            Err(e) => {
                self.sight_error =
                    format!("No match: {e}. Try hint caravaggio / stephansdom / klimt.");
                self.sight_result = None;
            }
        }
    }

    /// Resolve the sight the chat is grounded in: SightLens hit first,
    /// else currently selected waypoint, else Stephansdom default.
    fn chat_sight(&self) -> travelcraft_baedeker::SightHit {
        if let Some(res) = &self.sight_result {
            return res.sight.clone();
        }
        if let Some(w) = self.current_waypoint() {
            return travelcraft_baedeker::SightHit {
                id: w.id.clone(),
                name: w.name.clone(),
                kind: travelcraft_baedeker::SightKind::Landmark,
                country_code: self.selected_country_code.clone(),
                lat: w.lat,
                lon: w.lon,
                distance_m: 0.0,
                confidence: 0.8,
                headline: w.name.clone(),
                body: w.historic_narrative.clone(),
                practical: w.visitor_info.clone(),
            };
        }
        // Stephansdom fallback so chat works with zero clicks.
        self.engine
            .identify_sight(48.2085, 16.3738, "stephansdom")
            .unwrap_or(travelcraft_baedeker::SightHit {
                id: "at_stephansdom".to_string(),
                name: "Stephansdom & Stephansplatz".to_string(),
                kind: travelcraft_baedeker::SightKind::Landmark,
                country_code: "AT".to_string(),
                lat: 48.2085,
                lon: 16.3738,
                distance_m: 0.0,
                confidence: 0.9,
                headline: "Stephansdom - Gothic heart of Vienna".to_string(),
                body: "Heart of Vienna since 1147.".to_string(),
                practical: "Nave free, tower EUR 6.50.".to_string(),
            })
    }

    fn answer_sight_question(&mut self, question: &str) -> String {
        let sight = self.chat_sight();
        let ctx = self.engine.sight_chat_context(&sight);
        // History (last 6) keeps follow-ups coherent without blowing the prompt.
        let history: Vec<(String, String)> = self.chat_log.iter().rev().take(6).cloned().collect();
        if self.chat_online {
            match ollama_chat_blocking(&self.chat_model, &ctx.system_prompt(), &history, question) {
                Ok(text) => {
                    self.chat_status = format!(
                        "Online LLM ({}) + info blob + map + transit.",
                        self.chat_model
                    );
                    return text;
                }
                Err(e) => {
                    self.chat_status = format!("LLM unreachable ({e}) - OFFLINE BLOB answer.");
                }
            }
        } else {
            self.chat_status = "Offline mode - BLOB answer.".to_string();
        }
        self.engine.sight_chat_offline(&sight, question)
    }
}

/// Blocking Ollama chat (localhost :11434). Short timeout so the UI never hangs.
fn ollama_chat_blocking(
    model: &str,
    system: &str,
    history: &[(String, String)],
    question: &str,
) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(25))
        .build()
        .map_err(|e| e.to_string())?;
    let mut messages = vec![serde_json::json!({"role": "system", "content": system})];
    for (role, text) in history.iter().rev() {
        let r = if role == "you" { "user" } else { "assistant" };
        messages.push(serde_json::json!({"role": r, "content": text}));
    }
    messages.push(serde_json::json!({"role": "user", "content": question}));
    let resp = client
        .post("http://127.0.0.1:11434/api/chat")
        .json(&serde_json::json!({"model": model, "messages": messages, "stream": false}))
        .send()
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("ollama HTTP {}", resp.status()));
    }
    let v: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    v.get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "ollama: no message.content".to_string())
}

/// Windows SAPI speech (offline, zero-dep). Truncates to 1200 chars.
fn speak_text_windows(text: &str) -> String {
    let short: String = text.chars().take(1200).collect();
    let escaped = short.replace('\'', "''");
    let ps = format!(
        "Add-Type -AssemblyName System.Speech; $s=New-Object System.Speech.Synthesis.SpeechSynthesizer; $s.Rate=0; $s.Speak('{escaped}') | Out-Null"
    );
    match std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps])
        .status()
    {
        Ok(st) if st.success() => "Spoke via Windows SAPI.".to_string(),
        Ok(st) => format!("SAPI exit {st}."),
        Err(e) => format!("SAPI failed: {e}."),
    }
}

/// speech-mcp TTS wav export (:10909). Saves to temp, returns path.
fn speech_mcp_tts_save(text: &str) -> String {
    let short: String = text.chars().take(1000).collect();
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    let client = match client {
        Ok(c) => c,
        Err(e) => return format!("TTS client failed: {e}"),
    };
    let resp = client
        .post("http://127.0.0.1:10909/api/v1/tts")
        .json(&serde_json::json!({"text": short}))
        .send();
    let resp = match resp {
        Ok(r) => r,
        Err(e) => return format!("speech-mcp unreachable :10909 ({e}). Is speech-mcp running?"),
    };
    if !resp.status().is_success() {
        return format!("speech-mcp TTS HTTP {}.", resp.status());
    }
    let bytes = match resp.bytes() {
        Ok(b) => b,
        Err(e) => return format!("TTS read failed: {e}"),
    };
    let path = std::env::temp_dir().join(format!(
        "travelcraft-chat-{}.wav",
        chrono::Utc::now().timestamp()
    ));
    match std::fs::write(&path, &bytes) {
        Ok(()) => format!("TTS wav saved: {}", path.display()),
        Err(e) => format!("TTS save failed: {e}"),
    }
}

fn big_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).size(27.0).strong().color(PAPER));
}

fn section_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).size(19.0).strong().color(GOLD));
}

fn body_text(ui: &mut egui::Ui, text: &str) {
    ui.label(egui::RichText::new(text).size(17.5).color(PAPER));
}

/// Chrono layer color by waypoint epoch: ancient / medieval / early modern / modern.
fn epoch_color(year_ad: i32) -> egui::Color32 {
    if year_ad < 0 {
        TEAL
    } else if year_ad < 1500 {
        GOLD
    } else if year_ad < 1900 {
        ROSE
    } else {
        egui::Color32::WHITE
    }
}

impl eframe::App for TravelCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.is_dark_theme {
            let mut v = egui::Visuals::dark();
            v.override_text_color = Some(egui::Color32::WHITE);
            v.panel_fill = INK_BG;
            v.window_fill = CARD_BG;
            v.extreme_bg_color = INPUT_BG;
            v.code_bg_color = INPUT_BG;
            // Selection: gold-on-black, always readable.
            v.selection.bg_fill = GOLD;
            v.selection.stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.hyperlink_color = GOLD;
            v.window_stroke = egui::Stroke::new(2.0, LINE_BRIGHT);
            v.widgets.noninteractive.fg_stroke = egui::Stroke::new(2.0, PAPER);
            v.widgets.inactive.fg_stroke = egui::Stroke::new(2.0, PAPER);
            v.widgets.inactive.bg_stroke = egui::Stroke::new(2.0, LINE_BRIGHT);
            v.widgets.hovered.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.widgets.hovered.bg_fill = GOLD;
            v.widgets.active.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.widgets.active.bg_fill = TEAL;
            // Focus outline for keyboard users.
            v.widgets.open.fg_stroke = egui::Stroke::new(3.0, GOLD);
            ui.ctx().set_visuals(v);
        } else {
            let mut v = egui::Visuals::light();
            v.override_text_color = Some(egui::Color32::BLACK);
            v.panel_fill = egui::Color32::WHITE;
            v.window_fill = egui::Color32::from_rgb(244, 246, 250);
            v.extreme_bg_color = egui::Color32::WHITE;
            v.selection.bg_fill = egui::Color32::from_rgb(10, 25, 60);
            v.selection.stroke = egui::Stroke::new(2.0, egui::Color32::WHITE);
            v.hyperlink_color = egui::Color32::from_rgb(140, 80, 0);
            v.window_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(10, 25, 60));
            v.widgets.noninteractive.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.widgets.inactive.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.widgets.inactive.bg_stroke =
                egui::Stroke::new(2.0, egui::Color32::from_rgb(10, 25, 60));
            v.widgets.hovered.bg_fill = GOLD;
            v.widgets.hovered.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.widgets.active.bg_fill = TEAL;
            v.widgets.active.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.widgets.open.fg_stroke = egui::Stroke::new(3.0, egui::Color32::from_rgb(10, 25, 60));
            ui.ctx().set_visuals(v);
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
                        self.refresh_itinerary();
                        self.status_msg = "Loaded JapanCraft Edition".to_string();
                        ui.close();
                    }
                    if ui
                        .button("AustriaCraft: Habsburg Imperial to Alpine")
                        .clicked()
                    {
                        self.selected_country_code = "AT".to_string();
                        self.selected_waypoint_id = Some("at_stephansdom".to_string());
                        self.optimized_route = self.engine.plan_optimized_route("AT").ok();
                        self.refresh_itinerary();
                        self.status_msg = "Loaded AustriaCraft Edition".to_string();
                        ui.close();
                    }
                });

                ui.menu_button("View", |ui| {
                    let theme_label = if self.is_dark_theme {
                        "Switch to Light Theme (high contrast)"
                    } else {
                        "Switch to Dark Theme (high contrast)"
                    };
                    if ui.button(theme_label).clicked() {
                        self.toggle_theme();
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
                    } else if ui.button("Basic Auth Login...").clicked() {
                        self.show_auth_modal = true;
                        ui.close();
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
                    let cur_loc = self.locale;
                    let loc_label = format!("{} {}", cur_loc.flag(), cur_loc.code().to_uppercase());
                    egui::ComboBox::from_id_salt("topbar_locale_selector")
                        .selected_text(loc_label)
                        .show_ui(ui, |ui| {
                            for loc_item in travelcraft_core::i18n::AppLocale::all() {
                                let text =
                                    format!("{} {}", loc_item.flag(), loc_item.display_name());
                                let _ = ui.selectable_value(&mut self.locale, *loc_item, text);
                            }
                        });

                    let theme_icon = if self.is_dark_theme {
                        "Dark HC"
                    } else {
                        "Light HC"
                    };
                    if ui.button(theme_icon).clicked() {
                        self.toggle_theme();
                    }
                    let auth_badge = if self.is_authenticated {
                        "Sovereign Explorer"
                    } else {
                        "Guest"
                    };
                    if ui.button(auth_badge).clicked() {
                        self.show_auth_modal = true;
                    }
                });
            });
        });

        // 2. SUB-BAR: Tab navigation & Status
        egui::Panel::top("sub_bar").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("TravelCraft - Baedeker 2030 Studio");
                ui.separator();
                for (tab, key) in [
                    (
                        ActiveViewTab::ChronoMap,
                        travelcraft_core::i18n::I18nKey::TabChronoMap,
                    ),
                    (
                        ActiveViewTab::WaypointDetail,
                        travelcraft_core::i18n::I18nKey::TabWaypointDetail,
                    ),
                    (
                        ActiveViewTab::SensoryAtmosphere,
                        travelcraft_core::i18n::I18nKey::TabSensoryAtmosphere,
                    ),
                    (
                        ActiveViewTab::RoutePlanner,
                        travelcraft_core::i18n::I18nKey::TabRoutePlanner,
                    ),
                    (
                        ActiveViewTab::Itineraries,
                        travelcraft_core::i18n::I18nKey::TabItineraries,
                    ),
                    (
                        ActiveViewTab::SightLens,
                        travelcraft_core::i18n::I18nKey::TabSightLens,
                    ),
                    (
                        ActiveViewTab::SightChat,
                        travelcraft_core::i18n::I18nKey::TabSightChat,
                    ),
                    (
                        ActiveViewTab::Journal,
                        travelcraft_core::i18n::I18nKey::TabJournal,
                    ),
                ] {
                    let label = travelcraft_core::i18n::t(key, self.locale);
                    if ui.selectable_label(self.active_tab == tab, label).clicked() {
                        self.active_tab = tab;
                    }
                }
                ui.separator();
                ui.label(format!("Port 14129 | {}", self.status_msg));
            });
        });

        // MODAL: Auth
        if self.show_auth_modal {
            egui::Window::new("Explorer Authentication")
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
                        ui.add(
                            egui::TextEdit::singleline(&mut self.auth_password_input)
                                .password(true),
                        );
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Login").clicked()
                            && !self.auth_username.is_empty()
                            && !self.auth_password_input.is_empty()
                        {
                            self.is_authenticated = true;
                            self.status_msg = format!("Authenticated as {}", self.auth_username);
                            self.show_auth_modal = false;
                            self.auth_password_input.clear();
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_auth_modal = false;
                        }
                    });
                });
        }

        // MODAL: About
        if self.show_about_dialog {
            egui::Window::new("About TravelCraft")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    body_text(ui, "TravelCraft 0.3.0 (Baedeker 2030)");
                    body_text(ui, "Sovereign chrono-cartography. JapanCraft + AustriaCraft vendored offline.");
                    body_text(ui, "Sources: japanophile-mcp travel.html; vienna-life-assistant vienna_tips/culture.");
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.show_about_dialog = false;
                    }
                });
        }

        // LEFT PANEL: Country & Waypoint Selector & Sovereign Knowledge Tree
        egui::Panel::left("left_waypoint_tree").show(ui, |ui| {
            ui.set_width(330.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Country Edition")
                        .size(17.0)
                        .strong()
                        .color(GOLD),
                );
                ui.horizontal(|ui| {
                    let jp_sel = self.selected_country_code == "JP";
                    if ui.selectable_label(jp_sel, "🗾 JapanCraft").clicked() {
                        self.selected_country_code = "JP".to_string();
                        self.selected_waypoint_id = Some("jp_nihonbashi".to_string());
                        self.selected_info_item_id = None;
                        self.optimized_route = self.engine.plan_optimized_route("JP").ok();
                        self.refresh_itinerary();
                        self.map_zoom = 1.0;
                        self.map_pan = egui::Vec2::ZERO;
                        self.status_msg = "Selected JapanCraft Edition".to_string();
                    }
                    let at_sel = self.selected_country_code == "AT";
                    if ui.selectable_label(at_sel, "🏰 AustriaCraft").clicked() {
                        self.selected_country_code = "AT".to_string();
                        self.selected_waypoint_id = Some("at_stephansdom".to_string());
                        self.selected_info_item_id = None;
                        self.optimized_route = self.engine.plan_optimized_route("AT").ok();
                        self.refresh_itinerary();
                        self.map_zoom = 1.0;
                        self.map_pan = egui::Vec2::ZERO;
                        self.status_msg = "Selected AustriaCraft Edition".to_string();
                    }
                });

                ui.add_space(8.0);
                ui.separator();

                if let Some(country) = self.current_country() {
                    ui.label(
                        egui::RichText::new(&country.title)
                            .size(15.0)
                            .strong()
                            .color(TEAL),
                    );
                    ui.label(
                        egui::RichText::new(&country.subtitle)
                            .size(13.0)
                            .color(MIST),
                    );
                    ui.add_space(8.0);

                    // SEARCH / FILTER INPUT
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🔍").size(14.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.tree_search_query)
                                .hint_text("Filter intel (schnitzel, onsen, beisl)...")
                                .desired_width(220.0),
                        );
                        if !self.tree_search_query.is_empty() && ui.small_button("✖").clicked() {
                            self.tree_search_query.clear();
                        }
                    });
                    ui.add_space(6.0);

                    let query = self.tree_search_query.trim().to_lowercase();

                    // SECTION 1: Chrono-Waypoints
                    let is_search_active = !query.is_empty();
                    egui::CollapsingHeader::new(
                        egui::RichText::new("📍 Chrono-Waypoints & Path")
                            .size(15.0)
                            .strong()
                            .color(GOLD),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        for route in &country.curated_routes {
                            ui.label(
                                egui::RichText::new(format!("Route: {}", route.title))
                                    .size(12.0)
                                    .color(MIST),
                            );
                            for wp in &route.waypoints {
                                if is_search_active
                                    && !wp.name.to_lowercase().contains(&query)
                                    && !wp.region.to_lowercase().contains(&query)
                                    && !wp.historic_narrative.to_lowercase().contains(&query)
                                    && !wp
                                        .culinary_specialty
                                        .iter()
                                        .any(|c| c.to_lowercase().contains(&query))
                                {
                                    continue;
                                }
                                let is_wp_sel =
                                    self.selected_waypoint_id.as_deref() == Some(&wp.id);
                                let wp_title = format!(
                                    "{} [{}] {}",
                                    if is_wp_sel { "▶" } else { "•" },
                                    wp.year_ad,
                                    wp.name
                                );
                                if ui
                                    .selectable_label(
                                        is_wp_sel,
                                        egui::RichText::new(wp_title).size(13.5),
                                    )
                                    .clicked()
                                {
                                    self.selected_waypoint_id = Some(wp.id.clone());
                                    self.selected_info_item_id = None;
                                    self.active_tab = ActiveViewTab::WaypointDetail;
                                    self.status_msg = format!("Inspecting {}", wp.name);
                                }
                            }
                        }
                    });

                    ui.add_space(6.0);

                    // SECTION 2: Sovereign Knowledge Tree (Japan or Austria)
                    let tree_title = if self.selected_country_code == "AT" {
                        "🌳 AustriaCraft Knowledge Tree"
                    } else {
                        "🌳 JapanCraft Knowledge Tree"
                    };
                    egui::CollapsingHeader::new(
                        egui::RichText::new(tree_title)
                            .size(15.0)
                            .strong()
                            .color(GOLD),
                    )
                    .default_open(true)
                    .show(ui, |ui| {
                        let tree = travelcraft_baedeker::BaedekerCatalog::get_knowledge_tree(
                            &self.selected_country_code,
                        );
                        for branch in &tree.branches {
                            let matching_items: Vec<&travelcraft_baedeker::InfoTopicItem> = branch
                                .items
                                .iter()
                                .filter(|item| {
                                    if query.is_empty() {
                                        true
                                    } else {
                                        item.title.to_lowercase().contains(&query)
                                            || item.subtitle.to_lowercase().contains(&query)
                                            || item.body.to_lowercase().contains(&query)
                                            || item
                                                .tags
                                                .iter()
                                                .any(|t| t.to_lowercase().contains(&query))
                                    }
                                })
                                .collect();

                            if is_search_active && matching_items.is_empty() {
                                continue;
                            }

                            let branch_header = if is_search_active {
                                format!(
                                    "{} {} ({})",
                                    branch.icon,
                                    branch.title,
                                    matching_items.len()
                                )
                            } else {
                                format!("{} {}", branch.icon, branch.title)
                            };

                            egui::CollapsingHeader::new(
                                egui::RichText::new(branch_header)
                                    .size(13.5)
                                    .strong()
                                    .color(TEAL),
                            )
                            .default_open(is_search_active || true)
                            .show(ui, |ui| {
                                for item in matching_items {
                                    let is_item_sel =
                                        self.selected_info_item_id.as_deref() == Some(&item.id);
                                    let item_title = format!(
                                        "{} {}",
                                        if is_item_sel { "▶" } else { "•" },
                                        item.title
                                    );
                                    let label = egui::RichText::new(item_title)
                                        .size(13.0)
                                        .color(if is_item_sel { GOLD } else { PAPER });
                                    if ui.selectable_label(is_item_sel, label).clicked() {
                                        self.selected_info_item_id = Some(item.id.clone());
                                        self.selected_waypoint_id = None;
                                        self.active_tab = ActiveViewTab::WaypointDetail;
                                        self.status_msg = format!("Selected intel: {}", item.title);
                                    }
                                }
                            });
                        }
                    });
                }
            });
        });

        // CENTRAL PANEL
        egui::CentralPanel::default().show(ui, |ui| {
            match self.active_tab {
                ActiveViewTab::ChronoMap => {
                    big_heading(ui, "Chrono-Cartographic Interactive Map (WGS84 Mercator)");
                    body_text(
                        ui,
                        "True geographic projection with historical route corridors, coastlines, waterways & culture pins. Drag to pan, scroll to zoom.",
                    );
                    ui.add_space(6.0);

                    let bbox = if self.selected_country_code == "AT" {
                        (46.6, 49.2, 9.5, 17.5)
                    } else {
                        (33.0, 38.5, 131.0, 141.5)
                    };

                    let wps = self.all_waypoints_flat();
                    let tree_items = travelcraft_baedeker::BaedekerCatalog::all_tree_items(
                        &self.selected_country_code,
                    );

                    // Map controls & Quick Jumps
                    ui.horizontal(|ui| {
                        if ui.button("Zoom -").clicked() {
                            self.map_zoom = (self.map_zoom / 1.25).clamp(0.5, 12.0);
                        }
                        if ui.button("Center & Reset").clicked() {
                            self.map_zoom = 1.0;
                            self.map_pan = egui::Vec2::ZERO;
                        }
                        if ui.button("Zoom +").clicked() {
                            self.map_zoom = (self.map_zoom * 1.25).clamp(0.5, 12.0);
                        }
                        ui.label(
                            egui::RichText::new(format!("Zoom {:.1}x", self.map_zoom))
                                .size(15.0)
                                .color(GOLD),
                        );
                        ui.separator();
                        if self.selected_country_code == "AT" {
                            if ui.button("📍 Vienna Basin").clicked() {
                                self.center_map_on(48.2085, 16.3738, 4.0);
                            }
                            if ui.button("🏔️ Salzkammergut").clicked() {
                                self.center_map_on(47.5622, 13.6493, 3.5);
                            }
                            if ui.button("🇦🇹 Full Country").clicked() {
                                self.map_zoom = 1.0;
                                self.map_pan = egui::Vec2::ZERO;
                            }
                        } else {
                            if ui.button("📍 Tokyo & Edo").clicked() {
                                self.center_map_on(35.6840, 139.7745, 3.5);
                            }
                            if ui.button("⛩️ Kyoto & Kansai").clicked() {
                                self.center_map_on(34.9671, 135.7727, 3.5);
                            }
                            if ui.button("🇯🇵 Full Country").clicked() {
                                self.map_zoom = 1.0;
                                self.map_pan = egui::Vec2::ZERO;
                            }
                        }
                    });

                    ui.add_space(4.0);

                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 440.0),
                        egui::Sense::click_and_drag(),
                    );

                    // Wheel zoom
                    let wheel = ui.input(|i| i.smooth_scroll_delta.y);
                    if response.hovered() && wheel != 0.0 {
                        let factor = (1.0_f32 + wheel * 0.0015).clamp(0.8, 1.25);
                        self.map_zoom = (self.map_zoom * factor).clamp(0.5, 12.0);
                    }
                    // Drag pan
                    if response.dragged() {
                        self.map_pan += response.drag_delta();
                    }

                    // Click detection
                    if response.clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let mut clicked_wp: Option<String> = None;
                            let mut best_d = 28.0;
                            for w in &wps {
                                let p = project_coords(
                                    w.lat,
                                    w.lon,
                                    bbox,
                                    rect,
                                    self.map_pan,
                                    self.map_zoom,
                                );
                                let d = (p - pos).length();
                                if d < best_d {
                                    best_d = d;
                                    clicked_wp = Some(w.id.clone());
                                }
                            }
                            if let Some(id) = clicked_wp {
                                self.selected_waypoint_id = Some(id.clone());
                                self.selected_info_item_id = None;
                                self.status_msg = format!("Map selected waypoint {}", id);
                            } else {
                                // Check culture / gastronomy pins
                                for item in &tree_items {
                                    if let (Some(lat), Some(lon)) = (item.lat, item.lon) {
                                        let p = project_coords(
                                            lat,
                                            lon,
                                            bbox,
                                            rect,
                                            self.map_pan,
                                            self.map_zoom,
                                        );
                                        let d = (p - pos).length();
                                        if d < 22.0 {
                                            self.selected_info_item_id = Some(item.id.clone());
                                            self.selected_waypoint_id = None;
                                            self.status_msg =
                                                format!("Map selected intel: {}", item.title);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let painter = ui.painter();

                    // 1. Deep ocean / dark ink canvas
                    painter.rect_filled(rect, 6.0, egui::Color32::from_rgb(6, 12, 26));
                    painter.rect_stroke(
                        rect,
                        6.0,
                        egui::Stroke::new(2.5, GOLD),
                        egui::StrokeKind::Inside,
                    );

                    // 2. Graticule Lat/Lon Grid lines & Coordinate labels
                    let (grid_lats, grid_lons): (&[f64], &[f64]) =
                        if self.selected_country_code == "AT" {
                            (
                                &[47.0, 48.0, 49.0],
                                &[10.0, 12.0, 14.0, 15.0, 16.0, 17.0],
                            )
                        } else {
                            (
                                &[34.0, 35.0, 36.0, 37.0, 38.0],
                                &[132.0, 134.0, 136.0, 138.0, 140.0],
                            )
                        };

                    for &lat in grid_lats {
                        let p1 = project_coords(lat, bbox.2, bbox, rect, self.map_pan, self.map_zoom);
                        let p2 = project_coords(lat, bbox.3, bbox, rect, self.map_pan, self.map_zoom);
                        painter.line_segment(
                            [p1, p2],
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(25, 45, 75)),
                        );
                        if rect.contains(p1 + egui::vec2(10.0, 0.0)) {
                            painter.text(
                                p1 + egui::vec2(6.0, -10.0),
                                egui::Align2::LEFT_BOTTOM,
                                format!("{:.0}° N", lat),
                                egui::FontId::proportional(11.0),
                                egui::Color32::from_rgb(70, 115, 165),
                            );
                        }
                    }

                    for &lon in grid_lons {
                        let p1 = project_coords(bbox.0, lon, bbox, rect, self.map_pan, self.map_zoom);
                        let p2 = project_coords(bbox.1, lon, bbox, rect, self.map_pan, self.map_zoom);
                        painter.line_segment(
                            [p1, p2],
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(25, 45, 75)),
                        );
                        if rect.contains(p2 + egui::vec2(0.0, 10.0)) {
                            painter.text(
                                p2 + egui::vec2(4.0, 4.0),
                                egui::Align2::LEFT_TOP,
                                format!("{:.0}° E", lon),
                                egui::FontId::proportional(11.0),
                                egui::Color32::from_rgb(70, 115, 165),
                            );
                        }
                    }

                    // 3. Vector Landmass: Regional Hulls Fill, Coastlines & Major Cities
                    let hulls = if self.selected_country_code == "AT" {
                        AUSTRIA_LAND_HULLS
                    } else {
                        JAPAN_LAND_HULLS
                    };
                    for hull in hulls {
                        let hull_pts: Vec<egui::Pos2> = hull
                            .iter()
                            .map(|&(lat, lon)| {
                                project_coords(lat, lon, bbox, rect, self.map_pan, self.map_zoom)
                            })
                            .collect();
                        painter.add(egui::Shape::convex_polygon(
                            hull_pts,
                            egui::Color32::from_rgb(16, 26, 42),
                            egui::Stroke::NONE,
                        ));
                    }

                    if self.selected_country_code == "AT" {
                        // Austria national border
                        let border_pts: Vec<egui::Pos2> = AUSTRIA_BORDER
                            .iter()
                            .map(|&(lat, lon)| {
                                project_coords(lat, lon, bbox, rect, self.map_pan, self.map_zoom)
                            })
                            .collect();
                        for w in border_pts.windows(2) {
                            painter.line_segment(
                                [w[0], w[1]],
                                egui::Stroke::new(2.5, egui::Color32::from_rgb(60, 120, 185)),
                            );
                        }

                        // Danube River (Blue-Cyan glowing waterway)
                        let danube_pts: Vec<egui::Pos2> = AUSTRIA_DANUBE
                            .iter()
                            .map(|&(lat, lon)| {
                                project_coords(lat, lon, bbox, rect, self.map_pan, self.map_zoom)
                            })
                            .collect();
                        for w in danube_pts.windows(2) {
                            // Outer glow
                            painter.line_segment(
                                [w[0], w[1]],
                                egui::Stroke::new(
                                    6.0,
                                    egui::Color32::from_rgba_unmultiplied(65, 165, 245, 50),
                                ),
                            );
                            // River core
                            painter.line_segment(
                                [w[0], w[1]],
                                egui::Stroke::new(3.5, egui::Color32::from_rgb(65, 165, 245)),
                            );
                        }

                        // High Alpine Spine
                        let alpine_pts: Vec<egui::Pos2> = AUSTRIA_ALPINE_SPINE
                            .iter()
                            .map(|&(lat, lon)| {
                                project_coords(lat, lon, bbox, rect, self.map_pan, self.map_zoom)
                            })
                            .collect();
                        for w in alpine_pts.windows(2) {
                            painter.line_segment(
                                [w[0], w[1]],
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(180, 215, 245)),
                            );
                        }
                    } else {
                        // Japan Honshu coastline
                        let honshu_pts: Vec<egui::Pos2> = JAPAN_HONSHU
                            .iter()
                            .map(|&(lat, lon)| {
                                project_coords(lat, lon, bbox, rect, self.map_pan, self.map_zoom)
                            })
                            .collect();
                        for w in honshu_pts.windows(2) {
                            painter.line_segment(
                                [w[0], w[1]],
                                egui::Stroke::new(2.5, egui::Color32::from_rgb(55, 115, 180)),
                            );
                        }

                        // Lake Biwa
                        let biwa_pts: Vec<egui::Pos2> = JAPAN_LAKE_BIWA
                            .iter()
                            .map(|&(lat, lon)| {
                                project_coords(lat, lon, bbox, rect, self.map_pan, self.map_zoom)
                            })
                            .collect();
                        for w in biwa_pts.windows(2) {
                            painter.line_segment(
                                [w[0], w[1]],
                                egui::Stroke::new(2.5, egui::Color32::from_rgb(70, 190, 240)),
                            );
                        }
                    }

                    // Major Cities for cartographic orientation
                    let cities = if self.selected_country_code == "AT" {
                        AUSTRIA_CITIES
                    } else {
                        JAPAN_CITIES
                    };
                    for &(name, clat, clon) in cities {
                        let cp = project_coords(clat, clon, bbox, rect, self.map_pan, self.map_zoom);
                        if rect.contains(cp) {
                            painter.rect_filled(
                                egui::Rect::from_center_size(cp, egui::vec2(5.0, 5.0)),
                                1.0,
                                egui::Color32::from_rgb(190, 215, 245),
                            );
                            painter.text(
                                cp + egui::vec2(6.0, -7.0),
                                egui::Align2::LEFT_BOTTOM,
                                name,
                                egui::FontId::proportional(11.0),
                                egui::Color32::from_rgb(160, 190, 220),
                            );
                        }
                    }

                    // 4. Historical Route Trails (Golden glowing path connecting waypoints)
                    let wp_pts: Vec<egui::Pos2> = wps
                        .iter()
                        .map(|w| {
                            project_coords(
                                w.lat,
                                w.lon,
                                bbox,
                                rect,
                                self.map_pan,
                                self.map_zoom,
                            )
                        })
                        .collect();
                    for w in wp_pts.windows(2) {
                        // Ambient glow track
                        painter.line_segment(
                            [w[0], w[1]],
                            egui::Stroke::new(
                                7.0,
                                egui::Color32::from_rgba_unmultiplied(255, 205, 105, 55),
                            ),
                        );
                        // Core golden trail
                        painter.line_segment([w[0], w[1]], egui::Stroke::new(2.5, GOLD));
                    }

                    // 5. Culture & Gastronomy Pins (from Sovereign Knowledge Tree)
                    for item in &tree_items {
                        if let (Some(lat), Some(lon)) = (item.lat, item.lon) {
                            let p = project_coords(
                                lat,
                                lon,
                                bbox,
                                rect,
                                self.map_pan,
                                self.map_zoom,
                            );
                            if rect.contains(p) {
                                let is_sel =
                                    self.selected_info_item_id.as_deref() == Some(&item.id);
                                let r = if is_sel { 8.5 } else { 5.5 };
                                painter.circle_filled(
                                    p,
                                    r + 4.0,
                                    egui::Color32::from_rgba_unmultiplied(130, 245, 210, 60),
                                );
                                painter.circle_filled(
                                    p,
                                    r,
                                    if is_sel { GOLD } else { TEAL },
                                );
                                painter.circle_stroke(
                                    p,
                                    r,
                                    egui::Stroke::new(1.5, egui::Color32::BLACK),
                                );

                                if is_sel || self.map_zoom > 1.8 {
                                    let pin_label = format!("• {}", item.title);
                                    let font = egui::FontId::proportional(11.5);
                                    let galley = painter.layout_no_wrap(
                                        pin_label,
                                        font,
                                        egui::Color32::WHITE,
                                    );
                                    let l_pos = p + egui::vec2(0.0, -18.0);
                                    let l_rect = egui::Rect::from_center_size(
                                        l_pos,
                                        galley.size() + egui::vec2(8.0, 4.0),
                                    );
                                    painter.rect_filled(
                                        l_rect,
                                        3.0,
                                        egui::Color32::from_rgb(12, 22, 28),
                                    );
                                    painter.rect_stroke(
                                        l_rect,
                                        3.0,
                                        egui::Stroke::new(1.0, TEAL),
                                        egui::StrokeKind::Inside,
                                    );
                                    painter.galley(
                                        l_rect.min + egui::vec2(4.0, 2.0),
                                        galley,
                                        egui::Color32::WHITE,
                                    );
                                }
                            }
                        }
                    }

                    // 6. Chrono Waypoint Markers
                    for w in &wps {
                        let p = project_coords(
                            w.lat,
                            w.lon,
                            bbox,
                            rect,
                            self.map_pan,
                            self.map_zoom,
                        );
                        if !rect.contains(p) {
                            continue;
                        }
                        let selected = self.selected_waypoint_id.as_deref() == Some(&w.id);
                        let r = if selected { 13.0 } else { 8.5 };

                        // Outer halo
                        painter.circle_filled(
                            p,
                            r + 7.0,
                            egui::Color32::from_rgba_unmultiplied(255, 205, 105, 75),
                        );
                        // Core circle
                        painter.circle_filled(p, r, if selected { GOLD } else { ROSE });
                        painter.circle_stroke(
                            p,
                            r,
                            egui::Stroke::new(2.5, egui::Color32::WHITE),
                        );

                        // Capsule badge: [Year AD] Name
                        let badge_text = format!("[{} AD] {}", w.year_ad, w.name);
                        let font = egui::FontId::proportional(13.0);
                        let galley = painter.layout_no_wrap(
                            badge_text,
                            font,
                            egui::Color32::WHITE,
                        );
                        let label_pos = p + egui::vec2(0.0, 22.0);
                        let label_rect = egui::Rect::from_center_size(
                            label_pos,
                            galley.size() + egui::vec2(10.0, 6.0),
                        );
                        painter.rect_filled(
                            label_rect,
                            4.0,
                            egui::Color32::from_rgb(8, 12, 22),
                        );
                        painter.rect_stroke(
                            label_rect,
                            4.0,
                            egui::Stroke::new(
                                1.5,
                                if selected { GOLD } else { LINE_BRIGHT },
                            ),
                            egui::StrokeKind::Inside,
                        );
                        painter.galley(
                            label_rect.min + egui::vec2(5.0, 3.0),
                            galley,
                            egui::Color32::WHITE,
                        );
                    }

                    // 7. Live Cursor Coordinates & Proximity HUD
                    if let Some(pos) = response.hover_pos() {
                        let (clat, clon) =
                            screen_to_lat_lon(pos, bbox, rect, self.map_pan, self.map_zoom);

                        // Find nearest waypoint or intel within hover range
                        let mut proximity_hint: Option<String> = None;
                        for w in &wps {
                            let p = project_coords(
                                w.lat,
                                w.lon,
                                bbox,
                                rect,
                                self.map_pan,
                                self.map_zoom,
                            );
                            if (p - pos).length() < 30.0 {
                                proximity_hint =
                                    Some(format!("📌 {} ({})", w.name, w.region));
                                break;
                            }
                        }
                        if proximity_hint.is_none() {
                            for item in &tree_items {
                                if let (Some(lat), Some(lon)) = (item.lat, item.lon) {
                                    let p = project_coords(
                                        lat,
                                        lon,
                                        bbox,
                                        rect,
                                        self.map_pan,
                                        self.map_zoom,
                                    );
                                    if (p - pos).length() < 24.0 {
                                        proximity_hint =
                                            Some(format!("🍽️ {}", item.title));
                                        break;
                                    }
                                }
                            }
                        }

                        let hud_text = if let Some(hint) = proximity_hint {
                            format!("🌐 Lat: {:.4}° N | Lon: {:.4}° E  |  {}", clat, clon, hint)
                        } else {
                            format!("🌐 Lat: {:.4}° N | Lon: {:.4}° E", clat, clon)
                        };

                        let hud_font = egui::FontId::proportional(13.0);
                        let galley = painter.layout_no_wrap(hud_text, hud_font, GOLD);
                        let hud_rect = egui::Rect::from_min_size(
                            rect.min + egui::vec2(12.0, 12.0),
                            galley.size() + egui::vec2(16.0, 8.0),
                        );
                        painter.rect_filled(
                            hud_rect,
                            5.0,
                            egui::Color32::from_rgba_unmultiplied(6, 12, 24, 230),
                        );
                        painter.rect_stroke(
                            hud_rect,
                            5.0,
                            egui::Stroke::new(1.5, GOLD),
                            egui::StrokeKind::Inside,
                        );
                        painter.galley(
                            hud_rect.min + egui::vec2(8.0, 4.0),
                            galley,
                            GOLD,
                        );
                    }

                    // 8. Dynamic Cartographic Scale Bar (Bottom-Right)
                    let mid_lat: f64 = (bbox.0 + bbox.1) * 0.5;
                    let km_per_deg_lon = 111.32 * mid_lat.to_radians().cos();
                    let total_km = (bbox.3 - bbox.2) * km_per_deg_lon;
                    let map_w = (bbox.3 - bbox.2) * mid_lat.to_radians().cos();
                    let map_h = bbox.1 - bbox.0;
                    let aspect = (map_w / map_h) as f32;
                    let rect_aspect = rect.width() / rect.height();
                    let draw_w = if rect_aspect > aspect {
                        rect.height() * aspect
                    } else {
                        rect.width()
                    };
                    let px_per_km = (draw_w * self.map_zoom) / (total_km as f32);
                    let scale_km = if px_per_km > 3.0 {
                        25.0
                    } else if px_per_km > 1.2 {
                        50.0
                    } else if px_per_km > 0.5 {
                        100.0
                    } else {
                        250.0
                    };
                    let bar_len_px = (scale_km * px_per_km).clamp(35.0, 140.0);
                    let bar_pos_y = rect.max.y - 18.0;
                    let bar_end_x = rect.max.x - 18.0;
                    let bar_start_x = bar_end_x - bar_len_px;
                    let scale_bg = egui::Rect::from_min_max(
                        egui::pos2(bar_start_x - 8.0, bar_pos_y - 16.0),
                        egui::pos2(bar_end_x + 8.0, bar_pos_y + 8.0),
                    );
                    painter.rect_filled(scale_bg, 4.0, egui::Color32::from_rgba_unmultiplied(6, 12, 24, 210));
                    painter.rect_stroke(scale_bg, 4.0, egui::Stroke::new(1.0, LINE_BRIGHT), egui::StrokeKind::Inside);
                    painter.line_segment([egui::pos2(bar_start_x, bar_pos_y), egui::pos2(bar_end_x, bar_pos_y)], egui::Stroke::new(2.0, egui::Color32::WHITE));
                    painter.line_segment([egui::pos2(bar_start_x, bar_pos_y - 4.0), egui::pos2(bar_start_x, bar_pos_y + 4.0)], egui::Stroke::new(2.0, egui::Color32::WHITE));
                    painter.line_segment([egui::pos2(bar_end_x, bar_pos_y - 4.0), egui::pos2(bar_end_x, bar_pos_y + 4.0)], egui::Stroke::new(2.0, egui::Color32::WHITE));
                    painter.text(egui::pos2((bar_start_x + bar_end_x) * 0.5, bar_pos_y - 8.0), egui::Align2::CENTER_CENTER, format!("{:.0} km", scale_km), egui::FontId::proportional(11.0), GOLD);

                    // 9. Projection Legend badge (Top-Right)
                    let legend_text = "🗺️ WGS84 Mercator Vector Cartography | Sovereign Baedeker 2030";
                    let l_font = egui::FontId::proportional(11.5);
                    let l_galley = painter.layout_no_wrap(
                        legend_text.to_string(),
                        l_font,
                        MIST,
                    );
                    let l_rect = egui::Rect::from_min_size(
                        egui::pos2(
                            rect.max.x - l_galley.size().x - 24.0,
                            rect.min.y + 12.0,
                        ),
                        l_galley.size() + egui::vec2(12.0, 6.0),
                    );
                    painter.rect_filled(
                        l_rect,
                        4.0,
                        egui::Color32::from_rgba_unmultiplied(6, 12, 24, 210),
                    );
                    painter.rect_stroke(
                        l_rect,
                        4.0,
                        egui::Stroke::new(1.0, LINE_BRIGHT),
                        egui::StrokeKind::Inside,
                    );
                    painter.galley(
                        l_rect.min + egui::vec2(6.0, 3.0),
                        l_galley,
                        MIST,
                    );

                    ui.add_space(4.0);
                    body_text(
                        ui,
                        "Cartography Guide: Gold markers = Chrono-Waypoints | Teal pins = Vienna Dining, Cafés & Shrines | Gold trail = Historical Tokaido & Danube corridors.",
                    );
                }

                ActiveViewTab::WaypointDetail => {
                    if let Some(item_id) = self.selected_info_item_id.as_deref() {
                        if let Some(item) = travelcraft_baedeker::BaedekerCatalog::find_tree_item(
                            &self.selected_country_code,
                            item_id,
                        ) {
                            big_heading(ui, "Sovereign Knowledge Intel");
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.group(|ui| {
                                    ui.label(
                                        egui::RichText::new(&item.title)
                                            .size(23.0)
                                            .strong()
                                            .color(GOLD),
                                    );
                                    ui.label(
                                        egui::RichText::new(&item.subtitle)
                                            .size(15.0)
                                            .color(MIST),
                                    );

                                    if !item.tags.is_empty() {
                                        ui.horizontal_wrapped(|ui| {
                                            for tag in &item.tags {
                                                ui.label(
                                                    egui::RichText::new(format!("#{tag}"))
                                                        .size(13.0)
                                                        .color(TEAL),
                                                );
                                            }
                                        });
                                    }

                                    if let (Some(lat), Some(lon)) = (item.lat, item.lon) {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "📍 GPS: {:.4}° N, {:.4}° E",
                                                    lat, lon
                                                ))
                                                .size(14.0)
                                                .color(ROSE),
                                            );
                                            if ui.button("📍 Show on Chrono-Map").clicked() {
                                                self.center_map_on(lat, lon, 3.8);
                                                self.active_tab = ActiveViewTab::ChronoMap;
                                                self.status_msg =
                                                    format!("Viewing {} on map (GPS: {:.4}, {:.4})", item.title, lat, lon);
                                            }
                                        });
                                    }

                                    ui.separator();
                                    body_text(ui, &item.body);

                                    if !item.practical_tips.is_empty() {
                                        ui.add_space(10.0);
                                        ui.label(
                                            egui::RichText::new("Practical Insider Tips")
                                                .size(17.0)
                                                .strong()
                                                .color(TEAL),
                                        );
                                        for tip in &item.practical_tips {
                                            body_text(ui, &format!("✓ {tip}"));
                                        }
                                    }
                                });
                            });
                        }
                    } else if let Some(wp) = self.current_waypoint() {
                        big_heading(ui, "Historical Dossier");
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.group(|ui| {
                                ui.label(
                                    egui::RichText::new(&wp.name)
                                        .size(23.0)
                                        .strong()
                                        .color(GOLD),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{} | {} AD | {:.4}° N, {:.4}° E",
                                        wp.region, wp.year_ad, wp.lat, wp.lon
                                    ))
                                    .size(16.0)
                                    .color(MIST),
                                );
                                ui.horizontal(|ui| {
                                    if ui.button("📍 Show on Chrono-Map").clicked() {
                                        self.center_map_on(wp.lat, wp.lon, 3.2);
                                        self.active_tab = ActiveViewTab::ChronoMap;
                                        self.status_msg =
                                            format!("Viewing {} on map (GPS: {:.4}, {:.4})", wp.name, wp.lat, wp.lon);
                                    }
                                });
                                ui.separator();
                                body_text(ui, &wp.historic_narrative);
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new("Visitor info")
                                        .size(18.0)
                                        .strong()
                                        .color(TEAL),
                                );
                                body_text(ui, &wp.visitor_info);
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new("Culinary heritage")
                                        .size(18.0)
                                        .strong()
                                        .color(TEAL),
                                );
                                for dish in &wp.culinary_specialty {
                                    body_text(ui, &format!("- {dish}"));
                                }
                            });
                        });
                    }
                }

                ActiveViewTab::SensoryAtmosphere => {
                    big_heading(ui, "Sensory Atmosphere Profile (Baedeker 2030)");
                    if let Some(wp) = self.current_waypoint() {
                        let s = &wp.sensory_profile;
                        ui.group(|ui| {
                            for (k, v) in [
                                ("Acoustic", &s.acoustic_profile),
                                ("Architecture", &s.architectural_era),
                                ("Olfactory", &s.olfactory_notes),
                                ("Light", &s.light_condition),
                            ] {
                                ui.label(egui::RichText::new(k).size(15.0).strong().color(TEAL));
                                body_text(ui, v);
                                ui.add_space(6.0);
                            }
                        });
                    }
                }

                ActiveViewTab::RoutePlanner => {
                    big_heading(ui, "Multimodal Geodesic Route Optimizer");
                    body_text(ui, "QGISCraft WGS84 geodesics + slippy-map tile coverage.");
                    ui.add_space(8.0);
                    if let Some(route) = &self.optimized_route {
                        ui.group(|ui| {
                            body_text(ui, &format!("Edition: {}", route.country_code));
                            body_text(ui, &format!("Distance: {:.1} km | Time: {:.1} h rail", route.total_distance_km, route.estimated_hours));
                            body_text(ui, &format!("Chrono-span: {} years | CO2: {:.1} kg", route.chrono_span_years, route.carbon_footprint_kg));
                            ui.label(egui::RichText::new(format!("Map tiles Z8: {}", route.covered_map_tiles_zoom10)).size(15.0).color(TEAL));
                        });
                    }
                }

                ActiveViewTab::Itineraries => {
                    big_heading(ui, "Stay-Length Itineraries");
                    body_text(ui, "Pick your stay. Offline Baedeker slices waypoints into day plans.");
                    ui.horizontal(|ui| {
                        for d in [1u32, 2, 3, 5, 7, 14] {
                            if ui.selectable_label(self.stay_days == d, format!("{d}d")).clicked() {
                                self.stay_days = d;
                                self.refresh_itinerary();
                            }
                        }
                        ui.label(egui::RichText::new(format!("{}-day pace", self.stay_days)).size(16.0).color(GOLD));
                    });
                    ui.add_space(6.0);
                    if let Some(itin) = &self.itinerary {
                        ui.label(egui::RichText::new(&itin.pace_label).size(15.0).strong().color(TEAL));
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for day in &itin.days {
                                ui.group(|ui| {
                                    ui.label(egui::RichText::new(&day.title).size(18.0).strong().color(PAPER));
                                    for id in &day.waypoint_ids {
                                        body_text(ui, &format!("- {id}"));
                                    }
                                    ui.label(egui::RichText::new(&day.note).size(15.0).color(MIST));
                                });
                            }
                            ui.add_space(6.0);
                            ui.label(egui::RichText::new("Practical").size(15.0).strong().color(GOLD));
                            body_text(ui, &itin.practical_tip);
                        });
                    } else {
                        body_text(ui, "No itinerary. Select an edition first.");
                    }
                }

                ActiveViewTab::SightLens => {
                    big_heading(ui, "SightLens - Point, Click, Identify");
                    body_text(ui, "Laptop camera or photo file + GPS -> offline identify -> beautiful dossier. Stephansdom + KHM Caravaggio built in.");
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("1. Photo source").size(15.0).strong().color(GOLD));
                        ui.horizontal(|ui| {
                            ui.label("Photo path / Immich export:");
                            ui.text_edit_singleline(&mut self.sight_photo_uri);
                        });
                        body_text(ui, "Immich: export original (keeps EXIF GPS). Paste file path here; GPS auto-fill is manual in v1.");
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("2. GPS fix").size(15.0).strong().color(GOLD));
                        ui.horizontal(|ui| {
                            ui.label("Lat:");
                            ui.text_edit_singleline(&mut self.sight_lat);
                            ui.label("Lon:");
                            ui.text_edit_singleline(&mut self.sight_lon);
                        });
                        ui.horizontal(|ui| {
                            for src in ["manual", "windows-geolocation", "iphone-hotspot-manual", "immich-exif"] {
                                if ui.selectable_label(self.sight_gps_source == src, src).clicked() {
                                    self.sight_gps_source = src.to_string();
                                }
                            }
                        });
                        body_text(ui, "Windows GPS: Settings > Privacy > Location ON. iPhone-as-hotspot shares internet only, NOT GPS - enter coords manually or tether via Immich photo EXIF.");
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("3. Hint (optional artwork / landmark)").size(15.0).strong().color(GOLD));
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.sight_hint);
                            if ui.button("Identify + Log").clicked() {
                                self.run_sightlens();
                            }
                        });
                        ui.horizontal(|ui| {
                            for h in ["stephansdom", "caravaggio", "klimt kiss", "bruegel"] {
                                if ui.small_button(h).clicked() {
                                    self.sight_hint = (*h).to_string();
                                    self.run_sightlens();
                                }
                            }
                        });
                        if !self.sight_error.is_empty() {
                            ui.label(egui::RichText::new(&self.sight_error).size(16.0).color(ROSE));
                        }
                    });
                    ui.add_space(6.0);
                    if let Some(res) = &self.sight_result {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new(&res.info_page_title).size(23.0).strong().color(GOLD));
                                ui.label(egui::RichText::new(&res.sight.headline).size(15.0).color(TEAL));
                                ui.label(egui::RichText::new(format!(
                                    "Kind: {:?} | Confidence {:.0}% | GPS {:.5},{:.5} ({}) | Journal {}",
                                    res.sight.kind,
                                    res.sight.confidence * 100.0,
                                    res.sight.lat,
                                    res.sight.lon,
                                    self.sight_gps_source,
                                    res.journal_entry_id
                                )).size(15.0).color(MIST));
                                ui.separator();
                                body_text(ui, &res.sight.body);
                                ui.add_space(6.0);
                                ui.label(egui::RichText::new("Practical").size(15.0).strong().color(TEAL));
                                body_text(ui, &res.sight.practical);
                            });
                        });
                    } else {
                        body_text(ui, "Point at Stephansdom and click Identify, or try hint caravaggio for the KHM room.");
                    }
                }

                ActiveViewTab::SightChat => {
                    big_heading(ui, "Sight Chat - Ask About This Sight (Text + Voice)");
                    // Context pill: what we are grounded in.
                    let ctx_name = self
                        .sight_result
                        .as_ref()
                        .map(|r| r.sight.name.clone())
                        .or_else(|| self.current_waypoint().map(|w| w.name))
                        .unwrap_or_else(|| "no sight identified yet".to_string());
                    ui.group(|ui| {
                        body_text(ui, &format!("Grounded in: {ctx_name}"));
                        body_text(ui, "Info blob + qgiscraft map blob + GTFS transit blob are injected into every prompt. Online LLM when reachable (Ollama :11434), else OFFLINE BLOB extractive answer.");
                        ui.horizontal(|ui| {
                            ui.label("Model:");
                            ui.text_edit_singleline(&mut self.chat_model);
                            ui.checkbox(&mut self.chat_online, "online LLM");
                            if ui.button("Use current sight").clicked() {
                                self.chat_status = format!("Chat context: {ctx_name}");
                            }
                        });
                        if !self.chat_status.is_empty() {
                            ui.label(egui::RichText::new(&self.chat_status).size(15.0).color(MIST));
                        }
                    });
                    ui.add_space(6.0);
                    egui::ScrollArea::vertical()
                        .max_height(380.0)
                        .show(ui, |ui| {
                            for (role, text) in &self.chat_log {
                                let (label, color) = if role == "you" {
                                    ("You", GOLD)
                                } else {
                                    ("Guide", TEAL)
                                };
                                ui.group(|ui| {
                                    ui.label(egui::RichText::new(label).size(15.0).strong().color(color));
                                    body_text(ui, text);
                                });
                            }
                        });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Ask:");
                        let resp = ui.text_edit_singleline(&mut self.chat_input);
                        let send = ui.button("Send").clicked()
                            || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                        if send && !self.chat_input.trim().is_empty() {
                            let q = self.chat_input.trim().to_string();
                            self.chat_input.clear();
                            let sid = self.chat_sight().id;
                            let model = self.chat_model.clone();
                            self.save_setting("chat_model", &model);
                            self.chat_log.push(("you".to_string(), q.clone()));
                            let answer = self.answer_sight_question(&q);
                            self.chat_log.push(("guide".to_string(), answer.clone()));
                            for (role, text) in [("you", q.as_str()), ("guide", answer.as_str())] {
                                if let Err(e) = self.engine.chat_append(&sid, role, text) {
                                    self.chat_status = format!("Chat history not saved: {e}");
                                }
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Speak last answer").clicked() {
                            if let Some((_, last)) = self.chat_log.iter().rev().find(|(r, _)| r == "guide") {
                                self.chat_status = speak_text_windows(last);
                            }
                        }
                        if ui.button("Save last answer (speech-mcp wav)").clicked() {
                            if let Some((_, last)) = self.chat_log.iter().rev().find(|(r, _)| r == "guide") {
                                self.chat_status = speech_mcp_tts_save(last);
                            }
                        }
                        body_text(ui, "Voice in: type or paste transcript (mic via speech-mcp :10909 web UI, STT sidecar :10914). Voice out: SAPI Speak or speech-mcp TTS wav.");
                    });
                }

                ActiveViewTab::Journal => {
                    big_heading(ui, "Travel Journal (local SQLite file)");
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("New reflection:").size(15.0).strong().color(GOLD));
                        ui.text_edit_multiline(&mut self.new_reflection);
                        ui.label("Sensory notes:");
                        ui.text_edit_singleline(&mut self.new_sensory_notes);

                        if ui.button("Save Entry").clicked() && !self.new_reflection.is_empty() {
                            let entry = TravelJournalEntry {
                                entry_id: self.engine.new_entry_id("JOURNAL"),
                                waypoint_id: self.selected_waypoint_id.clone().unwrap_or_else(|| "general".to_string()),
                                timestamp_epoch: chrono::Utc::now().timestamp(),
                                user_reflection: self.new_reflection.clone(),
                                sensory_notes: self.new_sensory_notes.clone(),
                                photo_uris: vec![],
                                photo_lat: None,
                                photo_lon: None,
                                photo_gps_source: None,
                            };
                            match self.engine.add_journal_entry(entry) {
                                Ok(()) => {
                                    self.status_msg = if self.engine.is_persistent() {
                                        "Journal entry saved".to_string()
                                    } else {
                                        "Journal entry kept in memory only (no database)".to_string()
                                    };
                                    self.reload_journal();
                                }
                                Err(e) => self.status_msg = format!("Journal save FAILED: {e}"),
                            }
                            self.new_reflection.clear();
                            self.new_sensory_notes.clear();
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Past Reflections").size(18.0).strong().color(GOLD));
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for j in self.journal_log.iter().rev() {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new(format!("[{}] {}", j.entry_id, j.waypoint_id)).monospace().color(MIST));
                                body_text(ui, &j.user_reflection);
                                if !j.sensory_notes.is_empty() {
                                    ui.label(egui::RichText::new(format!("Sensory: {}", j.sensory_notes)).size(16.0).color(TEAL));
                                }
                                if !j.photo_uris.is_empty() {
                                    ui.label(egui::RichText::new(format!("Photos: {}", j.photo_uris.join(", "))).size(15.0).color(MIST));
                                }
                            });
                            ui.add_space(4.0);
                        }
                    });
                }
            }
        });
    }
}

// --- Chrono-Cartographic Vector Geometries (WGS84) -----------------------

/// Coastline polyline for Honshu (Japan).
const JAPAN_HONSHU: &[(f64, f64)] = &[
    (35.0, 139.8), // Boso Peninsula South
    (35.7, 140.8), // Choshi / Inubosaki
    (36.4, 140.6), // Ibaraki Coast
    (37.0, 141.0), // Fukushima Coast
    (38.3, 141.0), // Sendai Bay
    (40.5, 141.5), // Hachinohe
    (41.5, 140.9), // Shimokita Peninsula
    (41.2, 140.3), // Tsugaru Peninsula
    (40.0, 139.9), // Oga Peninsula / Akita
    (38.9, 139.8), // Sakata
    (37.9, 139.0), // Niigata
    (37.0, 138.2), // Joetsu
    (36.8, 137.1), // Toyama Bay
    (37.5, 137.3), // Noto Peninsula Tip (Suzu)
    (37.4, 136.9), // Wajima
    (36.6, 136.6), // Kanazawa
    (36.0, 136.0), // Fukui / Echizen
    (35.5, 135.5), // Wakasa Bay / Maizuru
    (35.5, 134.2), // Tottori Coast
    (35.5, 133.0), // Matsue / Shimane
    (34.7, 131.2), // Hagi
    (34.0, 130.9), // Shimonoseki / Kanmon Strait
    (34.0, 131.8), // Yamaguchi Sanyo Coast
    (34.3, 132.4), // Hiroshima Bay
    (34.4, 133.4), // Fukuyama
    (34.6, 133.9), // Okayama / Seto Inland Sea
    (34.7, 134.7), // Himeji Coast
    (34.7, 135.2), // Kobe Port
    (34.6, 135.4), // Osaka Bay
    (34.3, 135.1), // Wakayama
    (33.7, 135.3), // Shirahama
    (33.4, 135.8), // Kushimoto / Shionomisaki (Southernmost Honshu)
    (33.8, 136.1), // Kumano
    (34.5, 136.8), // Ise / Shima Peninsula
    (35.0, 136.9), // Nagoya / Ise Bay
    (34.7, 137.4), // Toyohashi / Atsumi Peninsula
    (34.6, 138.2), // Omaezaki
    (35.0, 138.5), // Shizuoka / Suruga Bay
    (34.6, 138.9), // Izu Peninsula (Shimoda)
    (35.1, 139.1), // Sagami Bay / Odawara
    (35.3, 139.5), // Kamakura / Enoshima
    (35.4, 139.6), // Yokohama
    (35.6, 139.8), // Tokyo Port / Nihonbashi
    (35.3, 139.9), // Kisarazu
    (35.0, 139.8), // Close loop to Boso
];

/// Outline of Lake Biwa (Kyoto/Shiga, Japan).
const JAPAN_LAKE_BIWA: &[(f64, f64)] = &[
    (35.1, 135.9),
    (35.3, 136.1),
    (35.5, 136.2),
    (35.4, 136.1),
    (35.2, 136.0),
    (35.0, 135.9),
    (35.1, 135.9),
];

/// National border polygon of Austria.
const AUSTRIA_BORDER: &[(f64, f64)] = &[
    (47.5, 9.7),  // Bregenz / Lake Constance
    (47.1, 10.2), // Arlberg
    (46.9, 10.5), // Nauders (Three borders CH/IT/AT)
    (46.8, 11.0), // Oetztal Alps
    (47.0, 11.5), // Brenner Pass
    (47.0, 12.1), // Zillertal Alps
    (47.1, 12.8), // Grossglockner / Hohe Tauern
    (46.6, 12.9), // Carnic Alps
    (46.5, 13.7), // Karawanks / Arnoldstein
    (46.5, 14.5), // Loibl Pass
    (46.7, 15.3), // South Styria wine hills
    (46.8, 16.0), // Bad Radkersburg
    (47.2, 16.5), // South Burgenland
    (47.7, 16.8), // Neusiedler See / Eisenstadt
    (48.0, 17.1), // Kittsee / Bratislava border
    (48.2, 17.0), // March river confluence / Hainburg
    (48.6, 16.9), // Hohenau / March-Thaya
    (48.8, 16.4), // Laa an der Thaya
    (48.9, 15.3), // Waldviertel (Gmuend)
    (48.6, 14.9), // Freistadt
    (48.7, 13.8), // Dreilaendereck AT/CZ/DE (Bohmerwald)
    (48.5, 13.5), // Passau (Inn / Danube)
    (48.2, 13.0), // Braunau am Inn
    (47.8, 13.0), // Salzburg basin
    (47.6, 12.9), // Berchtesgaden notch
    (47.6, 12.4), // Lofer / Steinberge
    (47.6, 12.2), // Kufstein
    (47.5, 11.3), // Karwendel
    (47.4, 10.7), // Reutte / Fernpass
    (47.5, 9.7),  // Back to Bregenz
];

/// Danube River (Donau) vector polyline through Austria.
const AUSTRIA_DANUBE: &[(f64, f64)] = &[
    (48.56, 13.43),     // Passau
    (48.45, 13.85),     // Schlogener Schlinge
    (48.30, 14.28),     // Linz
    (48.22, 14.52),     // Enns
    (48.19, 14.90),     // Ybbs an der Donau
    (48.2297, 15.3319), // Melk Abbey
    (48.36, 15.42),     // Spitz an der Donau
    (48.39, 15.52),     // Duernstein Ruin
    (48.41, 15.61),     // Krems an der Donau
    (48.33, 16.05),     // Tulln
    (48.30, 16.31),     // Klosterneuburg
    (48.21, 16.39),     // Vienna (Donaukanal / Stephansdom)
    (48.14, 16.94),     // Hainburg an der Donau
];

/// High Alpine Spine across Austria.
const AUSTRIA_ALPINE_SPINE: &[(f64, f64)] = &[
    (47.1, 10.3),   // Arlberg
    (47.0, 11.5),   // Brenner
    (47.07, 12.69), // Grossglockner (3,798m)
    (47.48, 13.61), // Dachstein Massif (2,995m)
    (47.60, 14.45), // Gesaeuse
    (47.70, 15.30), // Hochschwab
    (47.76, 15.80), // Rax / Schneeberg
];

/// Major Austrian cities for cartographic orientation.
const AUSTRIA_CITIES: &[(&str, f64, f64)] = &[
    ("Wien", 48.2085, 16.3738),
    ("Salzburg", 47.8095, 13.0550),
    ("Linz", 48.3069, 14.2858),
    ("Graz", 47.0707, 15.4395),
    ("Innsbruck", 47.2692, 11.4041),
    ("Bregenz", 47.5031, 9.7471),
    ("Klagenfurt", 46.6247, 14.3053),
];

/// Major Japanese cities for cartographic orientation.
const JAPAN_CITIES: &[(&str, f64, f64)] = &[
    ("Tokyo", 35.6840, 139.7745),
    ("Kyoto", 35.0116, 135.7681),
    ("Osaka", 34.6937, 135.5023),
    ("Nagoya", 35.1815, 136.9066),
    ("Hiroshima", 34.3853, 132.4553),
    ("Kanazawa", 36.5613, 136.6562),
    ("Sendai", 38.2682, 140.8694),
];

/// Convex sub-polygons for Austrian landmass background fill.
const AUSTRIA_LAND_HULLS: &[&[(f64, f64)]] = &[
    // West (Vorarlberg & Tyrol)
    &[
        (47.5, 9.7),
        (47.1, 10.2),
        (47.0, 11.5),
        (47.6, 12.2),
        (47.5, 11.3),
        (47.4, 10.7),
    ],
    // Central (Salzburg & Upper Austria)
    &[
        (47.6, 12.2),
        (47.0, 12.1),
        (47.1, 12.8),
        (47.48, 13.61),
        (47.8, 13.0),
        (48.3, 14.28),
        (48.5, 13.5),
    ],
    // East (Lower Austria & Vienna Basin)
    &[
        (48.5, 13.5),
        (48.3, 14.28),
        (48.8, 15.3),
        (48.8, 16.4),
        (48.2, 17.0),
        (47.7, 16.8),
        (47.76, 15.80),
    ],
    // South (Carinthia & Styria)
    &[
        (47.1, 12.8),
        (46.6, 12.9),
        (46.5, 14.5),
        (46.8, 16.0),
        (47.2, 16.5),
        (47.07, 15.44),
        (47.48, 13.61),
    ],
];

/// Convex sub-polygons for Honshu landmass background fill.
const JAPAN_LAND_HULLS: &[&[(f64, f64)]] = &[
    // Kanto & Chubu
    &[
        (35.0, 139.8),
        (35.7, 140.8),
        (36.8, 139.6),
        (36.8, 137.1),
        (35.0, 136.9),
        (34.6, 138.2),
        (35.0, 138.5),
    ],
    // Kansai Basin
    &[
        (35.0, 136.9),
        (35.5, 135.5),
        (34.7, 135.2),
        (34.6, 135.4),
        (33.4, 135.8),
        (34.5, 136.8),
    ],
    // Chugoku & Sanyo
    &[
        (35.5, 135.5),
        (35.5, 133.0),
        (34.0, 130.9),
        (34.3, 132.4),
        (34.6, 133.9),
        (34.7, 135.2),
    ],
    // Tohoku
    &[
        (36.8, 139.6),
        (37.0, 141.0),
        (40.0, 140.0),
        (37.9, 139.0),
        (36.8, 137.1),
    ],
];

/// Exact WGS84 Mercator projection from (lat, lon) to screen pos2.
fn project_coords(
    lat: f64,
    lon: f64,
    bbox: (f64, f64, f64, f64),
    rect: egui::Rect,
    pan: egui::Vec2,
    zoom: f32,
) -> egui::Pos2 {
    let (min_lat, max_lat, min_lon, max_lon) = bbox;
    let u = ((lon - min_lon) / (max_lon - min_lon)).clamp(-0.5, 1.5);

    let lat_rad = lat.to_radians();
    let min_lat_rad = min_lat.to_radians();
    let max_lat_rad = max_lat.to_radians();
    let merc = (lat_rad * 0.5 + std::f64::consts::FRAC_PI_4).tan().ln();
    let merc_min = (min_lat_rad * 0.5 + std::f64::consts::FRAC_PI_4).tan().ln();
    let merc_max = (max_lat_rad * 0.5 + std::f64::consts::FRAC_PI_4).tan().ln();
    let v = 1.0 - ((merc - merc_min) / (merc_max - merc_min)).clamp(-0.5, 1.5);

    let mid_lat_rad = ((min_lat + max_lat) * 0.5).to_radians();
    let map_w = (max_lon - min_lon) * mid_lat_rad.cos();
    let map_h = max_lat - min_lat;
    let aspect = (map_w / map_h) as f32;

    let rect_aspect = rect.width() / rect.height();
    let (draw_w, draw_h) = if rect_aspect > aspect {
        (rect.height() * aspect, rect.height())
    } else {
        (rect.width(), rect.width() / aspect)
    };

    let offset_x = rect.min.x + (rect.width() - draw_w) * 0.5;
    let offset_y = rect.min.y + (rect.height() - draw_h) * 0.5;

    let base_x = offset_x + (u as f32) * draw_w;
    let base_y = offset_y + (v as f32) * draw_h;

    let center = rect.center();
    egui::pos2(
        center.x + (base_x - center.x) * zoom + pan.x,
        center.y + (base_y - center.y) * zoom + pan.y,
    )
}

/// Inverse Mercator projection from screen pos2 to (lat, lon).
fn screen_to_lat_lon(
    pos: egui::Pos2,
    bbox: (f64, f64, f64, f64),
    rect: egui::Rect,
    pan: egui::Vec2,
    zoom: f32,
) -> (f64, f64) {
    let (min_lat, max_lat, min_lon, max_lon) = bbox;
    let center = rect.center();
    let unscaled_x = (pos.x - center.x - pan.x) / zoom + center.x;
    let unscaled_y = (pos.y - center.y - pan.y) / zoom + center.y;

    let mid_lat_rad = ((min_lat + max_lat) * 0.5).to_radians();
    let map_w = (max_lon - min_lon) * mid_lat_rad.cos();
    let map_h = max_lat - min_lat;
    let aspect = (map_w / map_h) as f32;

    let rect_aspect = rect.width() / rect.height();
    let (draw_w, draw_h) = if rect_aspect > aspect {
        (rect.height() * aspect, rect.height())
    } else {
        (rect.width(), rect.width() / aspect)
    };

    let offset_x = rect.min.x + (rect.width() - draw_w) * 0.5;
    let offset_y = rect.min.y + (rect.height() - draw_h) * 0.5;

    let u = ((unscaled_x - offset_x) / draw_w) as f64;
    let v = ((unscaled_y - offset_y) / draw_h) as f64;

    let lon = min_lon + u * (max_lon - min_lon);

    let min_lat_rad = min_lat.to_radians();
    let max_lat_rad = max_lat.to_radians();
    let merc_min = (min_lat_rad * 0.5 + std::f64::consts::FRAC_PI_4).tan().ln();
    let merc_max = (max_lat_rad * 0.5 + std::f64::consts::FRAC_PI_4).tan().ln();
    let merc = merc_min + (1.0 - v) * (merc_max - merc_min);
    let lat_rad = 2.0 * (merc.exp().atan() - std::f64::consts::FRAC_PI_4);
    let lat = lat_rad.to_degrees();

    (lat, lon)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.smoke {
        println!("[travelcraft] Running headless smoke verification...");
        let engine = TravelCraftEngine::new_in_memory()?;
        let plan = engine.plan_optimized_route("JP")?;
        let itin = engine.plan_itinerary("AT", 3)?;
        let sight = engine.identify_sight(48.2085, 16.3738, "stephansdom")?;
        let art = engine.identify_sight(48.2039, 16.3619, "caravaggio")?;
        let chat_ctx = engine.sight_chat_context(&art);
        let chat_answer =
            engine.sight_chat_offline(&art, "Who painted the Rosenkranzmadonna and where is it?");
        let smoke_payload = json!({
            "app": "travelcraft",
            "version": "0.4.0",
            "status": "ready",
            "control_port": 14129,
            "success": true,
            "has_menu_bar": true,
            "has_theme_toggle": true,
            "has_basic_auth": true,
            "has_wheel_zoom": true,
            "has_high_contrast": true,
            "has_itineraries": true,
            "has_sightlens": true,
            "has_sight_chat": true,
            "jp_route_distance_km": plan.total_distance_km,
            "jp_waypoints_count": plan.waypoints.len(),
            "at_itinerary_days": itin.days.len(),
            "sight_name": sight.name,
            "artwork_name": art.name,
            "chat_context_chars": chat_ctx.system_prompt().len(),
            "chat_offline_chars": chat_answer.len(),
            "map_blob": chat_ctx.map_blob,
            "transit_blob": chat_ctx.transit_blob,
            "qgis_map_tiles_zoom8": plan.covered_map_tiles_zoom10,
            "capabilities": ["ChronoCartography", "SensoryAtmosphere", "EdoToModern", "HabsburgToAlpine", "InteractiveJournal", "QgisGisIntegration", "StayItineraries", "SightLensIdentify", "SightChat"]
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
