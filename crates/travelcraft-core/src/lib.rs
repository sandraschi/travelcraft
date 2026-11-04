//! TravelCraft Core Engine
//!
//! Integrates Baedeker 2030 chrono-cartography, offline vector tiles,
//! and sovereign travel journal storage backed by DatabaseCraft.

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use travelcraft_baedeker::{BaedekerCatalog, ChronoWaypoint, CountryEdition};
use databasecraft_store::DatabaseStore;
use qgiscraft_core::geo::{haversine_m, LonLat};
use qgiscraft_core::tiles::bbox_to_tiles;
use qgiscraft_core::geocode::{demo_search, Hit};

#[derive(Error, Debug)]
pub enum TravelCraftError {
    #[error("Store error: {0}")]
    Store(String),
    #[error("Country edition not found: {0}")]
    NotFound(String),
    #[error("GIS calculation error: {0}")]
    Gis(String),
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

pub struct TravelCraftEngine {
    store: Arc<Mutex<DatabaseStore>>,
}

impl TravelCraftEngine {
    pub fn new_in_memory() -> Result<Self, TravelCraftError> {
        let store = DatabaseStore::open_in_memory()
            .map_err(|e| TravelCraftError::Store(e.to_string()))?;
        
        let engine = Self {
            store: Arc::new(Mutex::new(store)),
        };
        engine.init_schema()?;
        Ok(engine)
    }

    fn init_schema(&self) -> Result<(), TravelCraftError> {
        let store_guard = self.store.lock()
            .map_err(|_| TravelCraftError::Store("Lock poison".to_string()))?;
        
        let schema_sql = "
            CREATE TABLE IF NOT EXISTS travel_journal (
                entry_id TEXT PRIMARY KEY,
                waypoint_id TEXT NOT NULL,
                timestamp_epoch INTEGER NOT NULL,
                user_reflection TEXT NOT NULL,
                sensory_notes TEXT NOT NULL,
                photo_uris_json TEXT NOT NULL
            );
        ";
        store_guard.execute(schema_sql)
            .map_err(|e| TravelCraftError::Store(e.to_string()))?;
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

    pub fn plan_optimized_route(&self, country_code: &str) -> Result<RouteOptimizationResult, TravelCraftError> {
        let country = self.get_country(country_code)?;
        let first_route = country.curated_routes.first()
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
            if wp.lat < min_lat { min_lat = wp.lat; }
            if wp.lat > max_lat { max_lat = wp.lat; }
            if wp.lon < min_lng { min_lng = wp.lon; }
            if wp.lon > max_lng { max_lng = wp.lon; }

            if i > 0 {
                let prev = &waypoints[i - 1];
                let p1 = LonLat::new(prev.lon, prev.lat).map_err(|e| TravelCraftError::Gis(e.to_string()))?;
                let p2 = LonLat::new(wp.lon, wp.lat).map_err(|e| TravelCraftError::Gis(e.to_string()))?;
                total_meters += haversine_m(p1, p2);
            }
        }

        let total_distance_km = (total_meters / 1000.0 * 10.0).round() / 10.0;
        // Travel velocity based on high-speed rail corridor (approx 110 km/h average with stops)
        let estimated_hours = (total_distance_km / 110.0 * 10.0).round() / 10.0;
        let carbon_footprint_kg = (total_distance_km * 0.038 * 10.0).round() / 10.0;

        // Fetch required QGISCraft map tile coordinates covering route bbox
        let tiles_zoom10 = bbox_to_tiles(min_lng, min_lat, max_lng, max_lat, 8)
            .unwrap_or_default();

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

    /// Geocode and enrich an itinerary location using QGISCraft geocoding engine
    pub fn geocode_waypoint(&self, query: &str) -> Result<Vec<Hit>, TravelCraftError> {
        demo_search(query, 5).map_err(|e| TravelCraftError::Gis(e.to_string()))
    }

    pub fn add_journal_entry(&self, entry: TravelJournalEntry) -> Result<(), TravelCraftError> {
        let store_guard = self.store.lock()
            .map_err(|_| TravelCraftError::Store("Lock poison".to_string()))?;
        
        let photos_json = serde_json::to_string(&entry.photo_uris)?;
        let sql = format!(
            "INSERT INTO travel_journal (entry_id, waypoint_id, timestamp_epoch, user_reflection, sensory_notes, photo_uris_json)
             VALUES ('{}', '{}', {}, '{}', '{}', '{}');",
            entry.entry_id,
            entry.waypoint_id,
            entry.timestamp_epoch,
            entry.user_reflection.replace('\'', "''"),
            entry.sensory_notes.replace('\'', "''"),
            photos_json.replace('\'', "''")
        );
        store_guard.execute(&sql)
            .map_err(|e| TravelCraftError::Store(e.to_string()))?;
        Ok(())
    }
}
