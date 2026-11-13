//! TravelCraft Core Engine
//!
//! Integrates Baedeker 2030 chrono-cartography, offline vector tiles,
//! and a durable travel journal (SQLite, WAL, synchronous=FULL).

pub mod fleet;
pub mod i18n;
pub mod nearby;

use giscraft_core::geo::{haversine_m, LonLat};
use giscraft_core::geocode::{demo_search, Hit};
use giscraft_core::tiles::bbox_to_tiles;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use thiserror::Error;
use travelcraft_baedeker::{
    offline_chat_answer, transit_blob_for_sight, BaedekerCatalog, ChronoWaypoint, CountryEdition,
    RegionPack, SightChatContext, SightHit, StayItinerary,
};

#[derive(Error, Debug)]
pub enum TravelCraftError {
    #[error("Store error: {0}")]
    Store(String),
    #[error("Country edition not found: {0}")]
    NotFound(String),
    #[error("GIS calculation error: {0}")]
    Gis(String),
    #[error("Invalid input: {0}")]
    Invalid(String),
    #[error("Offline or upstream unavailable: {0}")]
    Offline(String),
    #[error("Unexpected upstream response: {0}")]
    Protocol(String),
    #[error("No sight match for photo")]
    NoSight,
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl TravelCraftError {
    /// Stable machine-readable error class for front ends (`error_type`).
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Store(_) => "store",
            Self::NotFound(_) | Self::NoSight => "not_found",
            Self::Gis(_) => "gis",
            Self::Serialization(_) => "serialize",
            Self::Invalid(_) => "invalid",
            Self::Offline(_) => "offline",
            Self::Protocol(_) => "protocol",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelJournalEntry {
    pub entry_id: String,
    pub waypoint_id: String,
    pub timestamp_epoch: i64,
    pub user_reflection: String,
    pub sensory_notes: String,
    pub photo_uris: Vec<String>,
    /// Optional GPS fix attached to the photo (WGS84).
    pub photo_lat: Option<f64>,
    /// Optional GPS fix attached to the photo (WGS84).
    pub photo_lon: Option<f64>,
    /// Where the GPS came from: windows-geolocation | iphone-hotspot-manual | immich-exif | manual.
    pub photo_gps_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteOptimizationResult {
    pub country_code: String,
    pub total_distance_km: f64,
    pub estimated_hours: f64,
    pub waypoints: Vec<ChronoWaypoint>,
    pub carbon_footprint_kg: f64,
    pub chrono_span_years: i32,
    pub covered_map_tiles_zoom10: usize,
}

/// Result of the SightLens pipeline: photo -> sight -> info page payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SightLensResult {
    pub sight: SightHit,
    pub journal_entry_id: String,
    pub info_page_title: String,
    pub info_page_body: String,
}

const MAX_TEXT_CHARS: usize = 16_000;
const MAX_FIELD_CHARS: usize = 4_096;
const MAX_PHOTOS: usize = 32;
const CHAT_KEEP: i64 = 1_000;

fn db_err<E: std::fmt::Display>(e: E) -> TravelCraftError {
    TravelCraftError::Store(e.to_string())
}

/// Where durable state lives: `TRAVELCRAFT_DATA`, else `%LOCALAPPDATA%\travelcraft`,
/// else `~/.travelcraft`, else `./travelcraft-data`.
pub fn data_dir() -> PathBuf {
    if let Some(v) = std::env::var_os("TRAVELCRAFT_DATA").filter(|v| !v.is_empty()) {
        return PathBuf::from(v);
    }
    if let Some(v) = std::env::var_os("LOCALAPPDATA").filter(|v| !v.is_empty()) {
        return PathBuf::from(v).join("travelcraft");
    }
    if let Some(v) = std::env::var_os("HOME").filter(|v| !v.is_empty()) {
        return PathBuf::from(v).join(".travelcraft");
    }
    PathBuf::from("travelcraft-data")
}

/// The engine. File-backed engines persist the journal, chat history and
/// settings across restarts and crashes (SQLite WAL, synchronous=FULL).
/// `new_in_memory` is for tests and the headless smoke run only.
pub struct TravelCraftEngine {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
    seq: AtomicU64,
}

impl TravelCraftEngine {
    pub fn new_in_memory() -> Result<Self, TravelCraftError> {
        let conn = Connection::open_in_memory().map_err(db_err)?;
        Self::finish(conn, None)
    }

    /// Open (creating if needed) a file-backed engine at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, TravelCraftError> {
        let path = path.as_ref().to_path_buf();
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(db_err)?;
        }
        let conn = Connection::open(&path).map_err(db_err)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = FULL;
             PRAGMA busy_timeout = 5000;",
        )
        .map_err(db_err)?;
        Self::finish(conn, Some(path))
    }

    /// Open the default database `<data_dir>/travelcraft.db`.
    pub fn open_default() -> Result<Self, TravelCraftError> {
        Self::open(data_dir().join("travelcraft.db"))
    }

    fn finish(conn: Connection, path: Option<PathBuf>) -> Result<Self, TravelCraftError> {
        let engine = Self {
            conn: Mutex::new(conn),
            path,
            seq: AtomicU64::new(0),
        };
        engine.init_schema()?;
        Ok(engine)
    }

    fn lock(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn is_persistent(&self) -> bool {
        self.path.is_some()
    }

    pub fn db_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    fn init_schema(&self) -> Result<(), TravelCraftError> {
        let conn = self.lock();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS travel_journal (
                entry_id TEXT PRIMARY KEY,
                waypoint_id TEXT NOT NULL,
                timestamp_epoch INTEGER NOT NULL,
                user_reflection TEXT NOT NULL,
                sensory_notes TEXT NOT NULL,
                photo_uris_json TEXT NOT NULL,
                photo_lat REAL,
                photo_lon REAL,
                photo_gps_source TEXT
            );
            CREATE TABLE IF NOT EXISTS chat_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sight_id TEXT NOT NULL,
                role TEXT NOT NULL,
                text TEXT NOT NULL,
                ts INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS nearby_cache (
                cache_key TEXT PRIMARY KEY,
                fetched_at INTEGER NOT NULL,
                payload TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS region_packs (
                pack_id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                country_code TEXT NOT NULL,
                version TEXT NOT NULL,
                payload TEXT NOT NULL
            );",
        )
        .map_err(db_err)?;
        // Migration for older DBs without GPS columns; a duplicate-column error is expected.
        for ddl in [
            "ALTER TABLE travel_journal ADD COLUMN photo_lat REAL",
            "ALTER TABLE travel_journal ADD COLUMN photo_lon REAL",
            "ALTER TABLE travel_journal ADD COLUMN photo_gps_source TEXT",
        ] {
            let _ = conn.execute(ddl, []);
        }
        Ok(())
    }
    pub fn list_countries(&self) -> Vec<CountryEdition> {
        vec![
            BaedekerCatalog::get_japan_craft(),
            BaedekerCatalog::get_austria_craft(),
        ]
    }

    pub fn get_country(&self, country_code: &str) -> Result<CountryEdition, TravelCraftError> {
        match country_code.to_uppercase().as_str() {
            "JP" | "JAPAN" => Ok(BaedekerCatalog::get_japan_craft()),
            "AT" | "AUSTRIA" => Ok(BaedekerCatalog::get_austria_craft()),
            other => Err(TravelCraftError::NotFound(other.to_string())),
        }
    }

    /// Retrieve sovereign knowledge tree (hierarchical categorized intel).
    pub fn get_knowledge_tree(&self, country_code: &str) -> travelcraft_baedeker::KnowledgeTree {
        BaedekerCatalog::get_knowledge_tree(country_code)
    }

    /// Import or update an offline region pack into durable storage.
    pub fn import_region_pack(&self, pack: &RegionPack) -> Result<(), TravelCraftError> {
        let conn = self.lock();
        let payload = pack.to_json_string()?;
        conn.execute(
            "INSERT OR REPLACE INTO region_packs (pack_id, title, country_code, version, payload)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &pack.pack_id,
                &pack.title,
                &pack.country_code,
                &pack.version,
                &payload
            ],
        )
        .map_err(db_err)?;
        Ok(())
    }

    /// List all dynamically imported region packs.
    pub fn list_region_packs(&self) -> Result<Vec<RegionPack>, TravelCraftError> {
        let conn = self.lock();
        let mut stmt = conn
            .prepare("SELECT payload FROM region_packs ORDER BY title ASC")
            .map_err(db_err)?;
        let rows = stmt
            .query_map([], |row| {
                let payload: String = row.get(0)?;
                Ok(payload)
            })
            .map_err(db_err)?;
        let mut out = Vec::new();
        for r in rows {
            let payload = r.map_err(db_err)?;
            if let Ok(pack) = RegionPack::from_json_str(&payload) {
                out.push(pack);
            }
        }
        Ok(out)
    }

    /// Load any `.json` region packs present in `dir`.
    pub fn load_region_packs_from_dir(&self, dir: &Path) -> Result<usize, TravelCraftError> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(0);
        }
        let entries = std::fs::read_dir(dir).map_err(db_err)?;
        let mut count = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(pack) = RegionPack::from_json_str(&content) {
                        if self.import_region_pack(&pack).is_ok() {
                            count += 1;
                        }
                    }
                }
            }
        }
        Ok(count)
    }

    pub fn plan_optimized_route(
        &self,
        country_code: &str,
    ) -> Result<RouteOptimizationResult, TravelCraftError> {
        let country = self.get_country(country_code)?;
        let first_route = country
            .curated_routes
            .first()
            .ok_or_else(|| TravelCraftError::NotFound("No routes configured".to_string()))?;

        let waypoints = first_route.waypoints.clone();
        let min_year = waypoints.iter().map(|w| w.year_ad).min().unwrap_or(2026);
        let max_year = waypoints.iter().map(|w| w.year_ad).max().unwrap_or(2026);
        let chrono_span = (max_year - min_year).abs();

        // Calculate exact geodesic distance across legs using QGISCraft core haversine
        let mut total_meters = 0.0;
        let mut min_lat = 90.0_f64;
        let mut max_lat = -90.0_f64;
        let mut min_lng = 180.0_f64;
        let mut max_lng = -180.0_f64;

        for (i, wp) in waypoints.iter().enumerate() {
            if wp.lat < min_lat {
                min_lat = wp.lat;
            }
            if wp.lat > max_lat {
                max_lat = wp.lat;
            }
            if wp.lon < min_lng {
                min_lng = wp.lon;
            }
            if wp.lon > max_lng {
                max_lng = wp.lon;
            }

            if i > 0 {
                let prev = &waypoints[i - 1];
                let p1 = LonLat::new(prev.lon, prev.lat)
                    .map_err(|e| TravelCraftError::Gis(e.to_string()))?;
                let p2 = LonLat::new(wp.lon, wp.lat)
                    .map_err(|e| TravelCraftError::Gis(e.to_string()))?;
                total_meters += haversine_m(p1, p2);
            }
        }

        let total_distance_km = (total_meters / 1000.0 * 10.0).round() / 10.0;
        // Travel velocity based on high-speed rail corridor (approx 110 km/h average with stops)
        let estimated_hours = (total_distance_km / 110.0 * 10.0).round() / 10.0;
        let carbon_footprint_kg = (total_distance_km * 0.038 * 10.0).round() / 10.0;

        // Fetch required QGISCraft map tile coordinates covering route bbox
        let tiles_zoom10 = bbox_to_tiles(min_lng, min_lat, max_lng, max_lat, 8).unwrap_or_default();

        Ok(RouteOptimizationResult {
            country_code: country.country_code,
            total_distance_km,
            estimated_hours,
            waypoints,
            carbon_footprint_kg,
            chrono_span_years: chrono_span,
            covered_map_tiles_zoom10: tiles_zoom10.len(),
        })
    }

    /// Stay-length itinerary: 1/2/3/5/7/14-day Baedeker plans.
    pub fn plan_itinerary(
        &self,
        country_code: &str,
        days: u32,
    ) -> Result<StayItinerary, TravelCraftError> {
        // Validate edition first so unknown codes still error.
        let _ = self.get_country(country_code)?;
        Ok(BaedekerCatalog::itinerary_for_stay(country_code, days))
    }

    /// Geocode and enrich an itinerary location using QGISCraft geocoding engine
    pub fn geocode_waypoint(&self, query: &str) -> Result<Vec<Hit>, TravelCraftError> {
        demo_search(query, 5).map_err(|e| TravelCraftError::Gis(e.to_string()))
    }

    /// SightLens identify: GPS + hint -> SightHit (landmark / museum / artwork).
    pub fn identify_sight(
        &self,
        lat: f64,
        lon: f64,
        hint: &str,
    ) -> Result<SightHit, TravelCraftError> {
        BaedekerCatalog::identify_sight(lat, lon, hint).ok_or(TravelCraftError::NoSight)
    }

    /// Map blob from qgiscraft: slippy tiles + scale + OSM template around the sight.
    pub fn map_blob_for_sight(&self, lat: f64, lon: f64) -> String {
        use giscraft_core::tiles::{
            lnglat_to_tile_float, meters_per_pixel, nice_scale_bar, tile_url, Tile,
        };
        let zoom: u8 = 16;
        let (fx, fy) = lnglat_to_tile_float(lon, lat, zoom);
        let tile = Tile {
            z: zoom,
            x: fx.floor() as i64,
            y: fy.floor() as i64,
        };
        let url = tile_url(tile);
        let mpp = meters_per_pixel(lat, zoom as f32);
        let (scale_m, scale_px) = nice_scale_bar(mpp, 120.0);
        let span = 0.004;
        let nearby = bbox_to_tiles(lon - span, lat - span, lon + span, lat + span, 8)
            .map(|v| v.len())
            .unwrap_or(0);
        format!(
            "MAP qgiscraft Z{zoom} tile {}/{}/{} ({lat:.5},{lon:.5}) OSM: {url} | scale {scale_m:.0}m = {scale_px:.0}px ({mpp:.1} m/px) | Z8 tiles in bbox: {nearby}. Offline vector fallback + live tiles via qgiscraft :11254.",
            tile.x, tile.y, tile.z
        )
    }

    /// Assemble grounded chat context: info blob + map + transit + sources.
    pub fn sight_chat_context(&self, sight: &SightHit) -> SightChatContext {
        let map_blob = self.map_blob_for_sight(sight.lat, sight.lon);
        let transit_blob = transit_blob_for_sight(&sight.id);
        SightChatContext {
            sight_id: sight.id.clone(),
            sight_name: sight.name.clone(),
            info_body: format!("{} {}", sight.headline, sight.body),
            practical: sight.practical.clone(),
            map_blob,
            transit_blob,
            sources: travelcraft_baedeker::SOURCES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    /// Offline grounded answer (no network). Online LLM callers use
    /// `sight_chat_context(..).system_prompt()` + question as the prompt.
    pub fn sight_chat_offline(&self, sight: &SightHit, question: &str) -> String {
        let ctx = self.sight_chat_context(sight);
        offline_chat_answer(&ctx, question)
    }

    /// Full SightLens pipeline: identify, persist journal with GPS + photo, render info page.
    pub fn sightlens_capture(
        &self,
        photo_uri: &str,
        lat: Option<f64>,
        lon: Option<f64>,
        gps_source: &str,
        hint: &str,
    ) -> Result<SightLensResult, TravelCraftError> {
        // Default to Stephansdom when no GPS (laptop without sensor).
        let (plat, plon) = match (lat, lon) {
            (Some(a), Some(o)) => (a, o),
            _ => (48.2085, 16.3738),
        };
        let sight = self.identify_sight(plat, plon, hint)?;
        let entry_id = self.new_entry_id("SIGHT");
        let entry = TravelJournalEntry {
            entry_id: entry_id.clone(),
            waypoint_id: sight.id.clone(),
            timestamp_epoch: chrono::Utc::now().timestamp(),
            user_reflection: format!("SightLens capture: {} ({})", sight.name, photo_uri),
            sensory_notes: sight.headline.clone(),
            photo_uris: vec![photo_uri.to_string()],
            photo_lat: Some(plat),
            photo_lon: Some(plon),
            photo_gps_source: Some(gps_source.to_string()),
        };
        self.add_journal_entry(entry)?;
        let info_page_title = format!("{} - Baedeker Dossier", sight.name);
        let info_page_body = format!(
            "{}\n\n{}\n\nPractical: {}\nPhoto: {} (GPS {:.5},{:.5} via {})",
            sight.headline, sight.body, sight.practical, photo_uri, plat, plon, gps_source
        );
        Ok(SightLensResult {
            sight,
            journal_entry_id: entry_id,
            info_page_title,
            info_page_body,
        })
    }

    /// Unique, sortable id: `<PREFIX>-<unix millis>-<process sequence>`.
    pub fn new_entry_id(&self, prefix: &str) -> String {
        let n = self.seq.fetch_add(1, Ordering::Relaxed);
        format!("{prefix}-{}-{n:03}", chrono::Utc::now().timestamp_millis())
    }

    fn check_text(name: &str, s: &str, max: usize, required: bool) -> Result<(), TravelCraftError> {
        if required && s.trim().is_empty() {
            return Err(TravelCraftError::Invalid(format!("{name} is empty")));
        }
        if s.chars().count() > max {
            return Err(TravelCraftError::Invalid(format!(
                "{name} is longer than {max} characters"
            )));
        }
        Ok(())
    }

    fn validate_entry(entry: &TravelJournalEntry) -> Result<(), TravelCraftError> {
        Self::check_text("entry_id", &entry.entry_id, MAX_FIELD_CHARS, true)?;
        Self::check_text("waypoint_id", &entry.waypoint_id, MAX_FIELD_CHARS, true)?;
        Self::check_text("reflection", &entry.user_reflection, MAX_TEXT_CHARS, true)?;
        Self::check_text("sensory_notes", &entry.sensory_notes, MAX_TEXT_CHARS, false)?;
        if entry.photo_uris.len() > MAX_PHOTOS {
            return Err(TravelCraftError::Invalid(format!(
                "more than {MAX_PHOTOS} photos"
            )));
        }
        for uri in &entry.photo_uris {
            Self::check_text("photo uri", uri, MAX_FIELD_CHARS, true)?;
        }
        if let Some(src) = &entry.photo_gps_source {
            Self::check_text("gps source", src, 64, false)?;
        }
        if let Some(lat) = entry.photo_lat {
            if !lat.is_finite() || !(-90.0..=90.0).contains(&lat) {
                return Err(TravelCraftError::Invalid("latitude out of range".into()));
            }
        }
        if let Some(lon) = entry.photo_lon {
            if !lon.is_finite() || !(-180.0..=180.0).contains(&lon) {
                return Err(TravelCraftError::Invalid("longitude out of range".into()));
            }
        }
        Ok(())
    }

    /// Append a journal entry. Parameterised SQL; a duplicate `entry_id` is an error.
    pub fn add_journal_entry(&self, entry: TravelJournalEntry) -> Result<(), TravelCraftError> {
        Self::validate_entry(&entry)?;
        let photos_json = serde_json::to_string(&entry.photo_uris)?;
        self.lock()
            .execute(
                "INSERT INTO travel_journal (entry_id, waypoint_id, timestamp_epoch, user_reflection, sensory_notes, photo_uris_json, photo_lat, photo_lon, photo_gps_source)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    entry.entry_id,
                    entry.waypoint_id,
                    entry.timestamp_epoch,
                    entry.user_reflection,
                    entry.sensory_notes,
                    photos_json,
                    entry.photo_lat,
                    entry.photo_lon,
                    entry.photo_gps_source,
                ],
            )
            .map_err(db_err)?;
        Ok(())
    }

    /// Newest-first journal entries, at most `limit` (clamped to 1..=1000).
    pub fn list_journal(&self, limit: usize) -> Result<Vec<TravelJournalEntry>, TravelCraftError> {
        let limit = i64::try_from(limit.clamp(1, 1000)).unwrap_or(1000);
        let conn = self.lock();
        let mut stmt = conn
            .prepare(
                "SELECT entry_id, waypoint_id, timestamp_epoch, user_reflection, sensory_notes, photo_uris_json, photo_lat, photo_lon, photo_gps_source
                 FROM travel_journal ORDER BY timestamp_epoch DESC, rowid DESC LIMIT ?1",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map(params![limit], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, Option<f64>>(6)?,
                    r.get::<_, Option<f64>>(7)?,
                    r.get::<_, Option<String>>(8)?,
                ))
            })
            .map_err(db_err)?;
        let mut out = Vec::new();
        for row in rows {
            let (
                entry_id,
                waypoint_id,
                timestamp_epoch,
                user_reflection,
                sensory_notes,
                photos,
                photo_lat,
                photo_lon,
                photo_gps_source,
            ) = row.map_err(db_err)?;
            out.push(TravelJournalEntry {
                entry_id,
                waypoint_id,
                timestamp_epoch,
                user_reflection,
                sensory_notes,
                photo_uris: serde_json::from_str(&photos)?,
                photo_lat,
                photo_lon,
                photo_gps_source,
            });
        }
        Ok(out)
    }

    pub fn journal_count(&self) -> Result<usize, TravelCraftError> {
        let n: i64 = self
            .lock()
            .query_row("SELECT COUNT(*) FROM travel_journal", [], |r| r.get(0))
            .map_err(db_err)?;
        Ok(usize::try_from(n).unwrap_or(0))
    }

    /// Append one chat line (role is "you" or "guide"). Keeps the newest 1000 lines.
    pub fn chat_append(
        &self,
        sight_id: &str,
        role: &str,
        text: &str,
    ) -> Result<(), TravelCraftError> {
        Self::check_text("sight_id", sight_id, MAX_FIELD_CHARS, false)?;
        Self::check_text("role", role, 32, true)?;
        Self::check_text("chat text", text, MAX_TEXT_CHARS, true)?;
        let conn = self.lock();
        conn.execute(
            "INSERT INTO chat_log (sight_id, role, text, ts) VALUES (?1, ?2, ?3, ?4)",
            params![sight_id, role, text, chrono::Utc::now().timestamp()],
        )
        .map_err(db_err)?;
        conn.execute(
            "DELETE FROM chat_log WHERE id <= (SELECT MAX(id) FROM chat_log) - ?1",
            params![CHAT_KEEP],
        )
        .map_err(db_err)?;
        Ok(())
    }

    /// The last `limit` chat lines, oldest first, as `(role, text)`.
    pub fn chat_recent(&self, limit: usize) -> Result<Vec<(String, String)>, TravelCraftError> {
        let limit = i64::try_from(limit.clamp(1, 1000)).unwrap_or(1000);
        let conn = self.lock();
        let mut stmt = conn
            .prepare(
                "SELECT role, text FROM (SELECT id, role, text FROM chat_log ORDER BY id DESC LIMIT ?1) ORDER BY id ASC",
            )
            .map_err(db_err)?;
        let rows = stmt
            .query_map(params![limit], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(db_err)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(db_err)?);
        }
        Ok(out)
    }

    pub fn setting_get(&self, key: &str) -> Result<Option<String>, TravelCraftError> {
        use rusqlite::OptionalExtension;
        self.lock()
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_err)
    }

    pub fn setting_set(&self, key: &str, value: &str) -> Result<(), TravelCraftError> {
        Self::check_text("setting key", key, 128, true)?;
        Self::check_text("setting value", value, MAX_FIELD_CHARS, false)?;
        self.lock()
            .execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )
            .map_err(db_err)?;
        Ok(())
    }
}
