//! TravelCraft Core Engine
//!
//! Integrates Baedeker 2030 chrono-cartography, offline vector tiles,
//! and sovereign travel journal storage backed by DatabaseCraft.

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use travelcraft_baedeker::{BaedekerCatalog, ChronoWaypoint, CountryEdition};
use databasecraft_store::DatabaseStore;

#[derive(Error, Debug)]
pub enum TravelCraftError {
    #[error("Store error: {0}")]
    Store(String),
    #[error("Country edition not found: {0}")]
    NotFound(String),
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

        // High-precision great circle mock calculation for demonstration
        let total_distance_km = 480.5;
        let estimated_hours = 6.2;
        let carbon_footprint_kg = 18.4; // Electric Shinkansen / ÖBB Railjet baseline

        Ok(RouteOptimizationResult {
            country_code: country.country_code,
            total_distance_km,
            estimated_hours,
            waypoints,
            carbon_footprint_kg,
            chrono_span_years: chrono_span,
        })
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
