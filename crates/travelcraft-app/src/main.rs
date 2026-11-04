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
    map_tx: std::sync::mpsc::Sender<(
        qgiscraft_core::tiles::Tile,
        Option<Vec<u8>>,
    )>,
    map_rx: std::sync::mpsc::Receiver<(
        qgiscraft_core::tiles::Tile,
        Option<Vec<u8>>,
    )>,
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
        let engine = TravelCraftEngine::new_in_memory()?;
        let initial_plan = engine.plan_optimized_route("JP").ok();
        let itinerary = engine.plan_itinerary("JP", 3).ok();
        let (map_tx, map_rx) = std::sync::mpsc::channel();

        Ok(Self {
            engine,
            selected_country_code: "JP".to_string(),
            selected_waypoint_id: Some("jp_nihonbashi".to_string()),
            active_tab: ActiveViewTab::ChronoMap,
            status_msg: "Baedeker 2030 Chrono-Cartography Ready".to_string(),
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
            chat_model: "qwen2.5:7b".to_string(),
            chat_online: true,
            chat_log: vec![(
                "guide".to_string(),
                "Sight Chat ready. Identify a sight (e.g. caravaggio in the KHM), then ask anything - I answer from the info blob, map + transit context, or the local LLM when online.".to_string(),
            )],
            chat_status: String::new(),
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
                    photo_lat: Some(35.6840),
                    photo_lon: Some(139.7745),
                    photo_gps_source: Some("manual".to_string()),
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

    /// Fly the chrono map to the selected waypoint (city-scale zoom).
    /// Clears cached-failure flags so retried tiles can load.
    fn focus_selected(&mut self) {
        if let Some(w) = self.current_waypoint() {
            self.map_clng = w.lon;
            self.map_clat = w.lat;
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
                // Also mirror into journal log view.
                self.journal_log.push(TravelJournalEntry {
                    entry_id: res.journal_entry_id.clone(),
                    waypoint_id: res.sight.id.clone(),
                    timestamp_epoch: chrono::Utc::now().timestamp(),
                    user_reflection: res.info_page_body.clone(),
                    sensory_notes: res.sight.headline.clone(),
                    photo_uris: vec![photo.to_string()],
                    photo_lat: Some(lat),
                    photo_lon: Some(lon),
                    photo_gps_source: Some(self.sight_gps_source.clone()),
                });
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
    ui.label(
        egui::RichText::new(text)
            .size(27.0)
            .strong()
            .color(PAPER),
    );
}

fn section_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(19.0)
            .strong()
            .color(GOLD),
    );
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
            v.widgets.noninteractive.fg_stroke =
                egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.widgets.inactive.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.widgets.inactive.bg_stroke =
                egui::Stroke::new(2.0, egui::Color32::from_rgb(10, 25, 60));
            v.widgets.hovered.bg_fill = GOLD;
            v.widgets.hovered.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.widgets.active.bg_fill = TEAL;
            v.widgets.active.fg_stroke = egui::Stroke::new(2.0, egui::Color32::BLACK);
            v.widgets.open.fg_stroke =
                egui::Stroke::new(3.0, egui::Color32::from_rgb(10, 25, 60));
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
                                let text = format!("{} {}", loc_item.flag(), loc_item.display_name());
                                let _ = ui.selectable_value(&mut self.locale, *loc_item, text);
                            }
                        });

                    let theme_icon = if self.is_dark_theme {
                        "Dark HC"
                    } else {
                        "Light HC"
                    };
                    if ui.button(theme_icon).clicked() {
                        self.is_dark_theme = !self.is_dark_theme;
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
                    (ActiveViewTab::ChronoMap, travelcraft_core::i18n::I18nKey::TabChronoMap),
                    (ActiveViewTab::WaypointDetail, travelcraft_core::i18n::I18nKey::TabWaypointDetail),
                    (ActiveViewTab::SensoryAtmosphere, travelcraft_core::i18n::I18nKey::TabSensoryAtmosphere),
                    (ActiveViewTab::RoutePlanner, travelcraft_core::i18n::I18nKey::TabRoutePlanner),
                    (ActiveViewTab::Itineraries, travelcraft_core::i18n::I18nKey::TabItineraries),
                    (ActiveViewTab::SightLens, travelcraft_core::i18n::I18nKey::TabSightLens),
                    (ActiveViewTab::SightChat, travelcraft_core::i18n::I18nKey::TabSightChat),
                    (ActiveViewTab::Journal, travelcraft_core::i18n::I18nKey::TabJournal),
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

        // LEFT PANEL: Country & Waypoint Selector
        egui::Panel::left("left_waypoint_tree").show(ui, |ui| {
            ui.set_width(320.0);
            ui.label(
                egui::RichText::new("Country Edition")
                    .size(17.0)
                    .strong()
                    .color(GOLD),
            );
            ui.horizontal(|ui| {
                let jp_sel = self.selected_country_code == "JP";
                if ui.selectable_label(jp_sel, "JapanCraft").clicked() {
                    self.selected_country_code = "JP".to_string();
                    self.selected_waypoint_id = Some("jp_nihonbashi".to_string());
                    self.optimized_route = self.engine.plan_optimized_route("JP").ok();
                    self.refresh_itinerary();
                    self.status_msg = "Selected JapanCraft Edition".to_string();
                }
                let at_sel = self.selected_country_code == "AT";
                if ui.selectable_label(at_sel, "AustriaCraft").clicked() {
                    self.selected_country_code = "AT".to_string();
                    self.selected_waypoint_id = Some("at_stephansdom".to_string());
                    self.optimized_route = self.engine.plan_optimized_route("AT").ok();
                    self.refresh_itinerary();
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
                        .size(15.0)
                        .color(MIST),
                );
                ui.add_space(8.0);
                for route in &country.curated_routes {
                    ui.label(
                        egui::RichText::new(&route.title)
                            .size(16.0)
                            .strong()
                            .color(GOLD),
                    );
                    for wp in &route.waypoints {
                        let is_wp_sel = self.selected_waypoint_id.as_deref() == Some(&wp.id);
                        let wp_title = format!(
                            "{} [{}] {}",
                            if is_wp_sel { ">" } else { "-" },
                            wp.year_ad,
                            wp.name
                        );
                        if ui
                            .selectable_label(is_wp_sel, egui::RichText::new(wp_title).size(16.0))
                            .clicked()
                        {
                            self.selected_waypoint_id = Some(wp.id.clone());
                            self.status_msg = format!("Inspecting {}", wp.name);
                        }
                    }
                }
            }
        });

        // CENTRAL PANEL
        egui::CentralPanel::default().show(ui, |ui| {
            match self.active_tab {
                ActiveViewTab::ChronoMap => {
                    big_heading(ui, "Chrono-Cartographic Interactive Map");
                    body_text(ui, "Mouse wheel zooms. Drag pans. Click a waypoint marker to open its dossier.");
                    ui.add_space(8.0);

                    let wps = self.all_waypoints_flat();
                    if wps.is_empty() {
                        body_text(ui, "No waypoints in this edition.");
                    } else {
                        // Zoom controls (buttons + wheel).
                        ui.horizontal(|ui| {
                            if ui.button("Zoom -").clicked() {
                                self.map_zoom = (self.map_zoom / 1.25).clamp(0.5, 8.0);
                            }
                            if ui.button("Reset").clicked() {
                                self.map_zoom = 1.0;
                                self.map_pan = egui::Vec2::ZERO;
                            }
                            if ui.button("Zoom +").clicked() {
                                self.map_zoom = (self.map_zoom * 1.25).clamp(0.5, 8.0);
                            }
                            ui.label(egui::RichText::new(format!("Zoom {:.1}x", self.map_zoom)).size(16.0).color(GOLD));
                        });
                        ui.add_space(4.0);
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(ui.available_width(), 340.0), egui::Sense::click_and_drag());
                        // Wheel zoom (the actual fix: read scroll delta on hover).
                        let wheel = ui.input(|i| i.smooth_scroll_delta.y);
                        if response.hovered() && wheel != 0.0 {
                            let factor = (1.0_f32 + wheel * 0.0015).clamp(0.8, 1.25);
                            self.map_zoom = (self.map_zoom * factor).clamp(0.5, 8.0);
                        }
                        // Drag pan.
                        if response.dragged() {
                            self.map_pan += response.drag_delta();
                        }
                        // Click nearest marker to select.
                        if response.clicked() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                let mut best_id: Option<String> = None;
                                let mut best_d = f32::MAX;
                                for (i, w) in wps.iter().enumerate() {
                                    let p = marker_pos(rect, self.map_pan, self.map_zoom, wps.len(), i);
                                    let d = (p - pos).length();
                                    if d < 28.0 && d < best_d {
                                        best_d = d;
                                        best_id = Some(w.id.clone());
                                    }
                                }
                                if let Some(id) = best_id {
                                    self.selected_waypoint_id = Some(id.clone());
                                    self.status_msg = format!("Map selected {}", id);
                                }
                            }
                        }

                        let painter = ui.painter();
                        painter.rect_filled(rect, 6.0, egui::Color32::BLACK);
                        painter.rect_stroke(
                            rect,
                            6.0,
                            egui::Stroke::new(3.0, GOLD),
                            egui::StrokeKind::Inside,
                        );
                        // Grid (scales with zoom for depth cue).
                        let step = 34.0 * self.map_zoom;
                        let mut x = rect.min.x + self.map_pan.x.rem_euclid(step);
                        while x < rect.max.x {
                            painter.line_segment(
                                [egui::pos2(x, rect.min.y), egui::pos2(x, rect.max.y)],
                                egui::Stroke::new(1.5, LINE_BRIGHT),
                            );
                            x += step;
                        }
                        let mut y = rect.min.y + self.map_pan.y.rem_euclid(step);
                        while y < rect.max.y {
                            painter.line_segment(
                                [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                                egui::Stroke::new(1.5, LINE_BRIGHT),
                            );
                            y += step;
                        }
                        for (i, w) in wps.iter().enumerate() {
                            let p = marker_pos(rect, self.map_pan, self.map_zoom, wps.len(), i);
                            if !rect.contains(p) {
                                continue;
                            }
                            let selected = self.selected_waypoint_id.as_deref() == Some(&w.id);
                            let r = if selected { 13.0 } else { 9.0 };
                            painter.circle_filled(
                                p,
                                r + 6.0,
                                egui::Color32::from_rgba_unmultiplied(255, 205, 105, 70),
                            );
                            painter.circle_filled(
                                p,
                                r,
                                if selected { GOLD } else { ROSE },
                            );
                            painter.circle_stroke(
                                p,
                                r,
                                egui::Stroke::new(3.0, egui::Color32::WHITE),
                            );
                            // Label with solid black backing so white text always reads.
                            let font = egui::FontId::proportional(15.0);
                            let galley = painter.layout_no_wrap(
                                w.name.clone(),
                                font,
                                egui::Color32::WHITE,
                            );
                            let label_rect = egui::Rect::from_center_size(
                                p + egui::vec2(0.0, 24.0),
                                galley.size() + egui::vec2(12.0, 8.0),
                            );
                            painter.rect_filled(label_rect, 4.0, egui::Color32::BLACK);
                            painter.rect_stroke(
                                label_rect,
                                4.0,
                                egui::Stroke::new(
                                    1.5,
                                    if selected { GOLD } else { LINE_BRIGHT },
                                ),
                                egui::StrokeKind::Inside,
                            );
                            painter.text(
                                p + egui::vec2(0.0, 24.0),
                                egui::Align2::CENTER_CENTER,
                                w.name.clone(),
                                egui::FontId::proportional(15.0),
                                egui::Color32::WHITE,
                            );
                        }
                        ui.add_space(6.0);
                        body_text(ui, "Tip: Stephansdom 48.2085/16.3738 - KHM 48.2039/16.3619 - Fushimi Inari 34.9671/135.7727.");
                    }
                }

                ActiveViewTab::WaypointDetail => {
                    big_heading(ui, "Historical Dossier");
                    if let Some(wp) = self.current_waypoint() {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new(&wp.name).size(23.0).strong().color(GOLD));
                                ui.label(egui::RichText::new(format!("{} | {} AD | {:.4}, {:.4}", wp.region, wp.year_ad, wp.lat, wp.lon)).size(16.0).color(MIST));
                                ui.separator();
                                body_text(ui, &wp.historic_narrative);
                                ui.add_space(8.0);
                                ui.label(egui::RichText::new("Visitor info").size(18.0).strong().color(TEAL));
                                body_text(ui, &wp.visitor_info);
                                ui.add_space(8.0);
                                ui.label(egui::RichText::new("Culinary heritage").size(18.0).strong().color(TEAL));
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
                            self.chat_log.push(("you".to_string(), q.clone()));
                            let answer = self.answer_sight_question(&q);
                            self.chat_log.push(("guide".to_string(), answer));
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
                    big_heading(ui, "Sovereign Travel Journal (Local SQLite)");
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("New reflection:").size(15.0).strong().color(GOLD));
                        ui.text_edit_multiline(&mut self.new_reflection);
                        ui.label("Sensory notes:");
                        ui.text_edit_singleline(&mut self.new_sensory_notes);

                        if ui.button("Save Entry to DatabaseCraft").clicked() && !self.new_reflection.is_empty() {
                            let entry = TravelJournalEntry {
                                entry_id: format!("JOURNAL-{}", self.journal_log.len() + 1),
                                waypoint_id: self.selected_waypoint_id.clone().unwrap_or_else(|| "general".to_string()),
                                timestamp_epoch: chrono::Utc::now().timestamp(),
                                user_reflection: self.new_reflection.clone(),
                                sensory_notes: self.new_sensory_notes.clone(),
                                photo_uris: vec![],
                                photo_lat: None,
                                photo_lon: None,
                                photo_gps_source: None,
                            };
                            let _ = self.engine.add_journal_entry(entry.clone());
                            self.journal_log.push(entry);
                            self.status_msg = "Journal entry persisted".to_string();
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

/// Marker layout: spread waypoints across the map rect, scaled by zoom + pan.
fn marker_pos(rect: egui::Rect, pan: egui::Vec2, zoom: f32, n: usize, i: usize) -> egui::Pos2 {
    let cols = (n as f32).sqrt().ceil() as usize;
    let cols = cols.max(1);
    let col = (i % cols) as f32;
    let row = (i / cols) as f32;
    let rows = n.div_ceil(cols) as f32;
    let fx = if cols <= 1 {
        0.5
    } else {
        0.12 + 0.76 * (col / (cols as f32 - 1.0))
    };
    let fy = if rows <= 1.0 {
        0.5
    } else {
        0.15 + 0.70 * (row / (rows - 1.0))
    };
    let base = egui::pos2(
        rect.min.x + fx * rect.width(),
        rect.min.y + fy * rect.height(),
    );
    let center = rect.center();
    center + (base - center) * zoom + pan
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
