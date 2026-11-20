//! Universal Travel Notebook: Trips + Scoped Entries + Markdown Travelogue Export.
//!
//! Personal notes never live inside reference cartridge files. One universal
//! SQLite store holds `trips` and `notebook_entries`, so trips can cross
//! cartridge boundaries (e.g. Vienna -> Nightjet -> Venice -> Padua -> Vienna)
//! while still offering dedicated views per Trip, City Cartridge, or Country.

use crate::{
    db_err, TravelCraftEngine, TravelCraftError, MAX_FIELD_CHARS, MAX_PHOTOS, MAX_TEXT_CHARS,
};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Write as _;

const MAX_TAGS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TripStatus {
    Planned,
    Active,
    Completed,
}

impl TripStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Active => "active",
            Self::Completed => "completed",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "planned" => Some(Self::Planned),
            "active" => Some(Self::Active),
            "completed" => Some(Self::Completed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trip {
    pub trip_id: String,
    pub title: String,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub cartridge_ids: Vec<String>,
    pub status: TripStatus,
    #[serde(default)]
    pub notes: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTrip {
    #[serde(default)]
    pub trip_id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub cartridge_ids: Vec<String>,
    #[serde(default = "default_trip_status")]
    pub status: TripStatus,
    #[serde(default)]
    pub notes: String,
}

fn default_trip_status() -> TripStatus {
    TripStatus::Planned
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TripPatch {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub cartridge_ids: Option<Vec<String>>,
    #[serde(default)]
    pub status: Option<TripStatus>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotebookKind {
    Reflection,
    Photo,
    Food,
    Logistics,
    Expense,
    Idea,
}

impl NotebookKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reflection => "reflection",
            Self::Photo => "photo",
            Self::Food => "food",
            Self::Logistics => "logistics",
            Self::Expense => "expense",
            Self::Idea => "idea",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "reflection" => Some(Self::Reflection),
            "photo" => Some(Self::Photo),
            "food" => Some(Self::Food),
            "logistics" => Some(Self::Logistics),
            "expense" => Some(Self::Expense),
            "idea" => Some(Self::Idea),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookEntry {
    pub entry_id: String,
    #[serde(default)]
    pub trip_id: Option<String>,
    #[serde(default)]
    pub cartridge_id: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub waypoint_id: Option<String>,
    pub kind: NotebookKind,
    #[serde(default)]
    pub day_index: Option<u32>,
    pub timestamp_epoch: i64,
    #[serde(default)]
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub sensory_notes: String,
    #[serde(default)]
    pub photo_uris: Vec<String>,
    #[serde(default)]
    pub lat: Option<f64>,
    #[serde(default)]
    pub lon: Option<f64>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewNotebookEntry {
    #[serde(default)]
    pub trip_id: Option<String>,
    #[serde(default)]
    pub cartridge_id: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub waypoint_id: Option<String>,
    #[serde(default = "default_note_kind")]
    pub kind: NotebookKind,
    #[serde(default)]
    pub day_index: Option<u32>,
    #[serde(default)]
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub sensory_notes: String,
    #[serde(default)]
    pub photo_uris: Vec<String>,
    #[serde(default)]
    pub lat: Option<f64>,
    #[serde(default)]
    pub lon: Option<f64>,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_note_kind() -> NotebookKind {
    NotebookKind::Reflection
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotebookPatch {
    #[serde(default)]
    pub trip_id: Option<String>,
    #[serde(default)]
    pub cartridge_id: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub waypoint_id: Option<String>,
    #[serde(default)]
    pub kind: Option<NotebookKind>,
    #[serde(default)]
    pub day_index: Option<u32>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub sensory_notes: Option<String>,
    #[serde(default)]
    pub photo_uris: Option<Vec<String>>,
    #[serde(default)]
    pub lat: Option<f64>,
    #[serde(default)]
    pub lon: Option<f64>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotebookQuery {
    #[serde(default)]
    pub trip_id: Option<String>,
    #[serde(default)]
    pub cartridge_id: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub waypoint_id: Option<String>,
    #[serde(default)]
    pub kind: Option<NotebookKind>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

fn validate_coords(lat: Option<f64>, lon: Option<f64>) -> Result<(), TravelCraftError> {
    if let Some(la) = lat {
        if !la.is_finite() || !(-90.0..=90.0).contains(&la) {
            return Err(TravelCraftError::Invalid("latitude out of range".into()));
        }
    }
    if let Some(lo) = lon {
        if !lo.is_finite() || !(-180.0..=180.0).contains(&lo) {
            return Err(TravelCraftError::Invalid("longitude out of range".into()));
        }
    }
    Ok(())
}

fn normalize_country(cc: Option<&str>) -> Result<Option<String>, TravelCraftError> {
    let Some(raw) = cc else {
        return Ok(None);
    };
    let trimmed = raw.trim().to_ascii_uppercase();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.len() != 2 || !trimmed.bytes().all(|b| b.is_ascii_uppercase()) {
        return Err(TravelCraftError::Invalid(format!(
            "country_code '{raw}' must be 2 ASCII letters"
        )));
    }
    Ok(Some(trimmed))
}

fn validate_tags(tags: &[String]) -> Result<Vec<String>, TravelCraftError> {
    if tags.len() > MAX_TAGS {
        return Err(TravelCraftError::Invalid(format!(
            "more than {MAX_TAGS} tags"
        )));
    }
    let mut cleaned = Vec::with_capacity(tags.len());
    for t in tags {
        let s = t.trim().to_ascii_lowercase();
        if s.is_empty() {
            continue;
        }
        if s.chars().count() > 64 {
            return Err(TravelCraftError::Invalid("tag longer than 64 chars".into()));
        }
        if !cleaned.contains(&s) {
            cleaned.push(s);
        }
    }
    Ok(cleaned)
}

impl TravelCraftEngine {
    /// Create a new Trip container in the universal notebook.
    pub fn trip_create(&self, req: &NewTrip) -> Result<Trip, TravelCraftError> {
        Self::check_text("trip title", &req.title, MAX_FIELD_CHARS, true)?;
        Self::check_text("trip notes", &req.notes, MAX_TEXT_CHARS, false)?;
        let trip_id = match &req.trip_id {
            Some(id) if !id.trim().is_empty() => {
                Self::check_text("trip_id", id, 128, true)?;
                id.trim().to_string()
            }
            _ => self.new_entry_id("TRIP"),
        };
        let now = chrono::Utc::now().timestamp();
        let cart_json = serde_json::to_string(&req.cartridge_ids)?;
        let conn = self.lock();
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM trips WHERE trip_id = ?1",
                params![&trip_id],
                |_| Ok(true),
            )
            .optional()
            .map_err(db_err)?
            .unwrap_or(false);
        if exists {
            return Err(TravelCraftError::Invalid(format!(
                "trip '{trip_id}' already exists"
            )));
        }
        conn.execute(
            "INSERT INTO trips (trip_id, title, start_date, end_date, cartridge_ids_json, status, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &trip_id,
                req.title.trim(),
                &req.start_date,
                &req.end_date,
                &cart_json,
                req.status.as_str(),
                &req.notes,
                now,
                now,
            ],
        )
        .map_err(db_err)?;
        Ok(Trip {
            trip_id,
            title: req.title.trim().to_string(),
            start_date: req.start_date.clone(),
            end_date: req.end_date.clone(),
            cartridge_ids: req.cartridge_ids.clone(),
            status: req.status,
            notes: req.notes.clone(),
            created_at: now,
            updated_at: now,
        })
    }

    pub fn trip_get(&self, trip_id: &str) -> Result<Trip, TravelCraftError> {
        let conn = self.lock();
        let row = conn
            .query_row(
                "SELECT trip_id, title, start_date, end_date, cartridge_ids_json, status, notes, created_at, updated_at
                 FROM trips WHERE trip_id = ?1",
                params![trip_id],
                map_trip_row,
            )
            .optional()
            .map_err(db_err)?;
        row.ok_or_else(|| TravelCraftError::NotFound(format!("trip '{trip_id}'")))
    }

    pub fn trip_list(&self) -> Result<Vec<Trip>, TravelCraftError> {
        let conn = self.lock();
        let mut stmt = conn
            .prepare(
                "SELECT trip_id, title, start_date, end_date, cartridge_ids_json, status, notes, created_at, updated_at
                 FROM trips ORDER BY updated_at DESC, trip_id DESC",
            )
            .map_err(db_err)?;
        let rows = stmt.query_map([], map_trip_row).map_err(db_err)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(db_err)?);
        }
        Ok(out)
    }

    pub fn trip_update(&self, trip_id: &str, patch: &TripPatch) -> Result<Trip, TravelCraftError> {
        let mut trip = self.trip_get(trip_id)?;
        if let Some(v) = &patch.title {
            Self::check_text("trip title", v, MAX_FIELD_CHARS, true)?;
            trip.title = v.trim().to_string();
        }
        if let Some(v) = &patch.start_date {
            trip.start_date = if v.trim().is_empty() {
                None
            } else {
                Some(v.trim().to_string())
            };
        }
        if let Some(v) = &patch.end_date {
            trip.end_date = if v.trim().is_empty() {
                None
            } else {
                Some(v.trim().to_string())
            };
        }
        if let Some(v) = &patch.cartridge_ids {
            trip.cartridge_ids = v.clone();
        }
        if let Some(v) = patch.status {
            trip.status = v;
        }
        if let Some(v) = &patch.notes {
            Self::check_text("trip notes", v, MAX_TEXT_CHARS, false)?;
            trip.notes = v.clone();
        }
        trip.updated_at = chrono::Utc::now().timestamp();
        let cart_json = serde_json::to_string(&trip.cartridge_ids)?;
        self.lock()
            .execute(
                "UPDATE trips SET title = ?1, start_date = ?2, end_date = ?3,
                 cartridge_ids_json = ?4, status = ?5, notes = ?6, updated_at = ?7
                 WHERE trip_id = ?8",
                params![
                    &trip.title,
                    &trip.start_date,
                    &trip.end_date,
                    &cart_json,
                    trip.status.as_str(),
                    &trip.notes,
                    trip.updated_at,
                    &trip.trip_id,
                ],
            )
            .map_err(db_err)?;
        Ok(trip)
    }

    /// Delete a trip. If `delete_entries` is false, entries are kept in the universal notebook with `trip_id = NULL`.
    pub fn trip_delete(&self, trip_id: &str, delete_entries: bool) -> Result<(), TravelCraftError> {
        let conn = self.lock();
        if delete_entries {
            conn.execute(
                "DELETE FROM notebook_entries WHERE trip_id = ?1",
                params![trip_id],
            )
            .map_err(db_err)?;
        } else {
            conn.execute(
                "UPDATE notebook_entries SET trip_id = NULL WHERE trip_id = ?1",
                params![trip_id],
            )
            .map_err(db_err)?;
        }
        let n = conn
            .execute("DELETE FROM trips WHERE trip_id = ?1", params![trip_id])
            .map_err(db_err)?;
        if n == 0 {
            return Err(TravelCraftError::NotFound(format!("trip '{trip_id}'")));
        }
        Ok(())
    }

    /// Add an entry to the universal travel notebook.
    pub fn notebook_add(&self, req: &NewNotebookEntry) -> Result<NotebookEntry, TravelCraftError> {
        Self::check_text("entry title", &req.title, MAX_FIELD_CHARS, false)?;
        Self::check_text("entry body", &req.body, MAX_TEXT_CHARS, true)?;
        Self::check_text("sensory_notes", &req.sensory_notes, MAX_TEXT_CHARS, false)?;
        if req.photo_uris.len() > MAX_PHOTOS {
            return Err(TravelCraftError::Invalid(format!(
                "more than {MAX_PHOTOS} photos"
            )));
        }
        for uri in &req.photo_uris {
            Self::check_text("photo uri", uri, MAX_FIELD_CHARS, true)?;
        }
        validate_coords(req.lat, req.lon)?;
        let country_code = normalize_country(req.country_code.as_deref())?;
        let tags = validate_tags(&req.tags)?;

        let entry_id = self.new_entry_id("NB");
        let now = chrono::Utc::now().timestamp();
        let photos_json = serde_json::to_string(&req.photo_uris)?;
        let tags_json = serde_json::to_string(&tags)?;

        let conn = self.lock();
        if let Some(tid) = &req.trip_id {
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM trips WHERE trip_id = ?1",
                    params![tid],
                    |_| Ok(true),
                )
                .optional()
                .map_err(db_err)?
                .unwrap_or(false);
            if !exists {
                return Err(TravelCraftError::NotFound(format!("trip '{tid}'")));
            }
        }
        conn.execute(
            "INSERT INTO notebook_entries
             (entry_id, trip_id, cartridge_id, country_code, waypoint_id, kind, day_index,
              timestamp_epoch, title, body, sensory_notes, photo_uris_json, lat, lon, tags_json, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                &entry_id,
                &req.trip_id,
                &req.cartridge_id,
                &country_code,
                &req.waypoint_id,
                req.kind.as_str(),
                req.day_index.map(i64::from),
                now,
                req.title.trim(),
                &req.body,
                &req.sensory_notes,
                &photos_json,
                req.lat,
                req.lon,
                &tags_json,
                now,
            ],
        )
        .map_err(db_err)?;

        Ok(NotebookEntry {
            entry_id,
            trip_id: req.trip_id.clone(),
            cartridge_id: req.cartridge_id.clone(),
            country_code,
            waypoint_id: req.waypoint_id.clone(),
            kind: req.kind,
            day_index: req.day_index,
            timestamp_epoch: now,
            title: req.title.trim().to_string(),
            body: req.body.clone(),
            sensory_notes: req.sensory_notes.clone(),
            photo_uris: req.photo_uris.clone(),
            lat: req.lat,
            lon: req.lon,
            tags,
            updated_at: now,
        })
    }

    pub fn notebook_get(&self, entry_id: &str) -> Result<NotebookEntry, TravelCraftError> {
        let conn = self.lock();
        let row = conn
            .query_row(
                "SELECT entry_id, trip_id, cartridge_id, country_code, waypoint_id, kind, day_index,
                        timestamp_epoch, title, body, sensory_notes, photo_uris_json, lat, lon, tags_json, updated_at
                 FROM notebook_entries WHERE entry_id = ?1",
                params![entry_id],
                map_notebook_row,
            )
            .optional()
            .map_err(db_err)?;
        row.ok_or_else(|| TravelCraftError::NotFound(format!("notebook entry '{entry_id}'")))
    }

    pub fn notebook_update(
        &self,
        entry_id: &str,
        patch: &NotebookPatch,
    ) -> Result<NotebookEntry, TravelCraftError> {
        let mut e = self.notebook_get(entry_id)?;
        if let Some(v) = &patch.trip_id {
            e.trip_id = if v.trim().is_empty() {
                None
            } else {
                Some(v.trim().to_string())
            };
        }
        if let Some(v) = &patch.cartridge_id {
            e.cartridge_id = if v.trim().is_empty() {
                None
            } else {
                Some(v.trim().to_string())
            };
        }
        if let Some(v) = &patch.country_code {
            e.country_code = normalize_country(Some(v))?;
        }
        if let Some(v) = &patch.waypoint_id {
            e.waypoint_id = if v.trim().is_empty() {
                None
            } else {
                Some(v.trim().to_string())
            };
        }
        if let Some(k) = patch.kind {
            e.kind = k;
        }
        if let Some(d) = patch.day_index {
            e.day_index = Some(d);
        }
        if let Some(v) = &patch.title {
            Self::check_text("entry title", v, MAX_FIELD_CHARS, false)?;
            e.title = v.trim().to_string();
        }
        if let Some(v) = &patch.body {
            Self::check_text("entry body", v, MAX_TEXT_CHARS, true)?;
            e.body = v.clone();
        }
        if let Some(v) = &patch.sensory_notes {
            Self::check_text("sensory_notes", v, MAX_TEXT_CHARS, false)?;
            e.sensory_notes = v.clone();
        }
        if let Some(v) = &patch.photo_uris {
            if v.len() > MAX_PHOTOS {
                return Err(TravelCraftError::Invalid(format!(
                    "more than {MAX_PHOTOS} photos"
                )));
            }
            e.photo_uris = v.clone();
        }
        if let Some(la) = patch.lat {
            e.lat = Some(la);
        }
        if let Some(lo) = patch.lon {
            e.lon = Some(lo);
        }
        validate_coords(e.lat, e.lon)?;
        if let Some(v) = &patch.tags {
            e.tags = validate_tags(v)?;
        }
        e.updated_at = chrono::Utc::now().timestamp();
        let photos_json = serde_json::to_string(&e.photo_uris)?;
        let tags_json = serde_json::to_string(&e.tags)?;

        self.lock()
            .execute(
                "UPDATE notebook_entries
                 SET trip_id = ?1, cartridge_id = ?2, country_code = ?3, waypoint_id = ?4,
                     kind = ?5, day_index = ?6, title = ?7, body = ?8, sensory_notes = ?9,
                     photo_uris_json = ?10, lat = ?11, lon = ?12, tags_json = ?13, updated_at = ?14
                 WHERE entry_id = ?15",
                params![
                    &e.trip_id,
                    &e.cartridge_id,
                    &e.country_code,
                    &e.waypoint_id,
                    e.kind.as_str(),
                    e.day_index.map(i64::from),
                    &e.title,
                    &e.body,
                    &e.sensory_notes,
                    &photos_json,
                    e.lat,
                    e.lon,
                    &tags_json,
                    e.updated_at,
                    &e.entry_id,
                ],
            )
            .map_err(db_err)?;
        Ok(e)
    }

    pub fn notebook_delete(&self, entry_id: &str) -> Result<(), TravelCraftError> {
        let n = self
            .lock()
            .execute(
                "DELETE FROM notebook_entries WHERE entry_id = ?1",
                params![entry_id],
            )
            .map_err(db_err)?;
        if n == 0 {
            return Err(TravelCraftError::NotFound(format!(
                "notebook entry '{entry_id}'"
            )));
        }
        Ok(())
    }

    /// Scoped query across the universal notebook (This Trip | This City | This Country | All).
    pub fn notebook_query(
        &self,
        q: &NotebookQuery,
    ) -> Result<Vec<NotebookEntry>, TravelCraftError> {
        let limit = q.limit.unwrap_or(100).clamp(1, 1000);
        let search_lc = q
            .search
            .as_deref()
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty());
        let tag_lc = q
            .tag
            .as_deref()
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty());
        let cc_uc = q
            .country_code
            .as_deref()
            .map(|s| s.trim().to_ascii_uppercase())
            .filter(|s| !s.is_empty());

        let conn = self.lock();
        let mut stmt = conn
            .prepare(
                "SELECT entry_id, trip_id, cartridge_id, country_code, waypoint_id, kind, day_index,
                        timestamp_epoch, title, body, sensory_notes, photo_uris_json, lat, lon, tags_json, updated_at
                 FROM notebook_entries ORDER BY timestamp_epoch DESC, entry_id DESC",
            )
            .map_err(db_err)?;
        let rows = stmt.query_map([], map_notebook_row).map_err(db_err)?;
        let mut out = Vec::new();
        for r in rows {
            let e = r.map_err(db_err)?;
            if let Some(tid) = &q.trip_id {
                if e.trip_id.as_deref() != Some(tid.as_str()) {
                    continue;
                }
            }
            if let Some(cid) = &q.cartridge_id {
                if e.cartridge_id.as_deref() != Some(cid.as_str()) {
                    continue;
                }
            }
            if let Some(cc) = &cc_uc {
                if e.country_code.as_deref() != Some(cc.as_str()) {
                    continue;
                }
            }
            if let Some(wid) = &q.waypoint_id {
                if e.waypoint_id.as_deref() != Some(wid.as_str()) {
                    continue;
                }
            }
            if let Some(kind) = q.kind {
                if e.kind != kind {
                    continue;
                }
            }
            if let Some(tag) = &tag_lc {
                if !e.tags.iter().any(|t| t == tag) {
                    continue;
                }
            }
            if let Some(needle) = &search_lc {
                let hay = format!(
                    "{} {} {}",
                    e.title.to_ascii_lowercase(),
                    e.body.to_ascii_lowercase(),
                    e.sensory_notes.to_ascii_lowercase()
                );
                if !hay.contains(needle) {
                    continue;
                }
            }
            out.push(e);
            if out.len() >= limit {
                break;
            }
        }
        Ok(out)
    }

    /// Compile a standalone Markdown travelogue for a trip from the universal notebook.
    pub fn trip_export_markdown(&self, trip_id: &str) -> Result<String, TravelCraftError> {
        let trip = self.trip_get(trip_id)?;
        let mut entries = self.notebook_query(&NotebookQuery {
            trip_id: Some(trip_id.to_string()),
            limit: Some(1000),
            ..NotebookQuery::default()
        })?;
        entries.sort_by_key(|e| (e.day_index.unwrap_or(0), e.timestamp_epoch));

        let mut md = String::new();
        let _ = writeln!(md, "# {}\n", trip.title);
        let _ = writeln!(md, "- Trip ID: `{}`", trip.trip_id);
        let _ = writeln!(md, "- Status: `{}`", trip.status.as_str());
        if trip.start_date.is_some() || trip.end_date.is_some() {
            let _ = writeln!(
                md,
                "- Dates: {} to {}",
                trip.start_date.as_deref().unwrap_or("?"),
                trip.end_date.as_deref().unwrap_or("?")
            );
        }
        if !trip.cartridge_ids.is_empty() {
            let _ = writeln!(md, "- Cartridges: {}", trip.cartridge_ids.join(", "));
        }
        if !trip.notes.trim().is_empty() {
            let _ = writeln!(md, "\n{}\n", trip.notes.trim());
        } else {
            md.push('\n');
        }

        if entries.is_empty() {
            md.push_str("_No notebook entries recorded yet._\n");
            return Ok(md);
        }

        let mut grouped: BTreeMap<u32, Vec<&NotebookEntry>> = BTreeMap::new();
        for e in &entries {
            grouped.entry(e.day_index.unwrap_or(0)).or_default().push(e);
        }

        for (day, list) in grouped {
            if day == 0 {
                md.push_str("## General / Pre-Trip Notes\n\n");
            } else {
                let _ = writeln!(md, "## Day {day}\n");
            }
            for e in list {
                let heading = if e.title.is_empty() {
                    format!("[{}]", e.kind.as_str())
                } else {
                    format!("{} [{}]", e.title, e.kind.as_str())
                };
                let _ = writeln!(md, "### {heading}\n");
                let mut meta = Vec::new();
                if let Some(cid) = &e.cartridge_id {
                    meta.push(format!("cartridge: `{cid}`"));
                }
                if let Some(cc) = &e.country_code {
                    meta.push(format!("country: `{cc}`"));
                }
                if let Some(wid) = &e.waypoint_id {
                    meta.push(format!("waypoint: `{wid}`"));
                }
                if !e.tags.is_empty() {
                    meta.push(format!("tags: {}", e.tags.join(", ")));
                }
                if !meta.is_empty() {
                    let _ = writeln!(md, "_{}_\n", meta.join(" | "));
                }
                let _ = writeln!(md, "{}\n", e.body.trim());
                if !e.sensory_notes.trim().is_empty() {
                    let _ = writeln!(md, "> Sensory: {}\n", e.sensory_notes.trim());
                }
                if !e.photo_uris.is_empty() {
                    let _ = writeln!(md, "Photos: {}\n", e.photo_uris.join(", "));
                }
            }
        }
        Ok(md)
    }
}

fn map_trip_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Trip> {
    let cart_json: String = r.get(4)?;
    let cartridge_ids: Vec<String> = serde_json::from_str(&cart_json).unwrap_or_default();
    let status_raw: String = r.get(5)?;
    let status = TripStatus::parse(&status_raw).unwrap_or(TripStatus::Planned);
    Ok(Trip {
        trip_id: r.get(0)?,
        title: r.get(1)?,
        start_date: r.get(2)?,
        end_date: r.get(3)?,
        cartridge_ids,
        status,
        notes: r.get(6)?,
        created_at: r.get(7)?,
        updated_at: r.get(8)?,
    })
}

fn map_notebook_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<NotebookEntry> {
    let kind_raw: String = r.get(5)?;
    let kind = NotebookKind::parse(&kind_raw).unwrap_or(NotebookKind::Reflection);
    let day_raw: Option<i64> = r.get(6)?;
    let day_index = day_raw.and_then(|d| u32::try_from(d).ok());
    let photos_json: String = r.get(11)?;
    let photo_uris: Vec<String> = serde_json::from_str(&photos_json).unwrap_or_default();
    let tags_json: String = r.get(14)?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
    Ok(NotebookEntry {
        entry_id: r.get(0)?,
        trip_id: r.get(1)?,
        cartridge_id: r.get(2)?,
        country_code: r.get(3)?,
        waypoint_id: r.get(4)?,
        kind,
        day_index,
        timestamp_epoch: r.get(7)?,
        title: r.get(8)?,
        body: r.get(9)?,
        sensory_notes: r.get(10)?,
        photo_uris,
        lat: r.get(12)?,
        lon: r.get(13)?,
        tags,
        updated_at: r.get(15)?,
    })
}
