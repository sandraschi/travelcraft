//! TravelCraft Core Engine
//!
//! Integrates Baedeker 2030 chrono-cartography, offline vector tiles,
//! and sovereign travel journal storage backed by DatabaseCraft.

pub mod i18n;

use databasecraft_store::DatabaseStore;
use qgiscraft_core::geo::{haversine_m, LonLat};
use qgiscraft_core::geocode::{demo_search, Hit};
use qgiscraft_core::tiles::bbox_to_tiles;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use travelcraft_baedeker::{
    offline_chat_answer, transit_blob_for_sight, BaedekerCatalog, ChronoWaypoint, CountryEdition,
    SightChatContext, SightHit, StayItinerary,
};

#[derive(Error, Debug)]
pub enum TravelCraftError {
    #[error("Store error: {0}")]
    Store(String),
    #[error("Country edition not found: {0}")]
    NotFound(String),
    #[error("GIS calculation error: {0}")]
    Gis(String),
    #[error("No sight match for photo")]
    NoSight,
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
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

pub struct TravelCraftEngine {
    store: Arc<Mutex<DatabaseStore>>,
}

impl TravelCraftEngine {
    pub fn new_in_memory() -> Result<Self, TravelCraftError> {
        let store =
            DatabaseStore::open_in_memory().map_err(|e| TravelCraftError::Store(e.to_string()))?;

        let engine = Self {
            store: Arc::new(Mutex::new(store)),
        };
        engine.init_schema()?;
        Ok(engine)
    }

    fn init_schema(&self) -> Result<(), TravelCraftError> {
        let store_guard = self
            .store
            .lock()
            .map_err(|_| TravelCraftError::Store("Lock poison".to_string()))?;

        let schema_sql = "
            CREATE TABLE IF NOT EXISTS travel_journal (
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
        ";
        store_guard
            .execute(schema_sql)
            .map_err(|e| TravelCraftError::Store(e.to_string()))?;
        // Migration for older DBs without GPS columns.
        for ddl in [
            "ALTER TABLE travel_journal ADD COLUMN photo_lat REAL",
            "ALTER TABLE travel_journal ADD COLUMN photo_lon REAL",
            "ALTER TABLE travel_journal ADD COLUMN photo_gps_source TEXT",
        ] {
            let _ = store_guard.execute(ddl);
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
        use qgiscraft_core::tiles::{
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
        let entry_id = format!("SIGHT-{}", chrono::Utc::now().timestamp());
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

    pub fn add_journal_entry(&self, entry: TravelJournalEntry) -> Result<(), TravelCraftError> {
        let store_guard = self
            .store
            .lock()
            .map_err(|_| TravelCraftError::Store("Lock poison".to_string()))?;

        let photos_json = serde_json::to_string(&entry.photo_uris)?;
        let lat_sql = entry
            .photo_lat
            .map(|v| v.to_string())
            .unwrap_or("NULL".to_string());
        let lon_sql = entry
            .photo_lon
            .map(|v| v.to_string())
            .unwrap_or("NULL".to_string());
        let src_sql = entry
            .photo_gps_source
            .clone()
            .unwrap_or_default()
            .replace('\'', "''");
        let src_sql = if entry.photo_gps_source.is_some() {
            format!("'{}'", src_sql)
        } else {
            "NULL".to_string()
        };
        let sql = format!(
            "INSERT INTO travel_journal (entry_id, waypoint_id, timestamp_epoch, user_reflection, sensory_notes, photo_uris_json, photo_lat, photo_lon, photo_gps_source)
             VALUES ('{}', '{}', {}, '{}', '{}', '{}', {}, {}, {});",
            entry.entry_id,
            entry.waypoint_id,
            entry.timestamp_epoch,
            entry.user_reflection.replace('\'', "''"),
            entry.sensory_notes.replace('\'', "''"),
            photos_json.replace('\'', "''"),
            lat_sql,
            lon_sql,
            src_sql
        );
        store_guard
            .execute(&sql)
            .map_err(|e| TravelCraftError::Store(e.to_string()))?;
        Ok(())
    }
}
